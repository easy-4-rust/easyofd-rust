//! OFD 命名空间修改器。
//!
//! 对应 Java: org.ofdrw.pkg.tool.OFDNameSpaceModifier

/// OFD 命名空间修改器。
///
/// 用于修改 OFD XML 文件中的命名空间声明。
///
/// 对应 Java: org.ofdrw.pkg.tool.OFDNameSpaceModifier
#[derive(Debug, Clone)]
pub struct OfdNameSpaceModifier {
    /// 目标命名空间 URI。
    target_namespace: String,
}

impl OfdNameSpaceModifier {
    /// 创建命名空间修改器。
    pub fn new(target_namespace: impl Into<String>) -> Self {
        Self {
            target_namespace: target_namespace.into(),
        }
    }

    /// 获取目标命名空间。
    #[must_use]
    pub fn target_namespace(&self) -> &str {
        &self.target_namespace
    }

    /// 修改 XML 内容中的命名空间。
    #[must_use]
    pub fn modify(&self, xml_content: &str) -> String {
        // 简单替换 OFD 命名空间
        xml_content.replace("http://www.ofdspec.org/2016", &self.target_namespace)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ofd_namespace_modifier_new() {
        let modifier = OfdNameSpaceModifier::new("http://example.com/ns");
        assert_eq!(modifier.target_namespace(), "http://example.com/ns");
    }

    #[test]
    fn ofd_namespace_modifier_modify() {
        let modifier = OfdNameSpaceModifier::new("http://custom.ns");
        let input = r#"<ofd:OFD xmlns:ofd="http://www.ofdspec.org/2016">"#;
        let result = modifier.modify(input);
        assert!(result.contains("http://custom.ns"));
        assert!(!result.contains("ofdspec.org"));
    }

    #[test]
    fn ofd_namespace_modifier_no_match() {
        let modifier = OfdNameSpaceModifier::new("http://custom.ns");
        let input = r#"<ofd:OFD xmlns:ofd="http://other.ns">"#;
        let result = modifier.modify(input);
        assert_eq!(result, input);
    }
}
