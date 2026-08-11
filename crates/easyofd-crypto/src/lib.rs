//! # `easyofd-crypto`
//!
//! OFD 加密基础设施，对应 Java 版 [`ofdrw-crypto`](https://github.com/ofdrw/ofdrw) 子项目。
//!
//! ## 功能
//!
//! - [`encryption`] — 加密描述结构（`ProtectionCaseID`、`CT_EncryptInfo`、`EncryptEntry`）
//! - [`sm4`] — SM4-CBC 对称加密/解密（GB/T 32907-2016）
//! - [`encrypt_ofd`] / [`decrypt_ofd`] — OFD ZIP 级别的加密/解密统一入口
//!
//! ## 加密方案
//!
//! 遵循 GB/T 33190 第 19 章规范，使用 SM4-CBC 模式（128 位密钥 + PKCS#7 填充）。
//! 每个 ZIP 条目独立加密，IV 随机生成并前置于密文。
//!
//! ## 参考
//!
//! - GB/T 33190-2016 电子文件存储与交换格式（第 19 章：文件加密）
//! - GB/T 32907-2016 SM4 分组密码算法

pub mod encryption;
mod ofd_encrypt;
pub mod sm4;

pub use encryption::{CT_EncryptInfo, EncryptEntries, EncryptEntry, ProtectionCaseID};
pub use ofd_encrypt::{decrypt_ofd, encrypt_ofd};

/// 返回模块标识，用于运行时识别。
#[must_use]
pub fn module_name() -> &'static str {
    "easyofd-crypto"
}

#[cfg(test)]
mod tests {
    use super::*;
    use easyofd_core::OfdError;
    use std::io::{Cursor, Read, Write};

    const TEST_KEY: [u8; sm4::KEY_SIZE] = [
        0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF, 0xFE, 0xDC, 0xBA, 0x98, 0x76, 0x54, 0x32,
        0x10,
    ];

    /// 辅助函数：创建一个最小的 OFD ZIP 文件。
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

            zip.start_file("res/image.png", options).unwrap();
            zip.write_all(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A])
                .unwrap();

            zip.finish().unwrap();
        }
        buf
    }

    #[test]
    fn test_module_name() {
        assert_eq!(module_name(), "easyofd-crypto");
    }

    #[test]
    fn test_encrypt_ofd_produces_valid_zip() {
        let ofd = create_test_ofd();
        let encrypted = encrypt_ofd(&ofd, &TEST_KEY).unwrap();

        let reader = Cursor::new(&encrypted);
        let archive = zip::ZipArchive::new(reader);
        assert!(archive.is_ok(), "加密后的输出应为合法 ZIP");

        let mut archive = archive.unwrap();
        assert!(archive.by_name("EncryptInfo.xml").is_ok());
        assert!(archive.by_name("OFD.xml.enc").is_ok());
        assert!(archive.by_name("Doc.xml.enc").is_ok());
        assert!(archive.by_name("res/image.png.enc").is_ok());
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let ofd = create_test_ofd();
        let encrypted = encrypt_ofd(&ofd, &TEST_KEY).unwrap();
        let decrypted = decrypt_ofd(&encrypted, &TEST_KEY).unwrap();

        let reader = Cursor::new(&decrypted);
        let mut archive = zip::ZipArchive::new(reader).unwrap();

        assert!(archive.by_name("OFD.xml").is_ok());
        assert!(archive.by_name("Doc.xml").is_ok());
        assert!(archive.by_name("res/image.png").is_ok());

        let mut ofd_xml = String::new();
        archive
            .by_name("OFD.xml")
            .unwrap()
            .read_to_string(&mut ofd_xml)
            .unwrap();
        assert!(ofd_xml.contains("<OFD>"));
    }

    #[test]
    fn test_decrypt_wrong_key_fails() {
        let ofd = create_test_ofd();
        let encrypted = encrypt_ofd(&ofd, &TEST_KEY).unwrap();

        let wrong_key = [0xFFu8; sm4::KEY_SIZE];
        let result = decrypt_ofd(&encrypted, &wrong_key);
        assert!(result.is_err());
    }

    #[test]
    fn test_encrypt_empty_zip() {
        let mut buf = Vec::new();
        {
            let writer = Cursor::new(&mut buf);
            let zip = zip::ZipWriter::new(writer);
            zip.finish().unwrap();
        }
        let encrypted = encrypt_ofd(&buf, &TEST_KEY).unwrap();
        let reader = Cursor::new(&encrypted);
        let mut archive = zip::ZipArchive::new(reader).unwrap();
        assert!(archive.by_name("EncryptInfo.xml").is_ok());
    }

    #[test]
    fn test_decrypt_non_zip_input() {
        let result = decrypt_ofd(b"not a zip", &TEST_KEY);
        assert!(result.is_err());
        assert!(matches!(result, Err(OfdError::Zip(_))));
    }

    #[test]
    fn test_encrypt_ofd_preserves_directory_structure() {
        let ofd = create_test_ofd();
        let encrypted = encrypt_ofd(&ofd, &TEST_KEY).unwrap();
        let reader = Cursor::new(&encrypted);
        let mut archive = zip::ZipArchive::new(reader).unwrap();

        assert!(
            archive.by_name("res/").is_ok() || archive.by_name("res/image.png.enc").is_ok(),
            "应保留目录结构或加密后的文件"
        );
    }

    #[test]
    fn test_encrypt_info_xml_format() {
        let entries = EncryptEntries {
            entries: vec![
                EncryptEntry::new("a.xml".into(), "a.xml.enc".into()),
                EncryptEntry::new("b.png".into(), "b.png.enc".into()),
            ],
        };
        let xml = ofd_encrypt::build_encrypt_info_xml(&entries);
        assert!(xml.contains("SM4-CBC"));
        assert!(xml.contains("a.xml"));
        assert!(xml.contains("a.xml.enc"));
        assert!(xml.contains("b.png"));
    }

    #[test]
    fn test_escape_xml_attr() {
        assert_eq!(ofd_encrypt::escape_xml_attr("hello"), "hello");
        assert_eq!(ofd_encrypt::escape_xml_attr("a&b"), "a&amp;b");
        assert_eq!(ofd_encrypt::escape_xml_attr("a<b"), "a&lt;b");
        assert_eq!(ofd_encrypt::escape_xml_attr("a\"b"), "a&quot;b");
    }

    #[test]
    fn test_parse_encrypt_info_xml() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<EncryptInfo ProtectionCaseID="SM4-CBC" EncryptInfoCount="2">
  <EncryptEntry Original="OFD.xml" Encrypted="OFD.xml.enc" />
  <EncryptEntry Original="Doc.xml" Encrypted="Doc.xml.enc" />
</EncryptInfo>"#;
        let map = ofd_encrypt::parse_encrypt_info_xml(xml).unwrap();
        assert_eq!(map.get("OFD.xml.enc").unwrap(), "OFD.xml");
        assert_eq!(map.get("Doc.xml.enc").unwrap(), "Doc.xml");
    }

    #[test]
    fn test_parse_encrypt_info_xml_empty() {
        let xml = r#"<?xml version="1.0"?>
<EncryptInfo ProtectionCaseID="SM4-CBC">
</EncryptInfo>"#;
        let result = ofd_encrypt::parse_encrypt_info_xml(xml);
        assert!(result.is_err());
    }

    #[test]
    fn test_encrypt_decrypt_preserves_content() {
        let original_content = b"This is a longer test content for OFD document validation.";
        let mut buf = Vec::new();
        {
            let writer = Cursor::new(&mut buf);
            let mut zip = zip::ZipWriter::new(writer);
            let options = zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Stored);

            zip.start_file("Doc.xml", options).unwrap();
            zip.write_all(original_content).unwrap();
            zip.finish().unwrap();
        }

        let encrypted = encrypt_ofd(&buf, &TEST_KEY).unwrap();
        let decrypted = decrypt_ofd(&encrypted, &TEST_KEY).unwrap();

        let reader = Cursor::new(&decrypted);
        let mut archive = zip::ZipArchive::new(reader).unwrap();
        let mut content = Vec::new();
        archive
            .by_name("Doc.xml")
            .unwrap()
            .read_to_end(&mut content)
            .unwrap();
        assert_eq!(content, original_content);
    }
}
