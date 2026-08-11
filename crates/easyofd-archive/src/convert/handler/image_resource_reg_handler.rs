//! 图像资源注册处理器。
//!
//! 对应 Java: org.ofdrw.archive.convert.handler.ImageResourceRegHandler

use crate::convert::archive_handler::ArchiveHandler;

/// 图像资源注册处理器。
///
/// 对应 Java: org.ofdrw.archive.convert.handler.ImageResourceRegHandler
#[derive(Debug, Clone, Copy)]
pub struct ImageResourceRegHandler;

impl ArchiveHandler for ImageResourceRegHandler {
    fn name(&self) -> &'static str {
        "ImageResourceRegHandler"
    }

    fn handle(&self, _entries: &mut Vec<(String, Vec<u8>)>) -> Result<(), String> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn image_resource_reg_handler_name() {
        assert_eq!(ImageResourceRegHandler.name(), "ImageResourceRegHandler");
    }
}
