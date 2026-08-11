//! 扩展处理器。
//!
//! 对应 Java: org.ofdrw.archive.convert.handler.ExtensionHandler

use crate::convert::archive_handler::ArchiveHandler;

/// 扩展处理器。
///
/// 移除文档中的扩展信息。
///
/// 对应 Java: org.ofdrw.archive.convert.handler.ExtensionHandler
#[derive(Debug, Clone, Copy)]
pub struct ExtensionHandler;

impl ArchiveHandler for ExtensionHandler {
    fn name(&self) -> &'static str {
        "ExtensionHandler"
    }

    fn handle(&self, entries: &mut Vec<(String, Vec<u8>)>) -> Result<(), String> {
        for (name, data) in entries.iter_mut() {
            if name.ends_with("Document.xml") {
                let content = String::from_utf8_lossy(data).to_string();
                // 移除 Extensions 元素
                let cleaned = remove_xml_element(&content, "Extensions");
                if cleaned.len() != content.len() {
                    *data = cleaned.into_bytes();
                }
            }
        }
        Ok(())
    }
}

/// 简单移除 XML 元素（开始标签到结束标签）。
fn remove_xml_element(content: &str, element_name: &str) -> String {
    let start_tag = format!("<ofd:{element_name}");
    let end_tag = format!("</ofd:{element_name}>");
    let start_tag2 = format!("<{element_name}");
    let end_tag2 = format!("</{element_name}>");

    let mut result = content.to_string();
    // 尝试移除带命名空间的
    if let Some(start_idx) = result.find(&start_tag) {
        if let Some(end_idx) = result.find(&end_tag) {
            let end_pos = end_idx + end_tag.len();
            result = format!("{}{}", &result[..start_idx], &result[end_pos..]);
        }
    }
    // 尝试移除不带命名空间的
    if let Some(start_idx) = result.find(&start_tag2) {
        if let Some(end_idx) = result.find(&end_tag2) {
            let end_pos = end_idx + end_tag2.len();
            result = format!("{}{}", &result[..start_idx], &result[end_pos..]);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extension_handler_name() {
        assert_eq!(ExtensionHandler.name(), "ExtensionHandler");
    }

    #[test]
    fn extension_handler_removes_extensions() {
        let mut entries = vec![(
            "Doc_0/Document.xml".into(),
            br"<ofd:Document><ofd:Extensions><ofd:Ext/></ofd:Extensions></ofd:Document>".to_vec(),
        )];
        ExtensionHandler.handle(&mut entries).unwrap();
        let content = String::from_utf8_lossy(&entries[0].1);
        assert!(!content.contains("Extensions"));
    }

    #[test]
    fn extension_handler_preserves_other_content() {
        let mut entries = vec![(
            "Doc_0/Document.xml".into(),
            br"<ofd:Document><ofd:Pages/></ofd:Document>".to_vec(),
        )];
        ExtensionHandler.handle(&mut entries).unwrap();
        let content = String::from_utf8_lossy(&entries[0].1);
        assert!(content.contains("Pages"));
    }
}
