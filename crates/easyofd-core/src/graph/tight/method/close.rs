//! 闭合路径方法。
//!
//! 对应 Java: org.ofdrw.core.graph.tight.method.Close

use crate::xml_element::{XmlElement, XmlElementError, XmlNode};

/// 闭合路径方法。
///
/// 自动闭合到当前路径的起始点，并以该点为当前点。
///
/// 对应 Java: org.ofdrw.core.graph.tight.method.Close
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Close;

impl Close {
    /// 创建闭合命令。
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// 序列化为缩写数据字符串（C 命令格式）。
    #[must_use]
    pub fn to_abbreviated_string(&self) -> &'static str {
        "C"
    }
}

impl Default for Close {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for Close {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("C")
    }
}

impl XmlElement for Close {
    /// 对应 Java: Close 元素名 "Close"。
    fn element_name(&self) -> &'static str {
        "Close"
    }

    fn attributes(&self) -> Vec<(String, String)> {
        Vec::new()
    }

    /// 覆写 write_xml：文本内容为 "C"。
    fn write_xml(&self, out: &mut String) {
        out.push_str("<Close>C</Close>");
    }

    fn from_xml(_node: &XmlNode) -> Result<Self, XmlElementError> {
        Ok(Self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::xml_parse::parse_xml_to_nodes;

    #[test]
    fn close_new() {
        let c = Close::new();
        assert_eq!(c.to_abbreviated_string(), "C");
    }

    #[test]
    fn close_display() {
        assert_eq!(format!("{Close}"), "C");
    }

    #[test]
    fn close_default() {
        let c = Close;
        assert_eq!(c.to_abbreviated_string(), "C");
    }

    #[test]
    fn test_xml_element_name() {
        assert_eq!(Close.element_name(), "Close");
    }

    #[test]
    fn test_xml_element_roundtrip() {
        let xml = Close.to_xml();
        assert_eq!(xml, "<Close>C</Close>");
        let node = parse_xml_to_nodes(&xml).unwrap();
        let c2 = Close::from_xml(&node).unwrap();
        assert_eq!(Close, c2);
    }
}
