//! 注释处理器。
//!
//! 对应 Java: org.ofdrw.archive.convert.handler.AnnotationHandler

use crate::convert::archive_handler::ArchiveHandler;

/// 注释处理器。
///
/// 对应 Java: org.ofdrw.archive.convert.handler.AnnotationHandler
#[derive(Debug, Clone, Copy)]
pub struct AnnotationHandler;

impl ArchiveHandler for AnnotationHandler {
    fn name(&self) -> &'static str {
        "AnnotationHandler"
    }

    fn handle(&self, _entries: &mut Vec<(String, Vec<u8>)>) -> Result<(), String> {
        // Phase 1: 空实现，后续可添加注释属性修正逻辑
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn annotation_handler_name() {
        assert_eq!(AnnotationHandler.name(), "AnnotationHandler");
    }

    #[test]
    fn annotation_handler_handle_ok() {
        let mut entries = vec![];
        assert!(AnnotationHandler.handle(&mut entries).is_ok());
    }
}
