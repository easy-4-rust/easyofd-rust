//! 图像扩展处理器。
//!
//! 对应 Java: org.ofdrw.archive.convert.handler.ImageExtensionHandler

use crate::convert::archive_handler::ArchiveHandler;

/// 图像扩展处理器。
///
/// 对应 Java: org.ofdrw.archive.convert.handler.ImageExtensionHandler
#[derive(Debug, Clone, Copy)]
pub struct ImageExtensionHandler;

impl ArchiveHandler for ImageExtensionHandler {
    fn name(&self) -> &'static str {
        "ImageExtensionHandler"
    }

    fn handle(&self, _entries: &mut Vec<(String, Vec<u8>)>) -> Result<(), String> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn image_extension_handler_name() {
        assert_eq!(ImageExtensionHandler.name(), "ImageExtensionHandler");
    }
}
