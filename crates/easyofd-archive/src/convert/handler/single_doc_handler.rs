//! 单文档处理器。
//!
//! 对应 Java: org.ofdrw.archive.convert.handler.SingleDocHandler

use crate::convert::archive_handler::ArchiveHandler;

/// 单文档处理器。
///
/// 对应 Java: org.ofdrw.archive.convert.handler.SingleDocHandler
#[derive(Debug, Clone, Copy)]
pub struct SingleDocHandler;

impl ArchiveHandler for SingleDocHandler {
    fn name(&self) -> &'static str {
        "SingleDocHandler"
    }

    fn handle(&self, _entries: &mut Vec<(String, Vec<u8>)>) -> Result<(), String> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_doc_handler_name() {
        assert_eq!(SingleDocHandler.name(), "SingleDocHandler");
    }
}
