//! 图像插值处理器。
//!
//! 对应 Java: org.ofdrw.archive.convert.handler.ImageInterpolateHandler

#![allow(clippy::case_sensitive_file_extension_comparisons)]

use crate::convert::archive_handler::ArchiveHandler;

/// 图像插值处理器。
///
/// 将 ImageObject 的 Interpolate 属性设为 false。
///
/// 对应 Java: org.ofdrw.archive.convert.handler.ImageInterpolateHandler
#[derive(Debug, Clone, Copy)]
pub struct ImageInterpolateHandler;

impl ArchiveHandler for ImageInterpolateHandler {
    fn name(&self) -> &'static str {
        "ImageInterpolateHandler"
    }

    fn handle(&self, entries: &mut Vec<(String, Vec<u8>)>) -> Result<(), String> {
        for (name, data) in entries.iter_mut() {
            if name.ends_with(".xml") {
                let mut content = String::from_utf8_lossy(data).to_string();
                if content.contains("Interpolate=\"true\"") {
                    content = content.replace("Interpolate=\"true\"", "Interpolate=\"false\"");
                    *data = content.into_bytes();
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn image_interpolate_handler_name() {
        assert_eq!(ImageInterpolateHandler.name(), "ImageInterpolateHandler");
    }

    #[test]
    fn image_interpolate_handler_fixes_true() {
        let mut entries = vec![(
            "Page_0.xml".into(),
            br#"<ofd:ImageObject Interpolate="true"/>"#.to_vec(),
        )];
        ImageInterpolateHandler.handle(&mut entries).unwrap();
        let content = String::from_utf8_lossy(&entries[0].1);
        assert!(content.contains("Interpolate=\"false\""));
        assert!(!content.contains("Interpolate=\"true\""));
    }
}
