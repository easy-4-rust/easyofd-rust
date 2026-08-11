//! quick-xml 解析桥接：XML 字符串 → [`XmlNode`] 节点树。
//!
//! 对应 Java: ofdrw 解析侧（Element 代理）。

use quick_xml::Reader;
use quick_xml::events::Event;

use crate::xml_element::XmlNode;

/// 将 XML 字符串解析为 `XmlNode` 节点树（首个根元素）。
///
/// # 错误
///
/// XML 语法错误时返回错误。
pub fn parse_xml_to_nodes(xml: &str) -> Result<XmlNode, String> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(false);
    let mut buf = Vec::new();
    let mut stack: Vec<XmlNode> = Vec::new();
    let mut root: Option<XmlNode> = None;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let name = local_name(e.name().as_ref());
                let mut node = XmlNode::element(name);
                for attr in e.attributes().flatten() {
                    let key = local_name(attr.key.as_ref());
                    let value = attr
                        .decoded_and_normalized_value(
                            quick_xml::XmlVersion::Explicit1_0,
                            reader.decoder(),
                        )
                        .unwrap_or_default()
                        .to_string();
                    node.attrs.push((key, value));
                }
                stack.push(node);
            }
            Ok(Event::Empty(e)) => {
                let name = local_name(e.name().as_ref());
                let mut node = XmlNode::element(name);
                for attr in e.attributes().flatten() {
                    let key = local_name(attr.key.as_ref());
                    let value = attr
                        .decoded_and_normalized_value(
                            quick_xml::XmlVersion::Explicit1_0,
                            reader.decoder(),
                        )
                        .unwrap_or_default()
                        .to_string();
                    node.attrs.push((key, value));
                }
                attach_node(&mut stack, &mut root, node);
            }
            Ok(Event::Text(e)) => {
                let text = e
                    .xml10_content()
                    .map(|c| c.into_owned())
                    .unwrap_or_default();
                if !text.trim().is_empty() {
                    if let Some(top) = stack.last_mut() {
                        top.text = Some(text);
                    }
                }
            }
            Ok(Event::End(_)) => {
                if let Some(node) = stack.pop() {
                    attach_node(&mut stack, &mut root, node);
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(format!("XML 解析失败: {e}")),
            _ => {}
        }
        buf.clear();
    }

    root.ok_or_else(|| "XML 无根元素".to_string())
}

/// 去掉命名空间前缀（"ofd:Page" → "Page"）。
fn local_name(name: &[u8]) -> String {
    let s = String::from_utf8_lossy(name);
    match s.rsplit_once(':') {
        Some((_, local)) => local.to_string(),
        None => s.into_owned(),
    }
}

/// 将节点挂到栈顶父节点或作为根。
fn attach_node(stack: &mut [XmlNode], root: &mut Option<XmlNode>, node: XmlNode) {
    match stack.last_mut() {
        Some(parent) => parent.children.push(node),
        None => *root = Some(node),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple() {
        let xml = r#"<ofd:Page ID="1" BaseLoc="Pages/Page_0/Content.xml"/>"#;
        let node = parse_xml_to_nodes(xml).unwrap();
        assert_eq!(node.name, "Page");
        assert_eq!(node.get_attr("ID"), Some("1"));
        assert_eq!(node.get_attr("BaseLoc"), Some("Pages/Page_0/Content.xml"));
    }

    #[test]
    fn test_parse_nested_with_text() {
        let xml = r"<Document><CommonData><MaxUnitID>88</MaxUnitID></CommonData></Document>";
        let node = parse_xml_to_nodes(xml).unwrap();
        let common = node.child("CommonData").unwrap();
        let max = common.child("MaxUnitID").unwrap();
        assert_eq!(max.text.as_deref(), Some("88"));
    }

    #[test]
    fn test_parse_multiple_children() {
        let xml = r#"<Pages><Page ID="1"/><Page ID="2"/></Pages>"#;
        let node = parse_xml_to_nodes(xml).unwrap();
        let pages: Vec<_> = node.children_named("Page").collect();
        assert_eq!(pages.len(), 2);
        assert_eq!(pages[0].get_attr("ID"), Some("1"));
    }
}
