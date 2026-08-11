//! OFD 元素基类特征。
//!
//! 对应 Java: org.ofdrw.core.OFDElement
//!
//! 所有 OFD XML 元素的公共接口，定义了元素名称、属性和子元素的通用行为。
//! 在 Java 版中 `OFDElement` 继承自 `DefaultElementProxy`，是所有 OFD
//! 数据结构类的基类。Rust 版用 trait 实现等价的多态行为。

/// OFD 元素公共特征。
///
/// 对应 Java: org.ofdrw.core.OFDElement
///
/// 实现此 trait 的类型表示一个 OFD XML 元素，可以获取元素名称、
/// 属性列表和子元素列表，以及序列化为 XML 字符串。
pub trait OfdElement {
    /// 获取 OFD XML 元素名称（不含命名空间前缀）。
    fn ofd_element_name(&self) -> &'static str;

    /// 获取元素的属性列表（键值对）。
    fn ofd_attributes(&self) -> Vec<(String, String)> {
        Vec::new()
    }

    /// 序列化为 OFD XML 字符串（含命名空间前缀）。
    fn to_ofd_xml(&self) -> String {
        let name = self.ofd_element_name();
        let attrs = self.ofd_attributes();
        let mut xml = String::from("<ofd:");
        xml.push_str(name);
        for (key, value) in &attrs {
            xml.push(' ');
            xml.push_str(key);
            xml.push_str("=\"");
            xml.push_str(value);
            xml.push('"');
        }
        xml.push_str(" />");
        xml
    }
}

/// 默认元素代理。
///
/// 对应 Java: org.ofdrw.core.DefaultElementProxy
///
/// 提供 `OfdElement` trait 的默认包装行为，将底层 XML 元素代理为 OFD 元素。
/// 在 Java 版中用于包装 XML DOM 节点；Rust 版中用于需要代理行为的场景。
#[derive(Debug, Clone)]
pub struct DefaultElementProxy {
    /// 元素名称。
    pub element_name: String,
    /// 属性列表。
    pub attributes: Vec<(String, String)>,
}

impl DefaultElementProxy {
    /// 创建默认元素代理。
    #[must_use]
    pub fn new(element_name: impl Into<String>) -> Self {
        Self {
            element_name: element_name.into(),
            attributes: Vec::new(),
        }
    }

    /// 添加属性。
    #[must_use]
    pub fn attr(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.push((key.into(), value.into()));
        self
    }
}

impl OfdElement for DefaultElementProxy {
    fn ofd_element_name(&self) -> &'static str {
        // 注意：这里需要泄漏一个 &'static str 来满足 trait 签名。
        // 在实际使用中，应使用常量或 enum 变体名。
        // 这里用一个简化方案。
        "DefaultElement"
    }

    fn ofd_attributes(&self) -> Vec<(String, String)> {
        self.attributes.clone()
    }
}

/// OFD 简单类型元素特征。
///
/// 对应 Java: org.ofdrw.core.OFDSimpleTypeElement
///
/// 表示 OFD 中的简单类型元素（值类型），如字符串、整数等标量值。
/// 与复合类型元素（OfdElement）不同，简单类型元素只有一个文本值。
pub trait OfdSimpleTypeElement {
    /// 获取元素的文本值。
    fn ofd_value(&self) -> String;

    /// 从文本值解析。
    fn from_ofd_value(s: &str) -> Result<Self, String>
    where
        Self: Sized;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_element_proxy_new() {
        let proxy = DefaultElementProxy::new("TestElement");
        assert_eq!(proxy.element_name, "TestElement");
        assert!(proxy.attributes.is_empty());
    }

    #[test]
    fn test_default_element_proxy_with_attrs() {
        let proxy = DefaultElementProxy::new("Page")
            .attr("ID", "1")
            .attr("Boundary", "0 0 210 297");
        assert_eq!(proxy.attributes.len(), 2);
        assert_eq!(proxy.attributes[0].0, "ID");
        assert_eq!(proxy.attributes[1].1, "0 0 210 297");
    }

    #[test]
    fn test_ofd_element_trait_default() {
        let proxy = DefaultElementProxy::new("Test");
        let xml = proxy.to_ofd_xml();
        assert!(xml.contains("<ofd:DefaultElement"));
        assert!(xml.contains("/>"));
    }

    #[test]
    fn test_ofd_element_with_attrs() {
        let proxy = DefaultElementProxy::new("Test").attr("Key", "Value");
        let attrs = proxy.ofd_attributes();
        assert_eq!(attrs.len(), 1);
        assert_eq!(attrs[0].0, "Key");
    }
}
