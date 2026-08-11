//! OFD 解密器门面。
//!
//! 对应 Java: org.ofdrw.crypto.OFDDecryptor

use easyofd_core::{OfdError, OfdResult};

use crate::container_file_filter::{ContainerFileFilter, DefaultContainerFileFilter};
use crate::decrypt_result::DecryptResult;

/// OFD 解密器，提供 OFD 文档解密的统一入口。
///
/// 对应 Java: `org.ofdrw.crypto.OFDDecryptor`
///
/// 封装已有的 [`crate::decrypt_ofd`] 函数，提供面向对象的 API，
/// 支持自定义过滤器和解密回调。
pub struct OfdDecryptor {
    /// 文件过滤器。
    filter: Box<dyn ContainerFileFilter>,
}

impl OfdDecryptor {
    /// 创建使用默认过滤器的解密器。
    ///
    /// 对应 Java: `OFDDecryptor()`
    #[must_use]
    pub fn new() -> Self {
        Self {
            filter: Box::new(DefaultContainerFileFilter),
        }
    }

    /// 创建使用自定义过滤器的解密器。
    #[must_use]
    pub fn with_filter(filter: Box<dyn ContainerFileFilter>) -> Self {
        Self { filter }
    }

    /// 解密 OFD 文档。
    ///
    /// 对应 Java: `OFDDecryptor.decrypt(Path)`
    ///
    /// # 参数
    ///
    /// - `input`: 加密后的 OFD 文件字节流。
    /// - `key`: SM4 解密密钥（16 字节）。
    ///
    /// # 错误
    ///
    /// ZIP 格式错误、加密描述缺失或 SM4 解密失败时返回错误。
    pub fn decrypt(&self, input: &[u8], key: &[u8; 16]) -> OfdResult<Vec<u8>> {
        crate::decrypt_ofd(input, key)
    }

    /// 解密 OFD 文档并返回逐条解密结果。
    ///
    /// 对每个 ZIP 条目调用过滤器判断是否需要处理。
    ///
    /// # 参数
    ///
    /// - `input`: 加密后的 OFD 文件字节流。
    /// - `key`: SM4 解密密钥（16 字节）。
    ///
    /// # 错误
    ///
    /// ZIP 格式错误或解密失败时返回错误。
    pub fn decrypt_with_details(
        &self,
        input: &[u8],
        key: &[u8; 16],
    ) -> OfdResult<Vec<DecryptResult>> {
        use std::io::{Cursor, Read};

        let reader = Cursor::new(input);
        let mut archive =
            zip::ZipArchive::new(reader).map_err(|e| OfdError::Zip(format!("{e}")))?;

        // 读取加密描述
        let enc_map = crate::ofd_encrypt::parse_encrypt_info_xml_from_archive(&mut archive)?;

        let mut results = Vec::new();
        for i in 0..archive.len() {
            let mut file = archive
                .by_index(i)
                .map_err(|e| OfdError::Zip(format!("{e}")))?;
            let name = file.name().to_owned();

            if !self.filter.should_process(&name) {
                continue;
            }

            let mut content = Vec::new();
            file.read_to_end(&mut content).map_err(OfdError::Io)?;

            if let Some(original) = enc_map.get(&name) {
                let decrypted = crate::sm4::decrypt(key, &content)?;
                results.push(DecryptResult::decrypted(original.clone(), decrypted));
            } else {
                results.push(DecryptResult::plaintext(name, content));
            }
        }

        Ok(results)
    }

    /// 获取过滤器引用。
    #[must_use]
    pub fn filter(&self) -> &dyn ContainerFileFilter {
        self.filter.as_ref()
    }
}

impl Default for OfdDecryptor {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for OfdDecryptor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OfdDecryptor")
            .field("filter", &self.filter)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sm4;
    use std::io::{Cursor, Write};

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
            zip.write_all(b"<?xml version=\"1.0\"?><OFD><DocBody></DocBody></OFD>")
                .unwrap();

            zip.start_file("Doc.xml", options).unwrap();
            zip.write_all(b"<?xml version=\"1.0\"?><Doc><Page></Page></Doc>")
                .unwrap();

            zip.finish().unwrap();
        }
        buf
    }

    #[test]
    fn test_ofd_decryptor_new() {
        let decryptor = OfdDecryptor::new();
        assert!(decryptor.filter().should_process("OFD.xml"));
        assert!(!decryptor.filter().should_process("EncryptInfo.xml"));
    }

    #[test]
    fn test_ofd_decryptor_decrypt() {
        let ofd = create_test_ofd();
        let encrypted = crate::encrypt_ofd(&ofd, &TEST_KEY).unwrap();

        let decryptor = OfdDecryptor::new();
        let decrypted = decryptor.decrypt(&encrypted, &TEST_KEY).unwrap();

        let reader = Cursor::new(&decrypted);
        let mut archive = zip::ZipArchive::new(reader).unwrap();
        assert!(archive.by_name("OFD.xml").is_ok());
    }

    #[test]
    fn test_ofd_decryptor_wrong_key() {
        let ofd = create_test_ofd();
        let encrypted = crate::encrypt_ofd(&ofd, &TEST_KEY).unwrap();

        let decryptor = OfdDecryptor::new();
        let wrong_key = [0xFFu8; sm4::KEY_SIZE];
        assert!(decryptor.decrypt(&encrypted, &wrong_key).is_err());
    }
}
