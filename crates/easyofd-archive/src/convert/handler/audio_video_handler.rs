//! 音视频处理器。
//!
//! 对应 Java: org.ofdrw.archive.convert.handler.AudioVideoHandler

use crate::convert::archive_handler::ArchiveHandler;

/// 音视频处理器。
///
/// 对应 Java: org.ofdrw.archive.convert.handler.AudioVideoHandler
#[derive(Debug, Clone, Copy)]
pub struct AudioVideoHandler;

impl ArchiveHandler for AudioVideoHandler {
    fn name(&self) -> &'static str {
        "AudioVideoHandler"
    }

    fn handle(&self, _entries: &mut Vec<(String, Vec<u8>)>) -> Result<(), String> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn audio_video_handler_name() {
        assert_eq!(AudioVideoHandler.name(), "AudioVideoHandler");
    }
}
