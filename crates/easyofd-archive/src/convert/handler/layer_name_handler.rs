//! 图层名称处理器。
//!
//! 对应 Java: org.ofdrw.archive.convert.handler.LayerNameHandler

use crate::convert::archive_handler::ArchiveHandler;

/// 图层名称处理器。
///
/// 对应 Java: org.ofdrw.archive.convert.handler.LayerNameHandler
#[derive(Debug, Clone, Copy)]
pub struct LayerNameHandler;

impl ArchiveHandler for LayerNameHandler {
    fn name(&self) -> &'static str {
        "LayerNameHandler"
    }

    fn handle(&self, _entries: &mut Vec<(String, Vec<u8>)>) -> Result<(), String> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layer_name_handler_name() {
        assert_eq!(LayerNameHandler.name(), "LayerNameHandler");
    }
}
