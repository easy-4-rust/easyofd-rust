//! 文档内容替换。
//!
//! 对应 Java: org.ofdrw.layout.DocContentReplace

/// 文档内容替换，用于在渲染前替换文档中的占位内容。
///
/// 对应 Java: ofdrw layout DocContentReplace。
#[derive(Debug, Clone, PartialEq)]
pub struct DocContentReplace {
    /// 占位符标识（如 `"{{name}}"`）。
    pub placeholder: String,
    /// 替换内容。
    pub replacement: String,
}

impl DocContentReplace {
    /// 创建文档内容替换。
    #[must_use]
    pub fn new(placeholder: impl Into<String>, replacement: impl Into<String>) -> Self {
        Self {
            placeholder: placeholder.into(),
            replacement: replacement.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let r = DocContentReplace::new("{{name}}", "张三");
        assert_eq!(r.placeholder, "{{name}}");
        assert_eq!(r.replacement, "张三");
    }

    #[test]
    fn test_clone_eq() {
        let a = DocContentReplace::new("{{id}}", "12345");
        let b = a.clone();
        assert_eq!(a, b);
    }
}
