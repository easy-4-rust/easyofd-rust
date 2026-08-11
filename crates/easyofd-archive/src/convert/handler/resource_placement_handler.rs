//! 资源位置处理器。
//!
//! 对应 Java: org.ofdrw.archive.convert.handler.ResourcePlacementHandler

use crate::convert::archive_handler::ArchiveHandler;

/// 资源位置处理器。
///
/// 对应 Java: org.ofdrw.archive.convert.handler.ResourcePlacementHandler
#[derive(Debug, Clone, Copy)]
pub struct ResourcePlacementHandler;

impl ArchiveHandler for ResourcePlacementHandler {
    fn name(&self) -> &'static str {
        "ResourcePlacementHandler"
    }

    fn handle(&self, _entries: &mut Vec<(String, Vec<u8>)>) -> Result<(), String> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resource_placement_handler_name() {
        assert_eq!(ResourcePlacementHandler.name(), "ResourcePlacementHandler");
    }
}
