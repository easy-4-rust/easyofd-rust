//! 非 Goto 动作处理器。
//!
//! 对应 Java: org.ofdrw.archive.convert.handler.NonGotoActionHandler

use crate::convert::archive_handler::ArchiveHandler;

/// 非 Goto 动作处理器。
///
/// 对应 Java: org.ofdrw.archive.convert.handler.NonGotoActionHandler
#[derive(Debug, Clone, Copy)]
pub struct NonGotoActionHandler;

impl ArchiveHandler for NonGotoActionHandler {
    fn name(&self) -> &'static str {
        "NonGotoActionHandler"
    }

    fn handle(&self, _entries: &mut Vec<(String, Vec<u8>)>) -> Result<(), String> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn non_goto_action_handler_name() {
        assert_eq!(NonGotoActionHandler.name(), "NonGotoActionHandler");
    }
}
