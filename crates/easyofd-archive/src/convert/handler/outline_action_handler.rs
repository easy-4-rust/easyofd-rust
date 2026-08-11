//! 大纲动作处理器。
//!
//! 对应 Java: org.ofdrw.archive.convert.handler.OutlineActionHandler

use crate::convert::archive_handler::ArchiveHandler;

/// 大纲动作处理器。
///
/// 对应 Java: org.ofdrw.archive.convert.handler.OutlineActionHandler
#[derive(Debug, Clone, Copy)]
pub struct OutlineActionHandler;

impl ArchiveHandler for OutlineActionHandler {
    fn name(&self) -> &'static str {
        "OutlineActionHandler"
    }

    fn handle(&self, _entries: &mut Vec<(String, Vec<u8>)>) -> Result<(), String> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn outline_action_handler_name() {
        assert_eq!(OutlineActionHandler.name(), "OutlineActionHandler");
    }
}
