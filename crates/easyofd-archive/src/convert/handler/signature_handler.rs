//! 签名处理器。
//!
//! 对应 Java: org.ofdrw.archive.convert.handler.SignatureHandler

use crate::convert::archive_handler::ArchiveHandler;

/// 签名处理器。
///
/// 对应 Java: org.ofdrw.archive.convert.handler.SignatureHandler
#[derive(Debug, Clone, Copy)]
pub struct SignatureHandler;

impl ArchiveHandler for SignatureHandler {
    fn name(&self) -> &'static str {
        "SignatureHandler"
    }

    fn handle(&self, _entries: &mut Vec<(String, Vec<u8>)>) -> Result<(), String> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signature_handler_name() {
        assert_eq!(SignatureHandler.name(), "SignatureHandler");
    }
}
