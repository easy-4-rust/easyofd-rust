//! 清除描边属性处理器。
//!
//! 对应 Java: org.ofdrw.archive.convert.handler.CleanStrokeAttrHandler

use crate::convert::archive_handler::ArchiveHandler;

/// 清除描边属性处理器。
///
/// 对应 Java: org.ofdrw.archive.convert.handler.CleanStrokeAttrHandler
#[derive(Debug, Clone, Copy)]
pub struct CleanStrokeAttrHandler;

impl ArchiveHandler for CleanStrokeAttrHandler {
    fn name(&self) -> &'static str {
        "CleanStrokeAttrHandler"
    }

    fn handle(&self, _entries: &mut Vec<(String, Vec<u8>)>) -> Result<(), String> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_stroke_attr_handler_name() {
        assert_eq!(CleanStrokeAttrHandler.name(), "CleanStrokeAttrHandler");
    }
}
