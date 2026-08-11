//! 外部资源处理器。
//!
//! 对应 Java: org.ofdrw.archive.convert.handler.ExternalResourceHandler

use crate::convert::archive_handler::ArchiveHandler;

/// 外部资源处理器。
///
/// 对应 Java: org.ofdrw.archive.convert.handler.ExternalResourceHandler
#[derive(Debug, Clone, Copy)]
pub struct ExternalResourceHandler;

impl ArchiveHandler for ExternalResourceHandler {
    fn name(&self) -> &'static str {
        "ExternalResourceHandler"
    }

    fn handle(&self, _entries: &mut Vec<(String, Vec<u8>)>) -> Result<(), String> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn external_resource_handler_name() {
        assert_eq!(ExternalResourceHandler.name(), "ExternalResourceHandler");
    }
}
