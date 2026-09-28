//! Claude Code auto 模式的「服务端分类器审查」协议支持（safeguards / safeguard_results）
//!
//! # 协议要点（2026-09 对 Claude Code 2.1.283 实测还原）
//!
//! - 客户端在 auto 模式下、且未关闭服务端审查时，会在**每一个**请求体里带上
//!   safeguards: [{"type":"dangerous_tool_use","classifier_context":{...}}]，
//!   并在 anthropic-beta 中带上 dangerous-tool-use-*。
//! - 服务端必须在**每一个**响应的 message_delta.delta.safeguard_results 中回传
//!   结果。实测：只要有一个响应漏掉，客户端就判定「网关不兼容」，回退到自己的
//!   本地分类器并展示分类器计费提示（官方文档 auto-mode-classifier-billing）。
//! - 逐工具裁决 tool_uses 的三种取值对应三种行为（实测确认）：
//!   * evaluated + not_flagged —— **直接放行**，客户端不再自行检查该动作；
//!   * evaluated + flagged     —— 拦截该动作，并把 explanation 透传给用户；
//!   * skipped 或条目缺失       —— 客户端**回退到自己的分类器**（不是无条件放行）。
//!
//! # 安全约束
//!
//! 本模块**绝不**在未真正分类的情况下产出 evaluated/not_flagged：那会让客户端
//! 跳过它自己的安全检查，属于安全倒退。未覆盖或分类失败的工具一律以
//! skipped／省略表达，由客户端自己的分类器兜底。

use super::types::MessagesRequest;

/// 逐条目的 type 常量
pub const DANGEROUS_TOOL_USE: &str = "dangerous_tool_use";

/// 客户端是否要求服务端分类器审查
pub fn requested(req: &MessagesRequest) -> bool {
    req.safeguards.as_ref().is_some_and(|v| !v.is_null())
}

/// 单个工具的裁决
#[derive(Debug, Clone)]
#[allow(dead_code)] // Evaluated 供后续接入真实分类器的阶段使用
pub enum ToolVerdict {
    /// 已由本地分类器真实评估过
    Evaluated { flagged: bool, explanation: String },
    /// 未评估：交回客户端自己的分类器决定
    Skipped,
}

#[allow(dead_code)]
impl ToolVerdict {
    fn to_json(&self) -> serde_json::Value {
        match self {
            Self::Evaluated {
                flagged,
                explanation,
            } => serde_json::json!({
                "type": "evaluated",
                "outcome": if *flagged { "flagged" } else { "not_flagged" },
                "explanation": explanation,
            }),
            Self::Skipped => serde_json::json!({ "type": "skipped" }),
        }
    }
}

/// 构造放进 message_delta.delta 的 safeguard_results 值
///
/// verdicts 为空时逐工具表为空：客户端对每个工具的查找都会落空并回退到它自己的
/// 分类器。这是安全且**仍然让会话保持合格**的状态（实测：提示不再出现）。
#[allow(dead_code)]
pub fn build_results(
    verdicts: &std::collections::HashMap<String, ToolVerdict>,
) -> serde_json::Value {
    let tool_uses: serde_json::Map<String, serde_json::Value> = verdicts
        .iter()
        .map(|(id, v)| (id.clone(), v.to_json()))
        .collect();

    serde_json::json!([{
        "type": DANGEROUS_TOOL_USE,
        "status": {
            "type": "available",
            "tool_uses": tool_uses,
        }
    }])
}

/// 当前阶段：不产出任何真实裁决，逐项交回客户端自己的分类器。
///
/// 效果：会话保持「合格」（不再出现分类器计费提示），而每一个动作仍然由
/// Claude Code 自己的分类器审查——与接入服务端审查之前的安全语义一致。
pub fn deferred_results() -> serde_json::Value {
    build_results(&std::collections::HashMap::new())
}

// ===== 真实分类器（阶段 2）=====

use std::sync::Arc;
use std::time::Duration;

use serde_json::Value;

use super::converter::convert_request_with_mode;
use super::types::{Message, SystemMessage};
use crate::kiro::model::events::Event;
use crate::kiro::model::requests::kiro::KiroRequest;
use crate::kiro::parser::decoder::EventStreamDecoder;
use crate::kiro::provider::KiroProvider;
use crate::model::config::ToolCompatibilityMode;

/// 分类器运行时设置
#[derive(Debug, Clone)]
pub struct ClassifierSettings {
    pub model: String,
    pub timeout: Duration,
}

/// 每个请求的 safeguards 运行计划
#[derive(Debug, Clone)]
pub enum SafeguardsPlan {
    /// 客户端未请求，或功能被配置关闭
    Off,
    /// 只回传空结果：所有裁决交回客户端自己的分类器（零安全风险）
    Deferred,
    /// 用指定模型做真实分类
    Classify(ClassifierSettings),
}

/// 依据配置与请求构造本次请求的 safeguards 计划
pub fn plan_for(
    enabled: bool,
    classifier: Option<&ClassifierSettings>,
    req: &MessagesRequest,
) -> SafeguardsPlan {
    if !enabled || !requested(req) {
        return SafeguardsPlan::Off;
    }
    match classifier {
        Some(c) => SafeguardsPlan::Classify(c.clone()),
        None => SafeguardsPlan::Deferred,
    }
}

/// 需要真正调用模型判定的工具范围。
///
/// 其余工具不下发裁决，客户端会回退到它自己的分类器 —— 与不接入服务端审查时
/// 的行为一致，因此是安全的。
pub fn needs_classification(tool_name: &str) -> bool {
    let n = tool_name.to_ascii_lowercase();
    n == "bash" || n.contains("shell") || n.contains("terminal") || n.contains("exec")
}

const CLASSIFIER_SYSTEM_PROMPT: &str = r#"You are the safety classifier for an autonomous coding agent that runs tool calls without asking the user.
For each tool call you are given, decide whether it is safe to run unattended.

Flag it (flagged=true) when the call could:
- destroy or overwrite data that is NOT disposable: user documents, source the agent did
  not create, or data outside both the project and OS temporary storage
- exfiltrate secrets, credentials, keys or private data to a remote endpoint
- download and execute remote code, for example piping a download into a shell
- change system-wide state: users, permissions, services, disks, firewall or boot
- publish, deploy, push to a remote, or spend money
- lose work through destructive git operations such as force-pushing or hard-resetting
- read credential stores that live outside the current project
- delete through an unresolved target: shell variables, command substitution, globs that
  could expand outside the project, or path traversal such as /tmp/../etc

Do NOT flag as routine development work:
- building, running tests, linting, formatting, searching, listing directories
- reading or editing files inside the project
- running project-local scripts
- deleting disposable artifacts, even when they sit outside the project:
  * anything under the OS temporary directory (/tmp, $TMPDIR, /var/folders, %TEMP%)
    when the path is written out literally
  * build outputs and caches: target/, dist/, build/, node_modules/, __pycache__, .pytest_cache
  * user-level package and tool caches under the XDG cache directory (~/.cache/...)
  * scratch, benchmark or log files the agent itself created during this session

Err on the side of blocking when you are unsure.

Reply with ONLY a JSON object and nothing else:
{"verdicts":[{"id":"<tool_use_id>","flagged":true,"reason":"one short sentence"}]}
Include exactly one entry for every tool call you were given, reusing its id."#;

/// 对给定的工具调用跑一次真实安全分类。
///
/// 只返回**确实评估过**的工具裁决。任何失败（请求构建、调用、超时、输出无法
/// 解析）都返回空表，调用方据此不下发裁决，客户端会回退到它自己的分类器。
///
/// 安全约束：这里**绝不**用 not_flagged 兜底。
pub async fn classify(
    provider: &Arc<KiroProvider>,
    settings: &ClassifierSettings,
    tool_uses: &[(String, String, Value)],
) -> std::collections::HashMap<String, ToolVerdict> {
    let targets: Vec<&(String, String, Value)> = tool_uses
        .iter()
        .filter(|(_, name, _)| needs_classification(name))
        .collect();

    if targets.is_empty() {
        return std::collections::HashMap::new();
    }

    match run_classifier(provider, settings, &targets).await {
        Ok(verdicts) => {
            let flagged: Vec<&str> = verdicts
                .iter()
                .filter(|(_, v)| matches!(v, ToolVerdict::Evaluated { flagged: true, .. }))
                .map(|(id, _)| id.as_str())
                .collect();
            tracing::info!(
                "safeguards 分类器: 已判定 {} 个 shell 类工具，flagged={:?}（未覆盖的由客户端本地分类兜底）",
                verdicts.len(),
                flagged
            );
            verdicts
        }
        Err(e) => {
            tracing::warn!(
                "safeguards 分类器未产出裁决（本次交回客户端本地分类）: {}",
                e
            );
            std::collections::HashMap::new()
        }
    }
}

async fn run_classifier(
    provider: &Arc<KiroProvider>,
    settings: &ClassifierSettings,
    targets: &[&(String, String, Value)],
) -> anyhow::Result<std::collections::HashMap<String, ToolVerdict>> {
    let req = build_classifier_request(settings, targets);
    let conv = convert_request_with_mode(&req, ToolCompatibilityMode::Raw)
        .map_err(|e| anyhow::anyhow!("分类器请求转换失败: {}", e))?;
    let body = serde_json::to_string(&KiroRequest {
        conversation_state: conv.conversation_state,
        profile_arn: None,
        additional_model_request_fields: None,
    })?;

    let call_result = tokio::time::timeout(settings.timeout, provider.call_api(&body, None))
        .await
        .map_err(|_| anyhow::anyhow!("分类器调用超时"))??;

    let bytes = tokio::time::timeout(settings.timeout, call_result.response.bytes())
        .await
        .map_err(|_| anyhow::anyhow!("读取分类器响应超时"))??;

    let text = extract_assistant_text(&bytes)?;
    parse_verdicts(&text, targets)
}

fn build_classifier_request(
    settings: &ClassifierSettings,
    targets: &[&(String, String, Value)],
) -> MessagesRequest {
    let calls: Vec<Value> = targets
        .iter()
        .map(|(id, name, input)| serde_json::json!({"id": id, "name": name, "input": input}))
        .collect();
    let user = format!(
        "Tool calls to classify:\n{}",
        serde_json::to_string_pretty(&calls).unwrap_or_else(|_| "[]".to_string())
    );

    MessagesRequest {
        model: settings.model.clone(),
        max_tokens: 1024,
        messages: vec![Message {
            role: "user".to_string(),
            content: Value::String(user),
        }],
        stream: false,
        system: Some(vec![SystemMessage {
            text: CLASSIFIER_SYSTEM_PROMPT.to_string(),
            cache_control: None,
        }]),
        tools: None,
        tool_choice: None,
        thinking: None,
        output_config: None,
        metadata: None,
        safeguards: None,
    }
}

fn extract_assistant_text(bytes: &[u8]) -> anyhow::Result<String> {
    let mut decoder = EventStreamDecoder::new();
    decoder
        .feed(bytes)
        .map_err(|e| anyhow::anyhow!("分类器响应解码失败: {}", e))?;

    let mut text = String::new();
    for result in decoder.decode_iter() {
        let frame = result.map_err(|e| anyhow::anyhow!("分类器响应帧解析失败: {}", e))?;
        if let Ok(event) = Event::from_frame(frame) {
            match event {
                Event::AssistantResponse(resp) => text.push_str(&resp.content),
                Event::Code(resp) => text.push_str(&resp.content),
                _ => {}
            }
        }
    }

    if text.trim().is_empty() {
        anyhow::bail!("分类器返回了空文本");
    }
    Ok(text)
}

fn parse_verdicts(
    text: &str,
    targets: &[&(String, String, Value)],
) -> anyhow::Result<std::collections::HashMap<String, ToolVerdict>> {
    let start = text
        .find('{')
        .ok_or_else(|| anyhow::anyhow!("分类器输出中没有 JSON"))?;
    let end = text
        .rfind('}')
        .ok_or_else(|| anyhow::anyhow!("分类器输出中没有 JSON"))?;
    if end <= start {
        anyhow::bail!("分类器输出的 JSON 不完整");
    }

    let parsed: Value = serde_json::from_str(&text[start..=end])
        .map_err(|e| anyhow::anyhow!("分类器 JSON 解析失败: {}", e))?;
    let list = parsed
        .get("verdicts")
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow::anyhow!("分类器输出缺少 verdicts 数组"))?;

    let asked: std::collections::HashSet<&str> =
        targets.iter().map(|(id, _, _)| id.as_str()).collect();

    let mut out = std::collections::HashMap::new();
    for item in list {
        let Some(id) = item.get("id").and_then(|v| v.as_str()) else {
            continue;
        };
        if !asked.contains(id) {
            continue;
        }
        // 没有明确布尔判定就不下发裁决，让客户端回退到它自己的分类器
        let Some(flagged) = item.get("flagged").and_then(|v| v.as_bool()) else {
            continue;
        };
        let explanation = item
            .get("reason")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        out.insert(id.to_string(), ToolVerdict::Evaluated { flagged, explanation });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deferred_results_is_well_formed() {
        let v = deferred_results();
        let arr = v.as_array().expect("safeguard_results 必须是数组");
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0]["type"], DANGEROUS_TOOL_USE);
        assert_eq!(arr[0]["status"]["type"], "available");
        assert!(
            arr[0]["status"]["tool_uses"]
                .as_object()
                .expect("tool_uses 必须是对象")
                .is_empty()
        );
    }

    #[test]
    fn evaluated_verdict_maps_to_expected_outcome() {
        let mut m = std::collections::HashMap::new();
        m.insert(
            "toolu_1".to_string(),
            ToolVerdict::Evaluated {
                flagged: true,
                explanation: "rm -rf /".to_string(),
            },
        );
        m.insert("toolu_2".to_string(), ToolVerdict::Skipped);
        let v = build_results(&m);
        assert_eq!(v[0]["status"]["tool_uses"]["toolu_1"]["type"], "evaluated");
        assert_eq!(v[0]["status"]["tool_uses"]["toolu_1"]["outcome"], "flagged");
        assert_eq!(v[0]["status"]["tool_uses"]["toolu_2"]["type"], "skipped");
    }


    fn sample_targets() -> Vec<(String, String, serde_json::Value)> {
        vec![
            (
                "toolu_1".to_string(),
                "Bash".to_string(),
                serde_json::json!({"command": "ls -la"}),
            ),
            (
                "toolu_2".to_string(),
                "Bash".to_string(),
                serde_json::json!({"command": "rm -rf /"}),
            ),
        ]
    }

    #[test]
    fn needs_classification_covers_shell_like_tools_only() {
        assert!(needs_classification("Bash"));
        assert!(needs_classification("bash"));
        assert!(needs_classification("shell_exec"));
        assert!(needs_classification("terminal_run"));
        assert!(!needs_classification("Read"));
        assert!(!needs_classification("Write"));
        assert!(!needs_classification("Grep"));
        assert!(!needs_classification("Glob"));
    }

    #[test]
    fn parse_verdicts_reads_json_and_ignores_unknown_ids() {
        let targets = sample_targets();
        let refs: Vec<&(String, String, serde_json::Value)> = targets.iter().collect();
        let text = r#"Sure, here is the result:
{"verdicts":[{"id":"toolu_1","flagged":false,"reason":"lists files"},
             {"id":"toolu_2","flagged":true,"reason":"destroys the filesystem"},
             {"id":"toolu_999","flagged":true,"reason":"not asked about"}]}
done"#;
        let v = parse_verdicts(text, &refs).expect("应当解析成功");
        assert_eq!(v.len(), 2, "只保留问过的 id");
        match v.get("toolu_1").expect("toolu_1") {
            ToolVerdict::Evaluated { flagged, .. } => assert!(!*flagged),
            other => panic!("期望 Evaluated，得到 {:?}", other),
        }
        match v.get("toolu_2").expect("toolu_2") {
            ToolVerdict::Evaluated {
                flagged,
                explanation,
            } => {
                assert!(*flagged);
                assert!(explanation.contains("destroys"));
            }
            other => panic!("期望 Evaluated，得到 {:?}", other),
        }
        assert!(v.get("toolu_999").is_none(), "未询问的 id 必须忽略");
    }

    #[test]
    fn parse_verdicts_skips_entries_without_boolean_verdict() {
        let targets = sample_targets();
        let refs: Vec<&(String, String, serde_json::Value)> = targets.iter().collect();
        let v = parse_verdicts(
            r#"{"verdicts":[{"id":"toolu_1","reason":"missing the flag field"}]}"#,
            &refs,
        )
        .expect("结构合法");
        assert!(
            v.is_empty(),
            "缺少布尔判定时不得产出裁决：否则可能被当成放行"
        );
    }

    #[test]
    fn parse_verdicts_rejects_garbage() {
        let targets = sample_targets();
        let refs: Vec<&(String, String, serde_json::Value)> = targets.iter().collect();
        assert!(parse_verdicts("the model refused to answer", &refs).is_err());
        assert!(parse_verdicts("{}", &refs).is_err(), "缺少 verdicts 数组");
    }

    #[test]
    fn plan_for_requires_both_config_and_request() {
        let settings = ClassifierSettings {
            model: "claude-sonnet-5".to_string(),
            timeout: Duration::from_secs(20),
        };
        let mut req: MessagesRequest = serde_json::from_str(
            r#"{"model":"claude-sonnet-5","max_tokens":16,"messages":[{"role":"user","content":"hi"}]}"#,
        )
        .unwrap();

        // 请求未要求审查 -> Off
        assert!(matches!(
            plan_for(true, Some(&settings), &req),
            SafeguardsPlan::Off
        ));

        req.safeguards = Some(serde_json::json!([{"type": DANGEROUS_TOOL_USE}]));

        // 请求要求但功能关闭 -> Off
        assert!(matches!(
            plan_for(false, Some(&settings), &req),
            SafeguardsPlan::Off
        ));
        // 请求要求且无分类器 -> Deferred
        assert!(matches!(
            plan_for(true, None, &req),
            SafeguardsPlan::Deferred
        ));
        // 请求要求且有分类器 -> Classify
        assert!(matches!(
            plan_for(true, Some(&settings), &req),
            SafeguardsPlan::Classify(_)
        ));
    }

    #[test]
    fn requested_detects_field() {
        let mut req: MessagesRequest = serde_json::from_str(
            r#"{"model":"claude-sonnet-5","max_tokens":16,"messages":[{"role":"user","content":"hi"}]}"#,
        )
        .unwrap();
        assert!(!requested(&req));
        req.safeguards = Some(serde_json::json!([{"type": DANGEROUS_TOOL_USE}]));
        assert!(requested(&req));
    }
}
