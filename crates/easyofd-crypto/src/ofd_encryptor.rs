//! OFD 加密器门面。
//!
//! 对应 Java: org.ofdrw.crypto.OFDEncryptor

use easyofd_core::OfdResult;

use crate::container_file_filter::{ContainerFileFilter, DefaultContainerFileFilter};

/// OFD 加密器，提供 OFD 文档加密的统一入口。
///
/// 对应 Java: `org.ofdrw.crypto.OFDEncryptor`
///
/// 封装已有的 [`crate::encrypt_ofd`] 函数，提供面向对象的 API，
/// 支持自定义过滤器。
pub struct OfdEncryptor {
    /// 文件过滤器。
    filter: Box<dyn ContainerFileFilter>,
}

impl OfdEncryptor {
    /// 创建使用默认过滤器的加密器。
    ///
    /// 对应 Java: `OFDEncryptor()`
    #[must_use]
    pub fn new() -> Self {
        Self {
            filter: Box::new(DefaultContainerFileFilter),
        }
    }

    /// 创建使用自定义过滤器的加密器。
    #[must_use]
    pub fn with_filter(filter: Box<dyn ContainerFileFilter>) -> Self {
        Self { filter }
    }

    /// 加密 OFD 文档。
    ///
    /// 对应 Java: `OFDEncryptor.encrypt(Path)`
    ///
    /// # 参数
    ///
    /// - `input`: 原始 OFD 文件字节流。
    /// - `key`: SM4 加密密钥（16 字节）。
    ///
    /// # 错误
    ///
    /// ZIP 格式错误或 SM4 加密失败时返回错误。
    pub fn encrypt(&self, input: &[u8], key: &[u8; 16]) -> OfdResult<Vec<u8>> {
        crate::encrypt_ofd(input, key)
    }

    /// 获取过滤器引用。
    #[must_use]
    pub fn filter(&self) -> &dyn ContainerFileFilter {
        self.filter.as_ref()
    }
}

impl Default for OfdEncryptor {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for OfdEncryptor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OfdEncryptor")
            .field("filter", &self.filter)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::OfdDecryptor;
    use crate::sm4;
    use std::io::Cursor;

    const TEST_KEY: [u8; sm4::KEY_SIZE] = [
        0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF, 0xFE, 0xDC, 0xBA, 0x98, 0x76, 0x54, 0x32,
        0x10,
    ];

    fn create_test_ofd() -> Vec<u8> {
        let mut buf = Vec::new();
        {
            let writer = Cursor::new(&mut buf);
            let mut zip = zip::ZipWriter::new(writer);
            let options = zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Stored);

            zip.start_file("OFD.xml", options).unwrap();
            std::io::Write::write_all(&mut (zip), b"<OFD/>").unwrap();

            zip.finish().unwrap();
        }
        buf
    }

    #[test]
    fn test_ofd_encryptor_new() {
        let encryptor = OfdEncryptor::new();
        assert!(encryptor.filter().should_process("Doc.xml"));
    }

    #[test]
    fn test_ofd_encryptor_encrypt() {
        let ofd = create_test_ofd();
        let encryptor = OfdEncryptor::new();
        let encrypted = encryptor.encrypt(&ofd, &TEST_KEY).unwrap();

        let reader = Cursor::new(&encrypted);
        let archive = zip::ZipArchive::new(reader);
        assert!(archive.is_ok());
    }

    #[test]
    fn test_ofd_encryptor_roundtrip() {
        let ofd = create_test_ofd();
        let encryptor = OfdEncryptor::new();
        let decryptor = OfdDecryptor::new();

        let encrypted = encryptor.encrypt(&ofd, &TEST_KEY).unwrap();
        let decrypted = decryptor.decrypt(&encrypted, &TEST_KEY).unwrap();

        let reader = Cursor::new(&decrypted);
        let mut archive = zip::ZipArchive::new(reader).unwrap();
        assert!(archive.by_name("OFD.xml").is_ok());
    }
}
