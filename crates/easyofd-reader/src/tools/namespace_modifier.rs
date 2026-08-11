//! 命名空间变更工具。
//!
//! 对应 Java: org.ofdrw.reader.tools.NameSpaceModifier
//!
//! 已废弃：Java 原始类标记为 `@Deprecated`。
//! Rust 版提供 OFD XML 命名空间标准化功能。

/// OFD 标准命名空间 URI。
pub const OFD_NAMESPACE: &str = "http://www.ofdspec.org/2016";

/// OFD 命名空间前缀。
pub const OFD_PREFIX: &str = "ofd";

/// 命名空间变更器。
///
/// 对应 Java: `org.ofdrw.reader.tools.NameSpaceModifier`
///
/// **废弃**：Java 原始类标记为 `@Deprecated`，
/// 建议使用 `easyofd_package` 中的命名空间处理工具。
///
/// 将 XML 内容中的非标准命名空间替换为标准 OFD 命名空间。
#[derive(Debug, Clone)]
#[deprecated(since = "1.0.0", note = "使用 easyofd_package 中的命名空间工具替代")]
pub struct NamespaceModifier {
    /// 期望变更到的命名空间 URI。
    expect_ns: String,
}

#[allow(deprecated)]
impl NamespaceModifier {
    /// 使用默认 OFD 命名空间创建变更器。
    #[must_use]
    pub fn new() -> Self {
        Self {
            expect_ns: OFD_NAMESPACE.to_string(),
        }
    }

    /// 使用指定命名空间创建变更器。
    #[must_use]
    pub fn with_namespace(namespace: impl Into<String>) -> Self {
        Self {
            expect_ns: namespace.into(),
        }
    }

    /// 获取期望的命名空间 URI。
    #[must_use]
    pub fn expected_namespace(&self) -> &str {
        &self.expect_ns
    }

    /// 替换 XML 字符串中的命名空间声明。
    ///
    /// 将所有 `xmlns:xxx="..."` 声明替换为目标命名空间。
    #[must_use]
    pub fn modify_xml(&self, xml: &str) -> String {
        // 简化实现：替换 xmlns:ofd="..." 声明中的 URI
        let pattern = r#"xmlns:ofd=""#;
        if let Some(start) = xml.find(pattern) {
            let after_prefix = start + pattern.len();
            if let Some(end) = xml[after_prefix..].find('"') {
                let mut result = String::with_capacity(xml.len());
                result.push_str(&xml[..after_prefix]);
                result.push_str(&self.expect_ns);
                result.push_str(&xml[after_prefix + end..]);
                return result;
            }
        }
        xml.to_string()
    }
}

#[allow(deprecated)]
impl Default for NamespaceModifier {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    #[allow(deprecated)]
    use super::*;

    #[test]
    #[allow(deprecated)]
    fn test_namespace_modifier_new() {
        let modifier = NamespaceModifier::new();
        assert_eq!(modifier.expected_namespace(), OFD_NAMESPACE);
    }

    #[test]
    #[allow(deprecated)]
    fn test_namespace_modifier_with_namespace() {
        let modifier = NamespaceModifier::with_namespace("http://custom.ns");
        assert_eq!(modifier.expected_namespace(), "http://custom.ns");
    }

    #[test]
    #[allow(deprecated)]
    fn test_modify_xml() {
        let xml = r#"<?xml version="1.0"?>
<ofd:OFD xmlns:ofd="http://wrong.namespace">
  <ofd:DocBody/>
</ofd:OFD>"#;
        let modifier = NamespaceModifier::new();
        let result = modifier.modify_xml(xml);
        assert!(result.contains(OFD_NAMESPACE));
        assert!(!result.contains("wrong.namespace"));
    }

    #[test]
    #[allow(deprecated)]
    fn test_modify_xml_no_change() {
        let xml = r#"<ofd:OFD xmlns:ofd="http://www.ofdspec.org/2016"/>"#;
        let modifier = NamespaceModifier::new();
        let result = modifier.modify_xml(xml);
        assert!(result.contains(OFD_NAMESPACE));
    }

    #[test]
    #[allow(deprecated)]
    fn test_modify_xml_no_namespace() {
        let xml = "<root><child/></root>";
        let modifier = NamespaceModifier::new();
        let result = modifier.modify_xml(xml);
        assert_eq!(result, xml);
    }
}
