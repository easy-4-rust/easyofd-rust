//! OFD ZIP 级别的加密/解密。

use easyofd_core::{OfdError, OfdResult};
use std::io::{Cursor, Read, Write};
use zip::CompressionMethod;
use zip::write::SimpleFileOptions;

use crate::encryption::{EncryptEntries, EncryptEntry};
use crate::sm4;

/// 加密元数据在 OFD ZIP 中的路径。
const ENCRYPT_INFO_PATH: &str = "EncryptInfo.xml";

/// 对 OFD ZIP 中所有内容条目使用 SM4-CBC 加密。
///
/// 输入为完整的 OFD 文件字节流（ZIP 格式），输出为加密后的 OFD 文件字节流。
/// 加密后的 ZIP 包含：
/// - 原始 ZIP 中每个条目的加密版本（路径后缀 `.enc`）
/// - `EncryptInfo.xml` 加密描述文件（明文）
///
/// 对应 Java: `org.ofdrw.crypto.OFDEncryptor`
///
/// # 错误
///
/// - ZIP 解析失败时返回 [`OfdError::Zip`]。
/// - 条目读写失败时返回 [`OfdError::Io`]。
/// - SM4 加密失败时返回 [`OfdError::Conversion`]。
pub fn encrypt_ofd(input: &[u8], key: &[u8; sm4::KEY_SIZE]) -> OfdResult<Vec<u8>> {
    let reader = Cursor::new(input);
    let mut archive = zip::ZipArchive::new(reader).map_err(|e| OfdError::Zip(format!("{e}")))?;

    let mut output = Vec::new();
    {
        let writer = Cursor::new(&mut output);
        let mut zip_out = zip::ZipWriter::new(writer);
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);

        let mut entries = EncryptEntries::new();

        for i in 0..archive.len() {
            let mut file = archive
                .by_index(i)
                .map_err(|e| OfdError::Zip(format!("{e}")))?;

            let name = file.name().to_owned();

            // 跳过目录条目和已有的加密描述文件
            if name.ends_with('/') || name == ENCRYPT_INFO_PATH {
                let mut content = Vec::new();
                file.read_to_end(&mut content).map_err(OfdError::Io)?;
                zip_out
                    .start_file(&name, options)
                    .map_err(|e| OfdError::Zip(format!("{e}")))?;
                zip_out.write_all(&content).map_err(OfdError::Io)?;
                continue;
            }

            // 读取原始内容并加密
            let mut content = Vec::new();
            file.read_to_end(&mut content).map_err(OfdError::Io)?;

            let encrypted = sm4::encrypt(key, &content)?;
            let enc_name = format!("{name}.enc");

            zip_out
                .start_file(&enc_name, options)
                .map_err(|e| OfdError::Zip(format!("{e}")))?;
            zip_out.write_all(&encrypted).map_err(OfdError::Io)?;

            entries.push(EncryptEntry::new(name.clone(), enc_name));
        }

        // 写入加密描述文件（明文）
        let encrypt_info_xml = build_encrypt_info_xml(&entries);
        zip_out
            .start_file(ENCRYPT_INFO_PATH, options)
            .map_err(|e| OfdError::Zip(format!("{e}")))?;
        zip_out
            .write_all(encrypt_info_xml.as_bytes())
            .map_err(OfdError::Io)?;

        zip_out
            .finish()
            .map_err(|e| OfdError::Zip(format!("{e}")))?;
    }

    Ok(output)
}

/// 对 OFD ZIP 中已加密的条目使用 SM4-CBC 解密。
///
/// 输入为加密后的 OFD 文件字节流（ZIP 格式），输出为解密后的 OFD 文件字节流。
/// 解密逻辑：
/// 1. 读取 `EncryptInfo.xml` 获取明密文路径映射
/// 2. 对每个 `.enc` 条目进行 SM4-CBC 解密
/// 3. 将解密后的内容以原始路径写入新 ZIP
///
/// 对应 Java: `org.ofdrw.crypto.OFDDecryptor`
///
/// # 错误
///
/// - ZIP 解析失败时返回 [`OfdError::Zip`]。
/// - 加密描述文件缺失时返回 [`OfdError::InvalidDocument`]。
/// - SM4 解密失败时返回 [`OfdError::Conversion`]。
pub fn decrypt_ofd(input: &[u8], key: &[u8; sm4::KEY_SIZE]) -> OfdResult<Vec<u8>> {
    let reader = Cursor::new(input);
    let mut archive = zip::ZipArchive::new(reader).map_err(|e| OfdError::Zip(format!("{e}")))?;

    // 读取加密描述文件，构建路径映射
    let enc_map = read_encrypt_info(&mut archive)?;

    let mut output = Vec::new();
    {
        let writer = Cursor::new(&mut output);
        let mut zip_out = zip::ZipWriter::new(writer);
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);

        for i in 0..archive.len() {
            let mut file = archive
                .by_index(i)
                .map_err(|e| OfdError::Zip(format!("{e}")))?;

            let name = file.name().to_owned();

            // 跳过目录条目和加密描述文件
            if name.ends_with('/') || name == ENCRYPT_INFO_PATH {
                let mut content = Vec::new();
                file.read_to_end(&mut content).map_err(OfdError::Io)?;
                zip_out
                    .start_file(&name, options)
                    .map_err(|e| OfdError::Zip(format!("{e}")))?;
                zip_out.write_all(&content).map_err(OfdError::Io)?;
                continue;
            }

            let mut content = Vec::new();
            file.read_to_end(&mut content).map_err(OfdError::Io)?;

            // 查找对应的原始路径
            if let Some(original) = enc_map.get(&name) {
                let decrypted = sm4::decrypt(key, &content)?;
                zip_out
                    .start_file(original, options)
                    .map_err(|e| OfdError::Zip(format!("{e}")))?;
                zip_out.write_all(&decrypted).map_err(OfdError::Io)?;
            } else {
                // 非加密条目直接复制
                zip_out
                    .start_file(&name, options)
                    .map_err(|e| OfdError::Zip(format!("{e}")))?;
                zip_out.write_all(&content).map_err(OfdError::Io)?;
            }
        }

        zip_out
            .finish()
            .map_err(|e| OfdError::Zip(format!("{e}")))?;
    }

    Ok(output)
}

/// 构建 `EncryptInfo.xml` 内容。
pub(crate) fn build_encrypt_info_xml(entries: &EncryptEntries) -> String {
    use std::fmt::Write as _;

    let mut xml = String::from(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
    xml.push('\n');
    let _ = writeln!(
        xml,
        "<EncryptInfo ProtectionCaseID=\"SM4-CBC\" EncryptInfoCount=\"{}\">",
        entries.len()
    );
    for entry in &entries.entries {
        let _ = writeln!(
            xml,
            "  <EncryptEntry Original=\"{}\" Encrypted=\"{}\" />",
            escape_xml_attr(&entry.original_path),
            escape_xml_attr(&entry.encrypted_path),
        );
    }
    xml.push_str("</EncryptInfo>\n");
    xml
}

/// 简单 XML 属性转义。
pub(crate) fn escape_xml_attr(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// 从 ZIP 中读取 `EncryptInfo.xml` 并解析为加密路径映射表。
fn read_encrypt_info<R: Read + std::io::Seek>(
    archive: &mut zip::ZipArchive<R>,
) -> OfdResult<std::collections::HashMap<String, String>> {
    parse_encrypt_info_xml_from_archive(archive)
}

/// 从 ZIP 归档中读取并解析 `EncryptInfo.xml`。
///
/// 供 `OfdDecryptor` 等外部模块使用。
pub fn parse_encrypt_info_xml_from_archive<R: Read + std::io::Seek>(
    archive: &mut zip::ZipArchive<R>,
) -> OfdResult<std::collections::HashMap<String, String>> {
    let mut file = archive
        .by_name(ENCRYPT_INFO_PATH)
        .map_err(|_| OfdError::InvalidDocument("加密描述文件 EncryptInfo.xml 缺失".into()))?;

    let mut content = String::new();
    file.read_to_string(&mut content).map_err(OfdError::Io)?;

    parse_encrypt_info_xml(&content)
}

/// 解析 `EncryptInfo.xml` 的简化实现（基于字符串解析，避免引入 XML 依赖）。
pub(crate) fn parse_encrypt_info_xml(
    xml: &str,
) -> OfdResult<std::collections::HashMap<String, String>> {
    let mut map = std::collections::HashMap::new();

    for line in xml.lines() {
        let trimmed = line.trim();
        if let Some(original) = extract_attr(trimmed, "Original") {
            if let Some(encrypted) = extract_attr(trimmed, "Encrypted") {
                map.insert(encrypted, original);
            }
        }
    }

    if map.is_empty() {
        return Err(OfdError::InvalidDocument(
            "EncryptInfo.xml 中未找到有效的加密条目".into(),
        ));
    }

    Ok(map)
}

/// 从 XML 行中提取指定属性值。
fn extract_attr(line: &str, attr_name: &str) -> Option<String> {
    let pattern = format!("{attr_name}=\"");
    let start = line.find(&pattern)? + pattern.len();
    let end = line[start..].find('"')? + start;
    Some(line[start..end].to_owned())
}
