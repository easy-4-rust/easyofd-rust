//! 扩展属性。

/// 对应 Java: org.ofdrw.core.extendObj.Property
///
/// 扩展属性，以键值对形式存储扩展的配置信息。
#[derive(Debug, Clone)]
pub struct Property {
    /// 属性名。
    pub name: String,
    /// 属性值。
    pub value: String,
}

impl Property {
    /// 创建新的扩展属性。
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
            "<Property Name=\"{}\" Value=\"{}\"/>",
            self.name, self.value
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_property_new() {
        let p = Property::new("key", "value");
        assert_eq!(p.name, "key");
        assert_eq!(p.value, "value");
    }

    #[test]
    fn test_property_xml() {
        let p = Property::new("encoding", "UTF-8");
        let xml = p.to_xml_string();
        assert!(xml.contains("Name=\"encoding\""));
        assert!(xml.contains("Value=\"UTF-8\""));
        assert!(xml.contains("/>"));
    }
}
