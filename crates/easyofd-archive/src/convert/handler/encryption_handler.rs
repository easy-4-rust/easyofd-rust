//! 加密处理器。
//!
//! 对应 Java: org.ofdrw.archive.convert.handler.EncryptionHandler

use crate::convert::archive_handler::ArchiveHandler;

/// 加密处理器。
///
/// 对应 Java: org.ofdrw.archive.convert.handler.EncryptionHandler
#[derive(Debug, Clone, Default)]
pub struct EncryptionHandler {
    /// 是否有解密器配置。
    has_decryptor: bool,
}

impl EncryptionHandler {
    /// 创建新的加密处理器。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 添加解密器。
    pub fn add_decryptor(&mut self) {
        self.has_decryptor = true;
    }

    /// 是否已配置解密器。
    #[must_use]
    pub fn has_decryptor(&self) -> bool {
        self.has_decryptor
    }
}

impl ArchiveHandler for EncryptionHandler {
    fn name(&self) -> &'static str {
        "EncryptionHandler"
    }

    fn handle(&self, _entries: &mut Vec<(String, Vec<u8>)>) -> Result<(), String> {
        // Phase 1: 空实现，后续可添加解密逻辑
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encryption_handler_name() {
        assert_eq!(EncryptionHandler::new().name(), "EncryptionHandler");
    }

    #[test]
    fn encryption_handler_default_no_decryptor() {
        let handler = EncryptionHandler::new();
        assert!(!handler.has_decryptor());
    }

    #[test]
    fn encryption_handler_add_decryptor() {
        let mut handler = EncryptionHandler::new();
        handler.add_decryptor();
        assert!(handler.has_decryptor());
    }
}
