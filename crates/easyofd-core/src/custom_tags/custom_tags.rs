//! 标引入口。

use super::CustomTag;

/// 对应 Java: org.ofdrw.core.basicStructure.CustomTags
///
/// 自定义标引集合，包含文档中所有自定义标引。
#[derive(Debug, Clone)]
pub struct CustomTags {
    /// 标引列表。
    pub tags: Vec<CustomTag>,
}

impl CustomTags {
    /// 创建空的标引集。
    #[must_use]
    pub fn new() -> Self {
        Self { tags: Vec::new() }
    }

    /// 添加一个标引。
    pub fn push(&mut self, tag: CustomTag) {
        self.tags.push(tag);
    }

    /// 序列化为 XML 字符串。
    #[must_use]
    pub fn to_xml_string(&self) -> String {
        let inner: String = self.tags.iter().map(CustomTag::to_xml_string).collect();
        format!("<CustomTags>{inner}</CustomTags>")
    }
}

impl Default for CustomTags {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_custom_tags_new() {
        let ct = CustomTags::new();
        assert!(ct.tags.is_empty());
        let ct2 = CustomTags::default();
        assert!(ct2.tags.is_empty());
    }

    #[test]
    fn test_custom_tags_push_and_xml() {
        let mut ct = CustomTags::new();
        ct.push(CustomTag::new("tag1", "value1"));
        ct.push(CustomTag::new("tag2", "value2"));
        assert_eq!(ct.tags.len(), 2);
        let xml = ct.to_xml_string();
        assert!(xml.contains("<CustomTags>"));
        assert!(xml.contains("</CustomTags>"));
        assert!(xml.contains("tag1"));
        assert!(xml.contains("value2"));
    }
}
