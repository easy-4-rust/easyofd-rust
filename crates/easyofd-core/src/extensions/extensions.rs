//! 扩展根节点。

use super::CtExtension;

/// 对应 Java: org.ofdrw.core.extendObj.Extensions
///
/// OFD 文档扩展根节点，包含所有扩展定义。
#[derive(Debug, Clone)]
pub struct Extensions {
    /// 扩展列表。
    pub extensions: Vec<CtExtension>,
}

impl Extensions {
    /// 创建空的扩展容器。
    #[must_use]
    pub fn new() -> Self {
        Self {
            extensions: Vec::new(),
        }
    }

    /// 添加一个扩展。
    pub fn push(&mut self, ext: CtExtension) {
        self.extensions.push(ext);
    }

    /// 序列化为 XML 字符串。
    #[must_use]
    pub fn to_xml_string(&self) -> String {
        let inner: String = self
            .extensions
            .iter()
            .map(CtExtension::to_xml_string)
            .collect();
        format!("<Extensions>{inner}</Extensions>")
    }
}

impl Default for Extensions {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extensions::Property;

    #[test]
    fn test_extensions_new() {
        let e = Extensions::new();
        assert!(e.extensions.is_empty());
        let e2 = Extensions::default();
        assert!(e2.extensions.is_empty());
    }

    #[test]
    fn test_extensions_push_and_xml() {
        let mut e = Extensions::new();
        e.push(CtExtension::new("ext1", "1.0").with_property(Property::new("key", "val")));
        assert_eq!(e.extensions.len(), 1);
        let xml = e.to_xml_string();
        assert!(xml.contains("<Extensions>"));
        assert!(xml.contains("</Extensions>"));
        assert!(xml.contains("ext1"));
    }
}
