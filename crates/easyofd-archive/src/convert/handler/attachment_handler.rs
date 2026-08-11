//! 附件处理器。
//!
//! 对应 Java: org.ofdrw.archive.convert.handler.AttachmentHandler

use crate::convert::archive_handler::ArchiveHandler;

/// 附件处理器。
///
/// 对应 Java: org.ofdrw.archive.convert.handler.AttachmentHandler
#[derive(Debug, Clone, Copy)]
pub struct AttachmentHandler;

impl ArchiveHandler for AttachmentHandler {
    fn name(&self) -> &'static str {
        "AttachmentHandler"
    }

    fn handle(&self, _entries: &mut Vec<(String, Vec<u8>)>) -> Result<(), String> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attachment_handler_name() {
        assert_eq!(AttachmentHandler.name(), "AttachmentHandler");
    }

    #[test]
    fn attachment_handler_handle_ok() {
        let mut entries = vec![];
        assert!(AttachmentHandler.handle(&mut entries).is_ok());
    }
}
