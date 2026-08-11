//! 权限处理器。
//!
//! 对应 Java: org.ofdrw.archive.convert.handler.PermissionHandler

use crate::convert::archive_handler::ArchiveHandler;

/// 权限处理器。
///
/// 移除文档中的权限声明。
///
/// 对应 Java: org.ofdrw.archive.convert.handler.PermissionHandler
#[derive(Debug, Clone, Copy)]
pub struct PermissionHandler;

impl ArchiveHandler for PermissionHandler {
    fn name(&self) -> &'static str {
        "PermissionHandler"
    }

    fn handle(&self, entries: &mut Vec<(String, Vec<u8>)>) -> Result<(), String> {
        for (name, data) in entries.iter_mut() {
            if name.ends_with("Document.xml") {
                let content = String::from_utf8_lossy(data).to_string();
                let cleaned = remove_permissions(&content);
                if cleaned != content {
                    *data = cleaned.into_bytes();
                }
            }
        }
        Ok(())
    }
}

fn remove_permissions(content: &str) -> String {
    let mut result = content.to_string();
    // Remove self-closing tag
    result = result.replace("<ofd:Permissions/>", "");
    result = result.replace("<Permissions/>", "");
    // Remove element with content
    if let Some(start_idx) = result.find("<ofd:Permissions") {
        if let Some(end_idx) = result.find("</ofd:Permissions>") {
            let end_pos = end_idx + "</ofd:Permissions>".len();
            result = format!("{}{}", &result[..start_idx], &result[end_pos..]);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn permission_handler_name() {
        assert_eq!(PermissionHandler.name(), "PermissionHandler");
    }

    #[test]
    fn permission_handler_removes_self_closing() {
        let mut entries = vec![(
            "Doc_0/Document.xml".into(),
            br"<ofd:Document><ofd:Permissions/></ofd:Document>".to_vec(),
        )];
        PermissionHandler.handle(&mut entries).unwrap();
        let content = String::from_utf8_lossy(&entries[0].1);
        assert!(!content.contains("Permissions"));
        assert!(content.contains("Document"));
    }

    #[test]
    fn permission_handler_removes_with_content() {
        let mut entries = vec![(
            "Doc_0/Document.xml".into(),
            br"<ofd:Document><ofd:Permissions><ofd:Perm/></ofd:Permissions></ofd:Document>"
                .to_vec(),
        )];
        PermissionHandler.handle(&mut entries).unwrap();
        let content = String::from_utf8_lossy(&entries[0].1);
        assert!(!content.contains("Permissions"));
    }
}
