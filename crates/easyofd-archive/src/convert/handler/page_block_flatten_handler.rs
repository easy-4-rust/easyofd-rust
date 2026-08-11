//! PageBlock 展平处理器。
//!
//! 对应 Java: org.ofdrw.archive.convert.handler.PageBlockFlattenHandler

use crate::convert::archive_handler::ArchiveHandler;

/// PageBlock 展平处理器。
///
/// 对应 Java: org.ofdrw.archive.convert.handler.PageBlockFlattenHandler
#[derive(Debug, Clone, Copy)]
pub struct PageBlockFlattenHandler;

impl ArchiveHandler for PageBlockFlattenHandler {
    fn name(&self) -> &'static str {
        "PageBlockFlattenHandler"
    }

    fn handle(&self, _entries: &mut Vec<(String, Vec<u8>)>) -> Result<(), String> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_block_flatten_handler_name() {
        assert_eq!(PageBlockFlattenHandler.name(), "PageBlockFlattenHandler");
    }
}
