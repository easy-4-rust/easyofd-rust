//! 单个扩展。

use super::Property;

/// 对应 Java: org.ofdrw.core.extendObj.CT_Extension
///
/// 单个扩展定义，包含扩展名称、版本和属性列表。
#[derive(Debug, Clone)]
pub struct CtExtension {
    /// 扩展名称。
    pub name: String,
    /// 扩展版本。
    pub version: String,
    /// 扩展属性列表。
    pub properties: Vec<Property>,
}

impl CtExtension {
    /// 创建新的扩展。
    #[must_use]
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            properties: Vec::new(),
        }
    }

    /// 添加属性（链式调用）。
    #[must_use]
    pub fn with_property(mut self, prop: Property) -> Self {
        self.properties.push(prop);
        self
    }

    /// 添加属性。
    pub fn push_property(&mut self, prop: Property) {
        self.properties.push(prop);
    }

    /// 序列化为 XML 字符串。
    #[must_use]
    pub fn to_xml_string(&self) -> String {
        let inner: String = self
            .properties
            .iter()
            .map(Property::to_xml_string)
            .collect();
        format!(
            "<Extension Name=\"{}\" Version=\"{}\">{inner}</Extension>",
            self.name, self.version
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ct_extension_new() {
        let ext = CtExtension::new("myext", "1.0");
        assert_eq!(ext.name, "myext");
        assert_eq!(ext.version, "1.0");
        assert!(ext.properties.is_empty());
    }

    #[test]
    fn test_ct_extension_with_property_and_xml() {
        let ext = CtExtension::new("myext", "2.0")
            .with_property(Property::new("k1", "v1"))
            .with_property(Property::new("k2", "v2"));
        assert_eq!(ext.properties.len(), 2);
        let xml = ext.to_xml_string();
        assert!(xml.contains("Name=\"myext\""));
        assert!(xml.contains("Version=\"2.0\""));
        assert!(xml.contains("k1"));
        assert!(xml.contains("v2"));
    }
}
