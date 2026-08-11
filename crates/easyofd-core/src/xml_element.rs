//! XML 元素树抽象。
//!
//! 对应 Java: org.ofdrw.core.OFDElement
//!
//! ofdrw 中每个 OFD 类型都是 XML 元素（可 `toXML()` 自序列化、从 XML 解析）。
//! 本模块用 Rust trait 复刻该范式：类型实现 [`XmlElement`]，提供元素名、
//! 属性、子元素与文本；[`to_xml`](XmlElement::to_xml) 手工拼接 XML（对应
//! ofdrw writer 侧），[`from_xml`](XmlElement::from_xml) 从
//! [`XmlNode`] 解析（quick-xml 解析产物）。

use std::fmt::Write as _;

/// XML 节点（解析中间表示）。
///
/// quick-xml 解析结果归一化为统一的节点树，`XmlElement::from_xml` 从节点
/// 恢复类型。
#[derive(Debug, Clone, PartialEq)]
pub struct XmlNode {
    /// 元素名（不含命名空间前缀，如 "TextObject"）。
    pub name: String,
    /// 属性列表（保序）。
    pub attrs: Vec<(String, String)>,
    /// 子元素。
    pub children: Vec<XmlNode>,
    /// 文本内容（若为文本节点）。
    pub text: Option<String>,
}

impl XmlNode {
    /// 创建元素节点。
    #[must_use]
    pub fn element(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            attrs: Vec::new(),
            children: Vec::new(),
            text: None,
        }
    }

    /// 创建文本节点。
    #[must_use]
    pub fn text_node(text: impl Into<String>) -> Self {
        Self {
            name: String::new(),
            attrs: Vec::new(),
            children: Vec::new(),
            text: Some(text.into()),
        }
    }

    /// 添加属性。
    #[must_use]
    pub fn attr(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attrs.push((key.into(), value.into()));
        self
    }

    /// 添加子元素。
    pub fn push_child(&mut self, child: XmlNode) {
        self.children.push(child);
    }

    /// 读取属性值。
    #[must_use]
    pub fn get_attr(&self, key: &str) -> Option<&str> {
        self.attrs
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }

    /// 首个匹配名字的子元素。
    #[must_use]
    pub fn child(&self, name: &str) -> Option<&XmlNode> {
        self.children.iter().find(|c| c.name == name)
    }

    /// 所有匹配名字的子元素。
    pub fn children_named<'a>(&'a self, name: &'a str) -> impl Iterator<Item = &'a XmlNode> {
        self.children.iter().filter(move |c| c.name == name)
    }

    /// 将节点序列化为 XML 片段（不含 XML 声明）。
    ///
    /// 对应 Java: OFDElement.toString()
    #[must_use]
    pub fn to_xml_string(&self) -> String {
        let mut out = String::new();
        self.write_self_xml(&mut out);
        out
    }
}

/// XML 元素 trait（对应 Java: ofdrw OFDElement）。
///
/// 实现者描述一个 OFD XML 元素：元素名、属性、子元素、文本。
/// [`to_xml`](XmlElement::to_xml) 手工拼 XML（writer 侧），
/// [`from_xml`](XmlElement::from_xml) 从节点树解析（quick-xml 解析侧）。
pub trait XmlElement {
    /// 元素名（如 "TextObject"、"ofd:Page"）。
    fn element_name(&self) -> &'static str;

    /// XML 属性列表。
    fn attributes(&self) -> Vec<(String, String)>;

    /// 子元素（作为 `XmlNode` 提供，便于嵌套序列化）。
    fn child_nodes(&self) -> Vec<XmlNode> {
        Vec::new()
    }

    /// 文本内容（简单文本元素）。
    fn text_content(&self) -> Option<&str> {
        None
    }

    /// 手工拼接 XML（对应 ofdrw writer 侧；子元素由 `child_nodes` 递归）。
    fn to_xml(&self) -> String {
        let mut out = String::new();
        self.write_xml(&mut out);
        out
    }

    /// 将节点树写为 XML 片段（无 XML 声明）。
    fn write_xml(&self, out: &mut String) {
        let name = self.element_name();
        out.push('<');
        out.push_str(name);
        for (k, v) in self.attributes() {
            out.push(' ');
            out.push_str(&k);
            out.push_str("=\"");
            out.push_str(&xml_escape(&v));
            out.push('"');
        }
        let children = self.child_nodes();
        let text = self.text_content();
        if children.is_empty() && text.is_none() {
            out.push_str("/>");
            return;
        }
        out.push('>');
        if let Some(t) = text {
            out.push_str(&xml_escape(t));
        }
        for child in children {
            child.write_self_xml(out);
        }
        out.push_str("</");
        out.push_str(name);
        out.push('>');
    }

    /// 从节点树解析（对应 ofdrw 解析侧；quick-xml 产物）。
    ///
    /// # 错误
    ///
    /// 节点与元素结构不匹配时返回错误。
    fn from_xml(node: &XmlNode) -> Result<Self, XmlElementError>
    where
        Self: Sized;
}

/// 解析错误。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XmlElementError(pub String);

impl std::fmt::Display for XmlElementError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for XmlElementError {}

impl XmlNode {
    /// 递归写 XML 片段。
    fn write_self_xml(&self, out: &mut String) {
        if let Some(text) = &self.text {
            out.push_str(&xml_escape(text));
            return;
        }
        let _ = write!(out, "<{}", self.name);
        for (k, v) in &self.attrs {
            let _ = write!(out, " {k}=\"{}\"", xml_escape(v));
        }
        if self.children.is_empty() {
            out.push_str("/>");
            return;
        }
        out.push('>');
        for child in &self.children {
            child.write_self_xml(out);
        }
        let _ = write!(out, "</{}>", self.name);
    }
}

/// XML 特殊字符转义。
#[must_use]
pub fn xml_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            _ => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    struct SampleElement {
        id: String,
        name: String,
    }

    impl XmlElement for SampleElement {
        fn element_name(&self) -> &'static str {
            "Sample"
        }

        fn attributes(&self) -> Vec<(String, String)> {
            vec![("ID".to_string(), self.id.clone())]
        }

        fn text_content(&self) -> Option<&str> {
            Some(&self.name)
        }

        fn from_xml(node: &XmlNode) -> Result<Self, XmlElementError> {
            Ok(Self {
                id: node.get_attr("ID").unwrap_or_default().to_string(),
                name: node.text.clone().unwrap_or_default(),
            })
        }
    }

    #[test]
    fn test_to_xml_simple() {
        let el = SampleElement {
            id: "1".to_string(),
            name: "测试 & 数据".to_string(),
        };
        let xml = el.to_xml();
        assert_eq!(xml, r#"<Sample ID="1">测试 &amp; 数据</Sample>"#);
    }

    #[test]
    fn test_node_roundtrip() {
        let mut node = XmlNode::element("Page")
            .attr("ID", "5")
            .attr("BaseLoc", "Pages/Page_0/Content.xml");
        node.push_child(XmlNode::element("Layer"));
        assert_eq!(node.get_attr("ID"), Some("5"));
        assert_eq!(node.child("Layer").unwrap().name, "Layer");

        let mut out = String::new();
        node.write_self_xml(&mut out);
        let xml = out;
        assert!(xml.starts_with(r#"<Page ID="5" BaseLoc="Pages/Page_0/Content.xml">"#));
        assert!(xml.contains("<Layer/>"));
    }

    #[test]
    fn test_xml_escape() {
        assert_eq!(xml_escape("<a&b\"c'>"), "&lt;a&amp;b&quot;c&apos;&gt;");
    }
}
