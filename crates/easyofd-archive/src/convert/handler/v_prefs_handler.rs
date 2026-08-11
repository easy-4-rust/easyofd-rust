//! 视图首选项处理器。
//!
//! 对应 Java: org.ofdrw.archive.convert.handler.VPrefsHandler

use crate::convert::archive_handler::ArchiveHandler;

/// 视图首选项处理器。
///
/// 对应 Java: org.ofdrw.archive.convert.handler.VPrefsHandler
#[derive(Debug, Clone, Copy)]
pub struct VPrefsHandler;

impl ArchiveHandler for VPrefsHandler {
    fn name(&self) -> &'static str {
        "VPrefsHandler"
    }

    fn handle(&self, _entries: &mut Vec<(String, Vec<u8>)>) -> Result<(), String> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn v_prefs_handler_name() {
        assert_eq!(VPrefsHandler.name(), "VPrefsHandler");
    }
}
