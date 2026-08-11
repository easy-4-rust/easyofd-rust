//! 单个标引。

/// 对应 Java: org.ofdrw.core.basicStructure.CustomTag
///
/// 单个自定义标引，包含标引名称和值。
#[derive(Debug, Clone)]
pub struct CustomTag {
    /// 标引名称。
    pub name: String,
    /// 标引值。
    pub value: String,
}

impl CustomTag {
    /// 创建新的标引。
    #[must_use]
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
        }
    }

    /// 序列化为 XML 字符串。
    #[must_use]
    pub fn to_xml_string(&self) -> String {
        format!(
            "<CustomTag Name=\"{}\" Value=\"{}\"/>",
            self.name, self.value
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_custom_tag_new() {
        let t = CustomTag::new("keyword", "important");
        assert_eq!(t.name, "keyword");
        assert_eq!(t.value, "important");
    }

    #[test]
    fn test_custom_tag_xml() {
        let t = CustomTag::new("category", "invoice");
        let xml = t.to_xml_string();
        assert!(xml.contains("Name=\"category\""));
        assert!(xml.contains("Value=\"invoice\""));
        assert!(xml.contains("/>"));
    }
}
