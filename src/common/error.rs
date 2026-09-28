//! 错误格式化工具

/// 把错误及其 source 链拼成一行，便于定位根因
///
/// reqwest::Error 的 Display 只有 "error sending request for url (...)"，
/// 真正的原因（DNS 解析失败 / 连接被重置 / TLS 握手失败 / 证书不受信）挂在
/// source 链上。只打印 {} 会把它丢掉，排查时只能靠猜。
///
/// 链长上限 6 层，避免异常情况下无限展开。
pub fn chain(err: &(dyn std::error::Error + 'static)) -> String {
    const MAX_DEPTH: usize = 6;

    let mut out = err.to_string();
    let mut source = err.source();
    let mut depth = 0;
    while let Some(e) = source {
        out.push_str(" <- ");
        out.push_str(&e.to_string());
        depth += 1;
        if depth >= MAX_DEPTH {
            out.push_str(" <- ...");
            break;
        }
        source = e.source();
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt;

    #[derive(Debug)]
    struct Layer {
        msg: &'static str,
        inner: Option<Box<Layer>>,
    }

    impl fmt::Display for Layer {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(self.msg)
        }
    }

    impl std::error::Error for Layer {
        fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
            self.inner.as_ref().map(|b| b.as_ref() as &(dyn std::error::Error + 'static))
        }
    }

    fn layer(msg: &'static str, inner: Option<Layer>) -> Layer {
        Layer {
            msg,
            inner: inner.map(Box::new),
        }
    }

    #[test]
    fn single_error_has_no_arrow() {
        let e = layer("boom", None);
        assert_eq!(chain(&e), "boom");
    }

    #[test]
    fn nested_errors_are_joined_in_order() {
        // 模拟 reqwest 的形态：顶层是笼统的说明，根因在最里层
        let e = layer(
            "error sending request for url (https://example.com)",
            Some(layer(
                "client error (Connect)",
                Some(layer("dns error", Some(layer("failed to lookup address", None)))),
            )),
        );
        let out = chain(&e);
        assert_eq!(
            out,
            "error sending request for url (https://example.com) <- client error (Connect) \
             <- dns error <- failed to lookup address"
        );
        assert!(out.contains("failed to lookup address"), "根因必须出现在链尾");
    }

    #[test]
    fn depth_is_capped() {
        let mut e = layer("l0", None);
        for i in 1..12 {
            let msg: &'static str = Box::leak(format!("l{i}").into_boxed_str());
            e = layer(msg, Some(e));
        }
        let out = chain(&e);
        assert!(out.ends_with("<- ..."), "超过上限应截断：{out}");
        // 顶层 + 最多 6 层 source + 一个截断标记
        assert!(
            out.matches(" <- ").count() <= 7,
            "链长必须有界，实际: {out}"
        );
    }
}
