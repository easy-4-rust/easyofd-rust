//! DocType 处理器。
//!
//! 对应 Java: org.ofdrw.archive.convert.handler.DocTypeHandler

use crate::convert::archive_handler::ArchiveHandler;

/// DocType 处理器。
///
/// 将 OFD.xml 的 DocType 属性改为 "OFD-A"。
///
/// 对应 Java: org.ofdrw.archive.convert.handler.DocTypeHandler
#[derive(Debug, Clone, Copy)]
pub struct DocTypeHandler;

impl ArchiveHandler for DocTypeHandler {
    fn name(&self) -> &'static str {
        "DocTypeHandler"
    }

    fn handle(&self, entries: &mut Vec<(String, Vec<u8>)>) -> Result<(), String> {
        for (name, data) in entries.iter_mut() {
            if name == "OFD.xml" {
                let mut content = String::from_utf8_lossy(data).to_string();
                content = content.replace("DocType=\"OFD\"", "DocType=\"OFD-A\"");
                *data = content.into_bytes();
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn doc_type_handler_name() {
        assert_eq!(DocTypeHandler.name(), "DocTypeHandler");
    }

    #[test]
    fn doc_type_handler_replaces_doctype() {
        let mut entries = vec![(
            "OFD.xml".into(),
            br#"<?xml version="1.0"?><ofd:OFD DocType="OFD" Version="1.2"/>"#.to_vec(),
        )];
        DocTypeHandler.handle(&mut entries).unwrap();
        let content = String::from_utf8_lossy(&entries[0].1);
        assert!(content.contains("DocType=\"OFD-A\""));
        assert!(!content.contains("DocType=\"OFD\""));
    }
}
