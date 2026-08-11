//! 图像转换处理器。
//!
//! 对应 Java: org.ofdrw.archive.convert.handler.ImageConvertHandler

use crate::convert::archive_handler::ArchiveHandler;

/// 图像转换处理器。
///
/// 对应 Java: org.ofdrw.archive.convert.handler.ImageConvertHandler
#[derive(Debug, Clone, Copy)]
pub struct ImageConvertHandler;

impl ArchiveHandler for ImageConvertHandler {
    fn name(&self) -> &'static str {
        "ImageConvertHandler"
    }

    fn handle(&self, _entries: &mut Vec<(String, Vec<u8>)>) -> Result<(), String> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn image_convert_handler_name() {
        assert_eq!(ImageConvertHandler.name(), "ImageConvertHandler");
    }
}
