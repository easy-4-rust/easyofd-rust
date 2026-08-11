//! 清除填充属性处理器。
//!
//! 对应 Java: org.ofdrw.archive.convert.handler.CleanFillAttrHandler

use crate::convert::archive_handler::ArchiveHandler;

/// 清除填充属性处理器。
///
/// 对应 Java: org.ofdrw.archive.convert.handler.CleanFillAttrHandler
#[derive(Debug, Clone, Copy)]
pub struct CleanFillAttrHandler;

impl ArchiveHandler for CleanFillAttrHandler {
    fn name(&self) -> &'static str {
        "CleanFillAttrHandler"
    }

    fn handle(&self, _entries: &mut Vec<(String, Vec<u8>)>) -> Result<(), String> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_fill_attr_handler_name() {
        assert_eq!(CleanFillAttrHandler.name(), "CleanFillAttrHandler");
    }
}
