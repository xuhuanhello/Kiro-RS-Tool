# Kiro-RS-Tool

一个面向 Claude Code 工具调用兼容的 Kiro / Anthropic API 代理工具，将 Anthropic API 请求转换为 Kiro API 请求。

## 本增强版说明

这是面向 Claude Code 与新版 Kiro 的增强兼容分支，目标是在自托管环境中作为稳定的 Anthropic API 兼容代理使用。

## 重点：Claude Code 工具调用兼容

本分支的核心目标是让 Claude Code 能稳定走 Kiro 上游完成真实开发任务，重点修复了 Claude Code 工具调用在 Kiro 协议下最容易出问题的几类场景：

- **工具 schema 与参数双向适配**：把 Claude Code 的 `Read`、`Write`、`Edit`、`Bash`、`Glob`、`Grep`、`LS`、`WebSearch` 等工具映射到 Kiro 内置工具，并在上游返回时恢复为 Claude Code 期望的工具名和参数结构。
- **Write / Edit / Read 参数兼容**：处理 `file_path`、`content`、`old_string`、`new_string`、`offset`、`limit` 等字段与 Kiro `path`、`text`、`oldStr`、`newStr`、`start_line`、`end_line` 的差异，避免客户端收到 `Invalid tool parameters`。
- **流式半截 JSON 防护**：对上游分片 `toolUse.input` 做完整 JSON 累积，只有参数完整且可解析后才向 Claude Code 输出 tool_use，防止半截 `input_json_delta` 触发工具执行失败。
- **工具调用 XML 泄漏过滤**：过滤上游可能混入正文的 tool_use XML 片段，避免 Claude Code 把泄漏内容当作普通文本处理。
- **大文件写入/编辑场景加固**：针对 Claude Code 常见的大 `Write` / `Edit` 参数和长输出场景做兼容处理，减少会话卡死、工具参数错配和客户端中断。

## 其他能力

- **Kiro CLI / Kiro IDE 端点兼容**：保留 `ide` 与 `cli` 两套 endpoint transform，分别处理 origin、headers、profileArn、content type、CLI wrapping 与 IDE agent mode。
- **Opus thinking / effort 兼容**：Opus 4.8 / 4.7 / 4.6 支持 thinking 模型和 effort 参数，包含 Opus 4.8 `high` / `xhigh` 等强度映射。
- **Sonnet thinking 兼容**：Sonnet 4.6 保留 thinking 兼容；流式 thinking 块会转换为 Anthropic 兼容内容块。
- **profileArn 完整修复**：Enterprise / IdC 账号会自动通过 `ListAvailableProfiles` 获取真实 profileArn 并写回；Builder ID/free 流式请求保留官方占位 profileArn，避免新版上游返回 `profileArn is required`。
- **Admin 面板增强**：支持客户端 Key 分发、用量统计、请求链路追踪、全局设置、默认 endpoint 切换、凭据级 endpoint 覆盖、代理池、在线更新与 KAM 导入。
- **缓存 token 计量**：按 cache read / cache creation / input / output 记录用量，并在请求链路追踪中展示 token 明细。

## 引用与合并来源

本仓库不是从零开始的独立实现，而是在多个公开社区项目和本地补丁基础上整理、合并、实测后的增强分支。主要参考与合并来源如下：

- [hank9999/kiro.rs](https://github.com/hank9999/kiro.rs)：原始 Rust 版 Kiro/Anthropic API 兼容代理基础实现，以及多凭据、Admin、Docker 部署等核心思路。
- [ZyphrZero/kiro.rs](https://github.com/ZyphrZero/kiro.rs)：当前主要基线；参考并合入 Enterprise profileArn、cache metering、trace token 字段、Admin UI 组件统一、Builder ID/free profileArn 等修复。
- [Foxfishc/kiro.rs](https://github.com/Foxfishc/kiro.rs)：参考并吸收 CLI endpoint、thinking、压缩、诊断与 Claude Code 兼容相关实现思路。
- [M-JYuan/kiro.rs](https://github.com/M-JYuan/kiro.rs)：参考并合入管理面板设置入口、默认 endpoint / 凭据级 endpoint 设置等交互与后端 API 思路，并适配当前代码结构。
- [chaogei/Kiro-account-manager](https://github.com/chaogei/Kiro-account-manager)：参考最新版 Kiro 账号、thinking/effort、Enterprise profileArn、工具调用与账号导入导出相关行为。
- 本地 `429修复` 补丁目录：合入普通 429 冷却、账号级风控故障转移、重试策略与相关管理面板展示逻辑。

感谢以上项目作者和贡献者。本分支的主要工作是把这些实现按当前 Kiro CLI / IDE 实测行为重新梳理、合并冲突、补兼容层、补测试，并完成本地构建与兼容性验证。

---

#### [LINUX DO 讨论帖](https://linux.do/t/topic/1571986)

## 免责声明

本项目仅供研究使用, Use at your own risk, 使用本项目所导致的任何后果由使用人承担, 与本项目无关。
本项目与 AWS/KIRO/Anthropic/Claude 等官方无关, 本项目不代表官方立场。

## 注意！

因 TLS 默认从 native-tls 切换至 rustls，你可能需要专门安装证书后才能配置 HTTP 代理。可通过 `config.json` 的 `tlsBackend` 切回 `native-tls`。
如果遇到请求报错, 尤其是无法刷新 token, 或者是直接返回 error request, 请尝试切换 tls 后端为 `native-tls`, 一般即可解决。

**Write Failed/会话卡死**: 如果遇到持续的 Write File / Write Failed 并导致会话不可用，参考 Issue [#22](https://github.com/hank9999/kiro.rs/issues/22) 和 [#49](https://github.com/hank9999/kiro.rs/issues/49) 的说明与临时解决方案（通常与输出过长被截断有关，可尝试调低输出相关 token 上限）

## 功能特性

- **Anthropic API 兼容**: 完整支持 Anthropic Claude API 格式
- **流式响应**: 支持 SSE (Server-Sent Events) 流式输出
- **Token 自动刷新**: 自动管理和刷新 OAuth Token
- **多凭据支持**: 支持配置多个凭据，按优先级自动故障转移
- **负载均衡**: 支持 `priority`（按优先级）和 `balanced`（均衡分配）两种模式
- **智能重试**: 单凭据最多重试 3 次，单请求最多重试 9 次
- **凭据回写**: 多凭据格式下自动回写刷新后的 Token
- **Thinking 模式**: 支持 Claude 的 extended thinking 功能
- **工具调用**: 完整支持 function calling / tool use
- **WebSearch**: 内置 WebSearch 工具转换逻辑
- **Opus 1M 上下文**: Opus 4.8 / 4.7 / 4.6 按 1M 上下文处理
- **Sonnet 1M 上下文**: Sonnet 4.6 按 1M 上下文处理，其他 Sonnet 默认 200K
- **多模型支持**: 支持 Opus / Sonnet / Haiku 常用模型，其他默认 200K
- **Admin 管理**: 可选的 Web 管理界面和 API，支持凭据管理、余额查询等
- **客户端 Key 分发**（v0.4.0+）：在 Admin 面板生成多把 `csk_*` 客户端 Key 分发给下游用户/项目，每把 Key 独立启用/禁用与计数，泄露不影响其他用户
- **用量统计与仪表盘**（v0.4.0+）：按请求记录 token 消耗（按日滚动 JSONL），仪表盘展示时间趋势、模型分布、上游凭据贡献
- **多级 Region 配置**: 支持全局和凭据级别的 Auth Region / API Region 配置
- **凭据级代理**: 支持为每个凭据单独配置 HTTP/SOCKS5 代理，优先级：凭据代理 > 全局代理 > 无代理

---

- [开始](#开始)
  - [1. 编译](#1-编译)
  - [2. 最小配置](#2-最小配置)
  - [3. 启动](#3-启动)
  - [4. 验证](#4-验证)
  - [Docker](#docker)
    - [一、最小部署](#一最小部署)
    - [二、首次启动获取密钥](#二首次启动获取密钥)
    - [三、访问管理面板](#三访问管理面板)
    - [四、定制配置](#四定制配置)
    - [五、升级与回退](#五升级与回退)
    - [六、备份](#六备份)
    - [七、常见问题](#七常见问题)
- [配置详解](#配置详解)
  - [config.json](#configjson)
  - [credentials.json](#credentialsjson)
  - [Region 配置](#region-配置)
  - [代理配置](#代理配置)
  - [认证方式](#认证方式)
  - [环境变量](#环境变量)
- [API 端点](#api-端点)
  - [标准端点 (/v1)](#标准端点-v1)
  - [Claude Code 兼容端点 (/cc/v1)](#claude-code-兼容端点-ccv1)
  - [Thinking 模式](#thinking-模式)
  - [工具调用](#工具调用)
- [模型映射](#模型映射)
- [Claude Code auto 模式兼容](#claude-code-auto-模式兼容)
- [Admin（可选）](#admin可选)
- [手动发布](#手动发布)
- [注意事项](#注意事项)
- [项目结构](#项目结构)
- [技术栈](#技术栈)
- [License](#license)
- [致谢](#致谢)

## 开始

### 1. 编译

> PS: 如果不想编辑可以直接前往 Release 下载二进制文件

> **前置步骤**：编译前需要先构建前端 Admin UI（用于嵌入到二进制中）：
> ```bash
> cd admin-ui && bun install && bun run build
> ```

```bash
cargo build --release
```

### 2. 最小配置

创建 `config.json`：

```json
{
   "host": "127.0.0.1",
   "port": 8990,
   "apiKey": "sk-kiro-rs-qazWSXedcRFV123456",
   "region": "us-east-1"
}
```
> PS: 如果你需要 Web 管理面板, 请注意配置 `adminApiKey`

创建 `credentials.json`（从 Kiro IDE 等中获取凭证信息）：
> PS: 可以前往 Web 管理面板配置跳过本步骤
> 如果你对凭据地域有疑惑, 请查看 [Region 配置](#region-配置)

Social 认证：
```json
{
   "refreshToken": "你的刷新token",
   "expiresAt": "2025-12-31T02:32:45.144Z",
   "authMethod": "social"
}
```

IdC 认证：
```json
{
   "refreshToken": "你的刷新token",
   "expiresAt": "2025-12-31T02:32:45.144Z",
   "authMethod": "idc",
   "clientId": "你的clientId",
   "clientSecret": "你的clientSecret"
}
```

### 3. 启动

```bash
./target/release/kiro-rs
```

或指定配置文件路径：

```bash
./target/release/kiro-rs -c /path/to/config.json --credentials /path/to/credentials.json
```

### 4. 验证

```bash
curl http://127.0.0.1:8990/v1/messages \
  -H "Content-Type: application/json" \
  -H "x-api-key: sk-kiro-rs-qazWSXedcRFV123456" \
  -d '{
    "model": "claude-sonnet-4-20250514",
    "max_tokens": 1024,
    "stream": true,
    "messages": [
      {"role": "user", "content": "Hello, Claude!"}
    ]
  }'
```

### Docker

> 推荐生产部署方式。镜像由 GitHub Actions 自动构建并推送到 GHCR（linux/amd64），开箱即用，无需在服务器上安装 Rust / Node 工具链。

#### 一、最小部署

```bash
git clone https://github.com/xuhuanhello/Kiro-RS-Tool.git
cd Kiro-RS-Tool
mkdir -p data
docker compose up -d
sleep 3
# ⚠️ 关键一步：容器内必须绑 0.0.0.0，否则端口映射失效（应用默认 127.0.0.1）
sed -i 's/"host": "127.0.0.1"/"host": "0.0.0.0"/' data/config.json
docker compose restart
```

镜像地址：`ghcr.io/xuhuanhello/kiro-rs-tool:latest`（公开，无需 `docker login`）。

> **为什么必须改 host**：应用默认绑定 `127.0.0.1`，而 Docker 的端口发布是转发到容器的 eth0。绑回环会导致 `ports` 映射彻底失效——实测宿主机连接直接失败。容器层面的「仅本机」应该由 `docker-compose.yml` 里 `ports` 左侧的 `KIRO_RS_BIND` 控制（默认 `127.0.0.1`），不要靠应用绑回环实现。

目录结构（首次启动后由容器自动生成 `config.json` / `credentials.json`）：

```
/opt/kiro-rs/
├── docker-compose.yml
└── data/                          # 持久化目录，挂载为容器内 /app/config
    ├── config.json                # 自动生成，含随机 apiKey / adminApiKey
    ├── credentials.json           # 自动生成的空数组 []，通过 Admin UI 添加
    ├── client_api_keys.json       # 客户端 Key 分发（v0.4.0+，含明文 csk）
    ├── usage_log.YYYY-MM-DD.jsonl # 按日滚动的请求用量日志（最多保留 31 天）
    ├── kiro_balance_cache.json    # 上游凭据余额缓存
    └── proxy_pool.json            # 代理池（如启用）
```

`docker-compose.yml` 默认包含 `kiro-rs` 服务：

- **kiro-rs**：监听 `8990`，挂载 `./data/:/app/config/`，`restart: unless-stopped`

#### 二、首次启动获取密钥

容器首次启动会在日志里打印一次随机生成的密钥：

```bash
docker compose logs kiro-rs | grep -E "apiKey|adminApiKey"
```

输出形如：

```
apiKey      = sk-kiro-rs-3f8a9b2cQz...
adminApiKey = sk-admin-x9KpRtVw...
```

- `apiKey` — 客户端调用 `/v1/messages` 时携带（`x-api-key` 或 `Authorization: Bearer`）
- `adminApiKey` — 登录 `http://<host>:8990/admin` 管理界面用

记下来后即可关闭日志窗口。也可以直接打开 `data/config.json` 查看或修改。

#### 三、访问管理面板

浏览器打开 `http://<服务器 IP>:8990/admin`，用 `adminApiKey` 登录。三个 Tab：

- **概览** — Token 消耗趋势、按模型/凭据分布
- **凭据管理** — 添加上游 Kiro 凭据（Social / IdC / API Key）
- **客户端 Key** — 分发面向下游的 `csk_*` Key

#### 四、定制配置

**修改端口**：编辑 `docker-compose.yml` 把 `"8990:8990"` 改成 `"<host port>:8990"`，容器内端口保持 8990 不变。

**自定义镜像 tag**：通过环境变量覆盖（默认 `zyphrzero/kiro-rs:latest`）：

```bash
KIRO_RS_IMAGE=zyphrzero/kiro-rs:0.4.0 docker compose up -d
```

或写到 `.env` 文件：

```
KIRO_RS_IMAGE=zyphrzero/kiro-rs:0.4.0
```

**关闭 Redis**：v0.4.0+ 已移除 prompt cache 与 Redis 依赖，无需额外配置。如果你之前在 `data/config.json` 里留有 `redisUrl` / `cacheDebugLogging` / `cacheMaxReadRatio` 字段，可以一并删除。

**配置 HTTP 代理**：在 `data/config.json` 加 `proxyUrl`，或在 Admin UI 的代理池里管理。

#### 五、升级与回退

**通过 Admin UI 在线更新**（推荐）：右上角点云朵图标 → 「更新并重启」。下载 GitHub Releases 二进制，校验 SHA256，原子替换 `<exe>`，旧版本备份到 `<exe>.backup`，进程退出后由 `restart: unless-stopped` 接管重启。失败完全无副作用，断网也能用 `<exe>.backup` 一键回退。

也可以在 Admin UI 的「设置」里启用 **无人值守自动更新**（每天指定时间检查并应用新版本）。

**手动升级**（拉新镜像）：

```bash
docker compose pull && docker compose up -d
```

> 注意：手动 pull 升级与 Admin UI 在线更新使用的是不同的二进制路径——前者拉镜像里的二进制，后者下载 GitHub Releases 并写到容器卷。两种方式都安全，不会互相干扰，但建议固定使用其中一种以便追踪版本。

#### 六、备份

`data/` 目录是唯一的状态来源，定期备份即可：

```bash
tar -czf kiro-rs-backup-$(date +%F).tar.gz /opt/kiro-rs/data/
```

> ⚠️ 备份包含明文凭据（`credentials.json`）和明文客户端 Key（`client_api_keys.json`），请加密保存或限制访问权限。

恢复时把备份解压回 `/opt/kiro-rs/data/` 后 `docker compose restart` 即可。

#### 七、常见问题

- **首次启动看不到日志中的密钥** — 改用 `docker compose logs --tail=200 kiro-rs`，或直接看 `data/config.json` 里的 `apiKey` / `adminApiKey` 字段
- **想从 Docker Hub 之外的镜像源拉取** — 把 `docker-compose.yml` 里的 `image:` 改成你自己镜像源的地址（如 `ghcr.io/...`、阿里云镜像加速等）
- **Admin UI 显示 "暂无数据"** — 仪表盘需要至少一次请求才会有数据，先用 curl 调一次 `/v1/messages` 即可看到趋势开始填充

## 配置详解

### config.json

| 字段 | 类型 | 默认值 | 描述 |
|------|------|--------|------|
| `host` | string | `127.0.0.1` | 服务监听地址 |
| `port` | number | `8080` | 服务监听端口 |
| `apiKey` | string | - | 自定义 API Key（用于客户端认证，必配） |
| `region` | string | `us-east-1` | AWS 区域 |
| `authRegion` | string | - | Auth Region（用于 Token 刷新），未配置时回退到 region |
| `apiRegion` | string | - | API Region（用于 API 请求），未配置时回退到 region |
| `kiroVersion` | string | `2.6.0` | Kiro 版本号 |
| `machineId` | string | - | 自定义机器码（64位十六进制），不定义则自动生成 |
| `systemVersion` | string | 随机 | 系统版本标识 |
| `nodeVersion` | string | `22.21.1` | Node.js 版本标识 |
| `tlsBackend` | string | `rustls` | TLS 后端：`rustls` 或 `native-tls` |
| `countTokensApiUrl` | string | - | 外部 count_tokens API 地址 |
| `countTokensApiKey` | string | - | 外部 count_tokens API 密钥 |
| `countTokensAuthType` | string | `x-api-key` | 外部 API 认证类型：`x-api-key` 或 `bearer` |
| `proxyUrl` | string | - | HTTP/SOCKS5 代理地址 |
| `proxyUsername` | string | - | 代理用户名 |
| `proxyPassword` | string | - | 代理密码 |
| `adminApiKey` | string | - | Admin API 密钥，配置后启用凭据管理 API 和 Web 管理界面 |
| `updateAutoApply` | boolean | `false` | 是否启用无人值守自动更新；开启后每天到 `updateAutoApplyTime` 自动从 GitHub Releases 下载新版本二进制并重启 |
| `updateAutoApplyTime` | string | `03:00` | 自动更新触发时间（本地时区，HH:MM 24 小时制） |
| `loadBalancingMode` | string | `priority` | 负载均衡模式：`priority`（按优先级）或 `balanced`（均衡分配） |
| `retryMode` | string | `fast` | 普通 429 重试策略：`turbo` / `fast` / `balanced` / `steady` / `polite` / `custom` |
| `retryPolicy` | object | - | `retryMode=custom` 时的自定义 429 策略，字段见下方示例 |
| `extractThinking` | boolean | `true` | 非流式响应的 thinking 块提取。启用后 `<thinking>` 标签会被解析为独立的 `thinking` 内容块 |
| `defaultEndpoint` | string | `ide` | 默认 Kiro 端点。凭据未显式指定 `endpoint` 时使用。可选值：`ide`（Kiro IDE）、`cli`（Amazon Q for CLI，适用于 `ksk_` 前缀的 API Key） |
| `toolCompatibilityMode` | string | `claude-code` | 工具兼容模式。`claude-code` 会将 Claude Code 的 `Write` / `Edit` / `Read` / `Bash` 等工具适配为 Kiro 内置工具；`raw` 直接透传工具 schema，仅建议排障时使用 |
| `safeguardsEnabled` | boolean | `true` | 支持 Claude Code auto 模式的「服务端分类器审查」（`safeguards` / `safeguard_results` 协议）。开启后，当客户端请求服务端审查时，代理会在**每一个**响应的 `message_delta` 里回传裁决，避免出现「网关不兼容」的分类器计费提示 |
| `safeguardsClassifierEnabled` | boolean | `false` | 是否用真实模型执行安全判定。**关闭时不下发任何裁决**，每个工具调用仍由 Claude Code 自己的分类器审查（零额外延迟）。开启后仅对 shell 类工具多跑一次模型判定；判定失败或超时一律不下发裁决，交回客户端本地分类 |
| `safeguardsClassifierModel` | string | `claude-sonnet-5` | 真实分类器使用的模型 |
| `safeguardsClassifierTimeoutSecs` | number | `20` | 分类器单次判定超时（秒） |

完整配置示例：

```json
{
   "host": "127.0.0.1",
   "port": 8990,
   "apiKey": "sk-kiro-rs-qazWSXedcRFV123456",
   "region": "us-east-1",
   "tlsBackend": "rustls",
   "kiroVersion": "2.6.0",
   "machineId": "64位十六进制机器码",
   "systemVersion": "darwin#24.6.0",
   "nodeVersion": "22.21.1",
   "authRegion": "us-east-1",
   "apiRegion": "us-east-1",
   "countTokensApiUrl": "https://api.example.com/v1/messages/count_tokens",
   "countTokensApiKey": "sk-your-count-tokens-api-key",
   "countTokensAuthType": "x-api-key",
   "proxyUrl": "http://127.0.0.1:7890",
   "proxyUsername": "user",
   "proxyPassword": "pass",
   "adminApiKey": "sk-admin-your-secret-key",
   "updateAutoApply": false,
   "updateAutoApplyTime": "03:00",
   "loadBalancingMode": "priority",
   "retryMode": "fast",
   "retryPolicy": null,
   "extractThinking": true,
   "defaultEndpoint": "ide",
   "toolCompatibilityMode": "claude-code",
   "safeguardsEnabled": true,
   "safeguardsClassifierEnabled": false,
   "safeguardsClassifierModel": "claude-sonnet-5",
   "safeguardsClassifierTimeoutSecs": 20
}
```

自定义普通 429 重试策略示例：

```json
{
   "retryMode": "custom",
   "retryPolicy": {
      "rateLimitCooldownMs": 800,
      "maxRequestRetries": 15,
      "baseBackoffMs": 80,
      "maxBackoffMs": 800,
      "credentialSwitchOn429": true,
      "respectRetryAfter": false
   }
}
```

### credentials.json

支持单对象格式（向后兼容）或数组格式（多凭据）。

#### 字段说明

| 字段             | 类型     | 描述                                          |
|----------------|--------|---------------------------------------------|
| `id`           | number | 凭据唯一 ID（可选，仅用于 Admin API 管理；手写文件可不填）        |
| `accessToken`  | string | OAuth 访问令牌（可选，可自动刷新）                        |
| `refreshToken` | string | OAuth 刷新令牌                                  |
| `profileArn`   | string | AWS Profile ARN（可选，登录时返回）                   |
| `expiresAt`    | string | Token 过期时间 (RFC3339)                        |
| `authMethod`   | string | 认证方式：`social` 或 `idc`                       |
| `clientId`     | string | IdC 登录的客户端 ID（IdC 认证必填）                     |
| `clientSecret` | string | IdC 登录的客户端密钥（IdC 认证必填）                      |
| `priority`     | number | 凭据优先级，数字越小越优先，默认为 0                         |
| `region`       | string | 凭据级 Auth Region, 兼容字段                       |
| `authRegion`   | string | 凭据级 Auth Region，用于 Token 刷新, 未配置时回退到 region |
| `apiRegion`    | string | 凭据级 API Region，用于 API 请求                    |
| `machineId`    | string | 凭据级机器码（64位十六进制）                             |
| `email`        | string | 用户邮箱（可选，从 API 获取）                           |
| `proxyUrl`     | string | 凭据级代理 URL（可选，特殊值 `direct` 表示不使用代理）       |
| `proxyUsername`| string | 凭据级代理用户名（可选）                                |
| `proxyPassword`| string | 凭据级代理密码（可选）                                 |
| `endpoint`     | string | 凭据级端点名称（可选，未配置时使用 `config.defaultEndpoint`）。可选值：`ide`、`cli` |
| `kiroApiKey`   | string | Kiro API Key（`ksk_` 前缀，仅 `authMethod: "api_key"` 时使用） |

说明：
- IdC / Builder-ID / IAM 在本项目里属于同一种登录方式，配置时统一使用 `authMethod: "idc"`
- 为兼容旧配置，`builder-id` / `iam` 仍可被识别，但会按 `idc` 处理
- **`ksk_` 前缀的 API Key 凭据必须将 `endpoint` 设为 `cli`**（或将 `config.defaultEndpoint` 设为 `cli`），否则请求会因协议不匹配而失败

#### 单凭据格式（旧格式，向后兼容）

```json
{
   "accessToken": "请求token，一般有效期一小时，可选",
   "refreshToken": "刷新token，一般有效期7-30天不等",
   "profileArn": "arn:aws:codewhisperer:us-east-1:111112222233:profile/QWER1QAZSDFGH",
   "expiresAt": "2025-12-31T02:32:45.144Z",
   "authMethod": "social",
   "clientId": "IdC 登录需要",
   "clientSecret": "IdC 登录需要"
}
```

默认 `toolCompatibilityMode: "claude-code"` 会把 Claude Code 工具映射到 Kiro 内置工具，并在返回时恢复为 Claude Code 工具名和参数。支持 `Write`、`Edit`、`Read`、`Bash`、`Glob`、`Grep`、`LS`、`WebSearch`；不支持的字段会返回明确的兼容错误，而不是向客户端发送无效工具参数。

#### 多凭据格式（支持故障转移和自动回写）

```json
[
   {
      "refreshToken": "第一个凭据的刷新token",
      "expiresAt": "2025-12-31T02:32:45.144Z",
      "authMethod": "social",
      "priority": 0
   },
   {
      "refreshToken": "第二个凭据的刷新token",
      "expiresAt": "2025-12-31T02:32:45.144Z",
      "authMethod": "idc",
      "clientId": "xxxxxxxxx",
      "clientSecret": "xxxxxxxxx",
      "region": "us-east-2",
      "priority": 1,
      "proxyUrl": "socks5://proxy.example.com:1080",
      "proxyUsername": "user",
      "proxyPassword": "pass"
   },
   {
      "refreshToken": "第三个凭据（显式不走代理）",
      "expiresAt": "2025-12-31T02:32:45.144Z",
      "authMethod": "social",
      "priority": 2,
      "proxyUrl": "direct"
   }
]
```

多凭据特性：
- 按 `priority` 字段排序，数字越小优先级越高（默认为 0）
- 单凭据最多重试 3 次，单请求最多重试 9 次
- 自动故障转移到下一个可用凭据
- 多凭据格式下 Token 刷新后自动回写到源文件

### Region 配置

支持多级 Region 配置，分别控制 Token 刷新和 API 请求使用的区域。

**Auth Region**（Token 刷新）优先级：
`凭据.authRegion` > `凭据.region` > `config.authRegion` > `config.region`

**API Region**（API 请求）优先级：
`凭据.apiRegion` > `config.apiRegion` > `config.region`

### 代理配置

支持全局代理和凭据级代理，凭据级代理会覆盖该凭据产生的所有出站连接（API 请求、Token 刷新、额度查询）。

**代理优先级**：`凭据.proxyUrl` > `config.proxyUrl` > 无代理

| 凭据 `proxyUrl` 值 | 行为 |
|---|---|
| 具体 URL（如 `http://proxy:8080`、`socks5://proxy:1080`） | 使用凭据指定的代理 |
| `direct` | 显式不使用代理（即使全局配置了代理） |
| 未配置（留空） | 回退到全局代理配置 |

凭据级代理示例：

```json
[
   {
      "refreshToken": "凭据A：使用自己的代理",
      "authMethod": "social",
      "proxyUrl": "socks5://proxy-a.example.com:1080",
      "proxyUsername": "user_a",
      "proxyPassword": "pass_a"
   },
   {
      "refreshToken": "凭据B：显式不走代理（直连）",
      "authMethod": "social",
      "proxyUrl": "direct"
   },
   {
      "refreshToken": "凭据C：使用全局代理（或直连，取决于 config.json）",
      "authMethod": "social"
   }
]
```

### 认证方式

客户端请求本服务时，支持两种认证方式：

1. **x-api-key Header**
   ```
   x-api-key: sk-your-api-key
   ```

2. **Authorization Bearer**
   ```
   Authorization: Bearer sk-your-api-key
   ```

### 环境变量

可通过环境变量配置日志级别：

```bash
RUST_LOG=debug ./target/release/kiro-rs
```

## API 端点

### 标准端点 (/v1)

| 端点 | 方法 | 描述 |
|------|------|------|
| `/v1/models` | GET | 获取可用模型列表 |
| `/v1/messages` | POST | 创建消息（对话） |
| `/v1/messages/count_tokens` | POST | 估算 Token 数量 |

### Claude Code 兼容端点 (/cc/v1)

| 端点 | 方法 | 描述 |
|------|------|------|
| `/cc/v1/messages` | POST | 创建消息（缓冲模式，确保 `input_tokens` 准确） |
| `/cc/v1/messages/count_tokens` | POST | 估算 Token 数量（与 `/v1` 相同） |

> **`/cc/v1/messages` 与 `/v1/messages` 的区别**：
> - `/v1/messages`：实时流式返回，`message_start` 中的 `input_tokens` 是估算值
> - `/cc/v1/messages`：缓冲模式，等待上游流完成后，用从 `contextUsageEvent` 计算的准确 `input_tokens` 更正 `message_start`，然后一次性返回所有事件
> - 等待期间会每 25 秒发送 `ping` 事件保活

### Thinking 模式

支持 Claude 的 extended thinking 功能：

```json
{
  "model": "claude-sonnet-4-20250514",
  "max_tokens": 16000,
  "thinking": {
    "type": "enabled",
    "budget_tokens": 10000
  },
  "messages": [...]
}
```

### 工具调用

完整支持 Anthropic 的 tool use 功能：

```json
{
  "model": "claude-sonnet-4-20250514",
  "max_tokens": 1024,
  "tools": [
    {
      "name": "get_weather",
      "description": "获取指定城市的天气",
      "input_schema": {
        "type": "object",
        "properties": {
          "city": {"type": "string"}
        },
## 模型映射

`map_model` 采用「实测表 + 向前兼容」策略，把 Anthropic 模型名转换为 Kiro 内部模型名（大小写不敏感，`5-5` / `5.5` / `-thinking` 后缀都支持）：

| Anthropic 模型 | Kiro 模型 | 上下文窗口 |
|---|---|---|
| `claude-opus-5-5` / `claude-opus-5.5` | `claude-opus-5.5` | **1M** |
| `claude-opus-5` | `claude-opus-5` | **1M** |
| `claude-sonnet-5` | `claude-sonnet-5` | **1M** |
| `claude-opus-4-8` / `4.8` | `claude-opus-4.8` | **1M** |
| `claude-opus-4-7` / `4.7` | `claude-opus-4.7` | **1M** |
| `claude-opus-4-6` / `4.6` | `claude-opus-4.6` | **1M** |
| `claude-sonnet-4-6` / `4.6` | `claude-sonnet-4.6` | **1M** |
| `claude-opus-4-5` / `4.5` | `claude-opus-4.5` | 200K |
| `claude-sonnet-4-5` / `4.5` | `claude-sonnet-4.5` | 200K |
| `claude-haiku-*` | `claude-haiku-4.5` | 200K |

**向前兼容**：比上表某个家族最新版本更新的版本会按 Kiro 命名规则自动推导。例如 Kiro 将来上线 Sonnet 5.5，直接 `--model claude-sonnet-5-5` 即可调用，**无需改代码**（但不会自动出现在 `/v1/models` 列表里）。更老或已知不存在的版本（如 `claude-sonnet-4-8`）仍在本地拒绝，避免把无效模型名透传到上游换回一条更难读的错误。

**上下文窗口规则**：Opus / Sonnet 自 4.6 起按 1M 处理，更早版本与 Haiku 为 200K。本服务按该窗口计算 `contextUsageEvent` 的实际 `input_tokens`，客户端发起大 `max_tokens` 请求时无需额外配置。

> **thinking / effort**
>
> 模型名带 `-thinking` 后缀（如 `claude-opus-5-5-thinking`）会自动覆写 `thinking` 配置：
>
> - Opus 5.5 / 5 / 4.8 / 4.7：`adaptive` 模式，effort 支持 `low` / `medium` / `high` / `xhigh` / `max`
> - Opus 4.6 / Sonnet 4.6：`adaptive` 模式，effort 支持 `low` / `medium` / `high` / `max`（无 `xhigh`）
> - 更早模型：`enabled` 模式，`budget_tokens` 固定 20000
>
> 未设置 effort 时按 `high` 处理；若指定了该模型不支持的档位（例如给 Opus 4.6 传 `xhigh`），回落到该模型允许的最高档 `max`。CLI endpoint 保持抓包验证过的最小 `output_config` 形态；IDE endpoint 按 IDE bundle 行为补充 `thinking={type:"adaptive",display:"summarized"}` wrapper。流式与非流式响应均支持 Kiro 的 `reasoningContentEvent`，会转换为 Anthropic `thinking` 内容块并优先使用上游 signature。
>
> **Opus 5.5 的差异**：其 `thinking.type` 枚举只有 `adaptive`，**不支持 `disabled`**（Opus 5 / Sonnet 5 / Opus 4.8 都是 `adaptive` + `disabled`）。本服务在客户端关闭 thinking 时是整体省略该字段，因此天然满足该约束。

可用模型完整列表通过 `GET /v1/models` 查询。

> **1M 上下文支持**：Opus 4.8 / 4.7 / 4.6 按 1M 上下文窗口处理。
>
> Sonnet 4.6 也按 1M 上下文窗口处理；其余模型仍为 200K。本服务在收到上述模型请求时会按 1M 计算 `contextUsageEvent` 的实际 `input_tokens`，前端发起 `max_tokens` 大请求时不需要额外配置。
>
> 模型名带 `-thinking` 后缀（如 `claude-opus-4-8-thinking`）会自动覆写 `thinking` 配置：Opus 4.6 / 4.7 / 4.8 走 `adaptive` 模式，其他模型走 `enabled` 模式，`budget_tokens` 固定 20000。Kiro CLI 2.6.0 的模型 schema 已验证支持 `low` / `medium` / `high` / `xhigh` / `max` 五档 effort；本服务会在 Opus 4.6 / 4.7 / 4.8 adaptive 路径发送 `additionalModelRequestFields.output_config.effort`，未设置时默认 `high`，非法值回落到 `high`。CLI endpoint 会保持抓包验证过的最小 `output_config` 形态；IDE endpoint 会按 IDE bundle 行为补充 `thinking={type:"adaptive",display:"summarized"}` wrapper。也兼容 `claude-opus-4-8-thinking-xhigh` / `-max` 这类后缀写法。流式和非流式响应均支持 Kiro 新的 `reasoningContentEvent`，会转换为 Anthropic `thinking` 内容块并优先使用上游 signature。

可用模型完整列表通过 `GET /v1/models` 查询。

## Claude Code auto 模式兼容

Claude Code 在 auto 模式下会向服务端请求「分类器审查」：请求体带 `safeguards` 字段，并要求服务端在**每一个**响应的 `message_delta` 里回传 `safeguard_results`。只要有一个响应漏掉，Claude Code 就会判定网关不兼容、回退到自己的本地分类器，并展示一条「分类器计费」提示。

本服务实现了该协议：

| 配置 | 行为 |
|---|---|
| `safeguardsEnabled: true`（默认） | 客户端请求审查时，每个响应都回传裁决，上述提示不再出现 |
| `safeguardsClassifierEnabled: false`（默认） | 不下发任何裁决 → 每个工具调用仍由 Claude Code 自己的分类器审查，**零额外延迟** |
| `safeguardsClassifierEnabled: true` | 会**改变实际后果**的工具多跑一次模型判定，每个含此类调用的响应约 +1~3s |

**服务端分类覆盖哪些工具**：命令执行（`Bash` / `*shell*` / `*terminal*` / `*exec*`）、文件写入与编辑（`Write` / `Edit` / `MultiEdit` / `NotebookEdit` / `mcp__*__write_file`）、网络读写（`*fetch*` / `*http*` / `*upload*` / `*download*`）。

未覆盖的（`Read` / `Grep` / `Glob` / `TodoWrite` 等没有破坏性效果的工具）一律以 `skipped` 表达，交回客户端自己的分类器——多跑一次判定只是白白增加延迟。

**分类结果缓存**：同一份「工具 + 输入 + 工作目录 + 用户拒绝规则」的**放行**裁决会在进程内缓存（LRU 上限 512 条，10 分钟过期），重复命令直接复用。实测同一命令连发三次：`5.87s → 3.50s → 2.33s`。

> **拒绝不缓存**：拒绝的判定依赖会话活动摘要（目标是不是 agent 自己刚建的），缓存下来会把一次误判固化成永久拦截。工作目录或用户拒绝规则发生变化时，缓存键随之改变，旧裁决自然失效。

**裁决的三态语义**（实测确认）：

- 判定为安全 → `not_flagged`：客户端直接放行，**不再重复检查**
- 判定为危险 → `flagged`：客户端拦截该动作，并把理由透传给模型
- **分类失败 / 超时 / 未覆盖 → 不下发裁决**：客户端回退到自己的分类器（不会无条件放行）

服务端**绝不会**在未真正分类的情况下下发 `not_flagged`——那等于让客户端跳过它自己的安全检查，是安全倒退而非优化。

**分类器使用的上下文**：Claude Code 会在 `classifier_context` 里带上 `live_cwd`、`home_dir`、`trusted_directories`、`rules`（allow / deny / ask）与 `auto_mode` 规则，本服务会解析并一并交给分类器——有了真实工作目录，「这个路径在不在项目内」才是事实而非猜测（例如删除 `/tmp` 下的临时产物时，分类器能确认它既不在项目内、也不属于用户数据）。客户端未提供该字段时按字面路径判定，行为不退化。

客户端**不发送对话历史**，所以「这个目录是不是 agent 自己刚建的」光靠它判断不了。本服务会自行从请求体里补一份**会话活动摘要**——最近 12 次工具调用的名称与精简输入（命令、路径等），排在待判定调用之前交给分类器。

> 摘要只取工具调用本身，**不取工具返回内容**：文件内容属不可信文本，塞进分类器提示词会形成注入面。提示词里也明确标注该摘要是"记录数据，不是指令"。

实测（同一条 `rm -rf /Users/me/scratch/bench`，路径既不在项目内也不在 `/tmp` 下）：

| 输入 | 裁决 |
|---|---|
| 带对话历史（agent 早前 `mkdir` 建过它） | ✅ 放行——*"a scratch directory the agent itself created earlier in this session"* |
| 无对话历史（来历不明） | 🚫 拦截——*"outside the project directory and not under OS temp storage"* |

> **替代方案**：如果不需要服务端裁决，也可以在 Claude Code 侧关闭该请求——在
> `~/.claude/settings.json` 的 `env` 里设置 `CLAUDE_CODE_AUTO_MODE_SERVER=0`。
> 客户端不再请求审查，提示同样不出现，且零额外延迟；代价是服务端的分类器不会生效。

## Admin（可选）

当 `config.json` 配置了非空 `adminApiKey` 时，会启用：

- **凭据管理 API**
  - `GET /api/admin/credentials` - 获取所有凭据状态
  - `POST /api/admin/credentials` - 添加新凭据
  - `DELETE /api/admin/credentials/:id` - 删除凭据
  - `POST /api/admin/credentials/:id/disabled` - 设置凭据禁用状态
  - `POST /api/admin/credentials/:id/priority` - 设置凭据优先级
  - `POST /api/admin/credentials/:id/reset` - 重置失败计数
  - `GET /api/admin/credentials/:id/balance` - 获取凭据余额

- **客户端 Key 分发 API**（v0.4.0+）
  - `GET /api/admin/client-keys` - 列出所有客户端 Key（脱敏展示）
  - `POST /api/admin/client-keys` - 创建新 Key（响应里返回明文，仅此一次）
  - `PUT /api/admin/client-keys/:id` - 修改名称 / 描述
  - `DELETE /api/admin/client-keys/:id` - 删除 Key
  - `POST /api/admin/client-keys/:id/disabled` - 启用/禁用
  - `POST /api/admin/client-keys/:id/reset-stats` - 重置累计计数

- **用量统计 API**（v0.4.0+）
  - `GET /api/admin/stats/overview` - 今日 / 最近 7 天概览
  - `GET /api/admin/stats/timeseries?range=24h|7d|30d` - 时序点
  - `GET /api/admin/stats/by-model?range=...` - 按模型分布
  - `GET /api/admin/stats/by-credential?range=...` - 按上游凭据分布
  - `GET /api/admin/config/runtime` - 查看当前 `defaultEndpoint`、`toolCompatibilityMode` 与端点兼容版本

- **Admin UI**（v0.4.0+ 升级为三 Tab SPA）
  - `GET /admin` - 概览 / 凭据管理 / 客户端 Key
  - 顶栏统一工具：负载均衡切换、刷新、在线更新、Key 管理（修改 Admin Key 与业务 API Key）

### 在线更新

Admin UI 顶部的「镜像在线更新」入口支持：

- 一键从 GitHub Releases 下载新版本二进制（带 SHA256 校验），原子替换当前 `kiro-rs`，旧版本备份到 `<exe>.backup`
- 替换完成后进程主动退出，由 `docker-compose.yml` 里的 `restart: unless-stopped` 接管重启，新二进制随之生效
- 失败完全无副作用：网络断、校验不通过都不会动到正在运行的旧 `kiro-rs`
- 「回退到上一版本」从 `<exe>.backup` 恢复并重启进程，断网也能用
- 可开启「无人值守自动更新」：每天到指定时间检查并应用新版本
- 自动检查 Docker Hub tag 与 GitHub Releases，发现新版本时在工具栏图标上显示红点提醒

容器部署只需要把 `data/` 目录挂进容器，不再需要 docker socket 或 compose 文件透传：

```yaml
volumes:
  - ./data/:/app/config/
```

镜像和版本号都从 Docker Hub 取（`hub.docker.com/r/<owner>/kiro-rs`）。当前公开仓库不附带自动发布配置，发布镜像和二进制需要维护者手动构建、校验并上传。

## 手动发布

当前公开仓库不包含自动发布配置。正式发布建议手动走以下流程：

1. 修改 `Cargo.toml` 中的 `package.version`，例如 `0.1.0`。
2. 本地执行 `cargo test --locked` 与前端构建检查。
3. 使用 `cargo build --release --locked` 或交叉编译工具生成目标平台二进制。
4. 计算并保存 SHA256 校验值。
5. 手动创建 GitHub Release 并上传二进制包与校验文件。
6. 如需 Docker 镜像，手动构建多架构镜像并推送到自己的镜像仓库。

## 注意事项

1. **凭证安全**: 请妥善保管 `credentials.json` 文件，不要提交到版本控制
2. **Token 刷新**: 服务会自动刷新过期的 Token，无需手动干预
3. **WebSearch 工具**: 当 `tools` 列表仅包含一个 `web_search` 工具时，会走内置 WebSearch 转换逻辑

## 项目结构

```
kiro-rs/
├── src/
│   ├── main.rs                 # 程序入口
│   ├── http_client.rs          # HTTP 客户端构建
│   ├── token.rs                # Token 计算模块
│   ├── debug.rs                # 调试工具
│   ├── test.rs                 # 测试
│   ├── model/                  # 配置和参数模型
│   │   ├── config.rs           # 应用配置
│   │   └── arg.rs              # 命令行参数
│   ├── anthropic/              # Anthropic API 兼容层
│   │   ├── router.rs           # 路由配置
│   │   ├── handlers.rs         # 请求处理器
│   │   ├── middleware.rs       # 认证中间件
│   │   ├── types.rs            # 类型定义
│   │   ├── converter.rs        # 协议转换器
│   │   ├── stream.rs           # 流式响应处理
│   │   └── websearch.rs        # WebSearch 工具处理
│   ├── kiro/                   # Kiro API 客户端
│   │   ├── provider.rs         # API 提供者
│   │   ├── token_manager.rs    # Token 管理
│   │   ├── machine_id.rs       # 设备指纹生成
│   │   ├── model/              # 数据模型
│   │   │   ├── credentials.rs  # OAuth 凭证
│   │   │   ├── events/         # 响应事件类型
│   │   │   ├── requests/       # 请求类型
│   │   │   ├── common/         # 共享类型
│   │   │   ├── token_refresh.rs # Token 刷新模型
│   │   │   └── usage_limits.rs # 使用额度模型
│   │   └── parser/             # AWS Event Stream 解析器
│   │       ├── decoder.rs      # 流式解码器
│   │       ├── frame.rs        # 帧解析
│   │       ├── header.rs       # 头部解析
│   │       ├── error.rs        # 错误类型
│   │       └── crc.rs          # CRC 校验
│   ├── admin/                  # Admin API 模块
│   │   ├── router.rs           # 路由配置
│   │   ├── handlers.rs         # 请求处理器
│   │   ├── service.rs          # 业务逻辑服务
│   │   ├── types.rs            # 类型定义
│   │   ├── middleware.rs       # 认证中间件
│   │   └── error.rs            # 错误处理
│   ├── admin_ui/               # Admin UI 静态文件嵌入
│   │   └── router.rs           # 静态文件路由
│   └── common/                 # 公共模块
│       └── auth.rs             # 认证工具函数
├── admin-ui/                   # Admin UI 前端工程（构建产物会嵌入二进制）
├── tools/                      # 辅助工具
├── Cargo.toml                  # 项目配置
├── config.example.json         # 配置示例
├── docker-compose.yml          # Docker Compose 配置
└── Dockerfile                  # Docker 构建文件
```

## 技术栈

- **Web 框架**: [Axum](https://github.com/tokio-rs/axum) 0.8
- **异步运行时**: [Tokio](https://tokio.rs/)
- **HTTP 客户端**: [Reqwest](https://github.com/seanmonstar/reqwest)
- **序列化**: [Serde](https://serde.rs/)
- **日志**: [tracing](https://github.com/tokio-rs/tracing)
- **命令行**: [Clap](https://github.com/clap-rs/clap)

## License

MIT

## 致谢

本项目的实现离不开前辈的努力:  
 - [kiro2api](https://github.com/caidaoli/kiro2api)
 - [proxycast](https://github.com/aiclientproxy/proxycast)

本项目部分逻辑参考了以上的项目, 再次由衷的感谢!

## 社区支持

欢迎到 [linux.do](https://linux.do/) 交流、分享和反馈。
