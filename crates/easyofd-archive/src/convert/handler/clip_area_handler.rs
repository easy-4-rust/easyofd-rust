//! 裁剪区处理器。
//!
//! 对应 Java: org.ofdrw.archive.convert.handler.ClipAreaHandler

use crate::convert::archive_handler::ArchiveHandler;

/// 裁剪区处理器。
///
/// 对应 Java: org.ofdrw.archive.convert.handler.ClipAreaHandler
#[derive(Debug, Clone, Copy)]
pub struct ClipAreaHandler;

impl ArchiveHandler for ClipAreaHandler {
    fn name(&self) -> &'static str {
        "ClipAreaHandler"
    }

    fn handle(&self, _entries: &mut Vec<(String, Vec<u8>)>) -> Result<(), String> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clip_area_handler_name() {
        assert_eq!(ClipAreaHandler.name(), "ClipAreaHandler");
    }
}
