//! OFD 文件完整性保护。
//!
//! 对应 Java: `org.ofdrw.archive.check.integrity` 包。
//!
//! 根据 GMT 0099 / GB/T 33190-2016，OFD 包内各文件的摘要值可声明在
//! `OFD.xml` 的 `CheckValue` 元素中。本模块读取 ZIP 归档、解析声明的
//! 摘要条目并计算实际文件哈希，生成完整性报告。

use std::fmt::Write as _;
use std::io::{Cursor, Read};

use easyofd_core::{OfdError, OfdResult};
use quick_xml::Reader as XmlReader;
use quick_xml::events::Event;
use sha2::{Digest as Sha2Digest, Sha256};
use sm3::{Digest, Sm3};

/// 完整性条目 —— 描述一个文件的期望摘要值。
///
/// 对应 Java: `org.ofdrw.archive.check.integrity.IntegrityEntry`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntegrityEntry {
    /// 条目文件的路径（相对于 OFD 包根目录）。
    pub file_path: String,
    /// 摘要算法。
    pub check_method: CheckMethod,
    /// 期望的摘要值（十六进制小写字符串）。
    pub check_value: String,
}

/// 摘要算法枚举。
///
/// 对应 Java: `org.ofdrw.archive.check.integrity.CheckMethod`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckMethod {
    /// 国密 SM3 杂凑算法（GMT 0004）。
    SM3,
    /// SHA-256 杂凑算法。
    SHA256,
}

/// 完整性检查结果。
///
/// 对应 Java: `org.ofdrw.archive.check.integrity.IntegrityReport`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntegrityReport {
    /// 全部检查是否通过。
    pub passed: bool,
    /// 摘要不匹配的文件列表。
    pub failed_files: Vec<String>,
    /// 在 ZIP 中缺失的文件列表。
    pub missing_files: Vec<String>,
}

/// 验证 OFD 完整性 —— 检查 `OFD.xml` 中声明的文件摘要与实际是否一致。
///
/// 解析流程：
/// 1. 打开 ZIP 归档。
/// 2. 解析 `OFD.xml`，提取所有 `CheckValue` 条目。
/// 3. 对每个条目，读取对应文件并计算摘要。
/// 4. 比较声明值与计算值，生成 [`IntegrityReport`]。
///
/// 如果 `OFD.xml` 中没有声明任何 `CheckValue`，则报告通过（无断言 = 无失败）。
///
/// # Errors
///
/// 当 ZIP 或 XML 解析失败时返回错误。
pub fn verify_integrity(ofd_bytes: &[u8]) -> OfdResult<IntegrityReport> {
    let mut archive =
        zip::ZipArchive::new(Cursor::new(ofd_bytes)).map_err(|e| OfdError::Zip(e.to_string()))?;

    let entries = parse_check_values(&mut archive)?;

    // 无声明条目 → 通过
    if entries.is_empty() {
        return Ok(IntegrityReport {
            passed: true,
            failed_files: Vec::new(),
            missing_files: Vec::new(),
        });
    }

    let mut failed_files = Vec::new();
    let mut missing_files = Vec::new();

    for entry in &entries {
        let file_data = if let Ok(data) = read_entry(&mut archive, &entry.file_path) {
            data
        } else {
            missing_files.push(entry.file_path.clone());
            continue;
        };

        let actual = match entry.check_method {
            CheckMethod::SM3 => hex_encode(&compute_sm3(&file_data)),
            CheckMethod::SHA256 => hex_encode(&compute_sha256(&file_data)),
        };

        if actual != entry.check_value {
            failed_files.push(entry.file_path.clone());
        }
    }

    Ok(IntegrityReport {
        passed: failed_files.is_empty() && missing_files.is_empty(),
        failed_files,
        missing_files,
    })
}

// ─── 摘要计算 ────────────────────────────────────────────────────────────────

/// 计算 SM3 摘要，返回 32 字节。
pub(crate) fn compute_sm3(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sm3::new();
    hasher.update(data);
    let result = hasher.finalize();
    let mut output = [0u8; 32];
    output.copy_from_slice(&result);
    output
}

/// 计算 SHA-256 摘要，返回 32 字节。
pub(crate) fn compute_sha256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    let mut output = [0u8; 32];
    output.copy_from_slice(&result);
    output
}

/// 将字节数组编码为十六进制小写字符串。
pub(crate) fn hex_encode(bytes: &[u8]) -> String {
    bytes
        .iter()
        .fold(String::with_capacity(bytes.len() * 2), |mut acc, b| {
            let _ = write!(acc, "{b:02x}");
            acc
        })
}

// ─── XML 解析 ────────────────────────────────────────────────────────────────

/// 解析 `OFD.xml` 中的 `CheckValue` 条目。
fn parse_check_values<R: Read + std::io::Seek>(
    archive: &mut zip::ZipArchive<R>,
) -> OfdResult<Vec<IntegrityEntry>> {
    let xml_bytes = read_entry(archive, "OFD.xml")?;
    let mut reader = XmlReader::from_reader(Cursor::new(&xml_bytes));
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    let mut entries = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) if e.name().as_ref() == b"ofd:CheckValue" => {
                let mut file_path = String::new();
                let mut method = CheckMethod::SM3;
                for attr in e.attributes().flatten() {
                    let value = attr
                        .decoded_and_normalized_value(
                            quick_xml::XmlVersion::Explicit1_0,
                            reader.decoder(),
                        )
                        .unwrap_or_default();
                    match attr.key.as_ref() {
                        b"FileLoc" => file_path = value.to_string(),
                        b"HashMethod" => {
                            method = if value.contains("SHA256") || value.contains("sha256") {
                                CheckMethod::SHA256
                            } else {
                                CheckMethod::SM3
                            };
                        }
                        _ => {}
                    }
                }
                if !file_path.is_empty() {
                    // 读取 CheckValue 元素的文本内容
                    let mut check_value = String::new();
                    loop {
                        match reader.read_event_into(&mut buf) {
                            Ok(Event::Text(ref t)) => {
                                check_value = t
                                    .xml10_content()
                                    .map(|c| c.into_owned())
                                    .unwrap_or_default();
                            }
                            Ok(Event::End(_) | Event::Eof) => break,
                            _ => {}
                        }
                        buf.clear();
                    }
                    entries.push(IntegrityEntry {
                        file_path,
                        check_method: method,
                        check_value,
                    });
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(OfdError::Xml(format!("OFD.xml: {e}"))),
            _ => {}
        }
        buf.clear();
    }
    Ok(entries)
}

/// 从 ZIP 归档中读取指定条目的全部内容。
fn read_entry<R: Read + std::io::Seek>(
    archive: &mut zip::ZipArchive<R>,
    name: &str,
) -> OfdResult<Vec<u8>> {
    let mut file = archive
        .by_name(name)
        .map_err(|e| OfdError::Zip(format!("{name}: {e}")))?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).map_err(OfdError::Io)?;
    Ok(buf)
}

// ─── 测试 ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    /// 构建一个包含自定义 `OFD.xml` 的最小 ZIP 归档。
    fn build_zip(ofd_xml: &str) -> Vec<u8> {
        let mut buf = Cursor::new(Vec::new());
        {
            let mut zip = zip::ZipWriter::new(&mut buf);
            let options = zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Stored);

            zip.start_file("OFD.xml", options).unwrap();
            zip.write_all(ofd_xml.as_bytes()).unwrap();

            zip.start_file("Doc_0/Document.xml", options).unwrap();
            zip.write_all(b"<ofd:Document/>").unwrap();

            zip.finish().unwrap();
        }
        buf.into_inner()
    }

    #[test]
    fn test_no_check_values_passes() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<ofd:OFD xmlns:ofd="http://www.ofdspec.org/2016" Version="1.2">
  <ofd:DocBody><ofd:DocRoot>Doc_0/Document.xml</ofd:DocRoot></ofd:DocBody>
</ofd:OFD>"#;
        let bytes = build_zip(xml);
        let report = verify_integrity(&bytes).unwrap();
        assert!(report.passed);
        assert!(report.failed_files.is_empty());
        assert!(report.missing_files.is_empty());
    }

    #[test]
    fn test_sm3_check_value_passes() {
        // 先计算 Doc_0/Document.xml 的 SM3 摘要
        let doc_data = b"<ofd:Document/>";
        let hash = compute_sm3(doc_data);
        let hex = hex_encode(&hash);

        let xml = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<ofd:OFD xmlns:ofd="http://www.ofdspec.org/2016" Version="1.2">
  <ofd:DocBody>
    <ofd:DocRoot>Doc_0/Document.xml</ofd:DocRoot>
    <ofd:CheckValue FileLoc="Doc_0/Document.xml" HashMethod="SM3">{hex}</ofd:CheckValue>
  </ofd:DocBody>
</ofd:OFD>"#
        );
        let bytes = build_zip(&xml);
        let report = verify_integrity(&bytes).unwrap();
        assert!(report.passed, "SM3 check should pass for correct hash");
    }

    #[test]
    fn test_sm3_check_value_fails_on_corruption() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<ofd:OFD xmlns:ofd="http://www.ofdspec.org/2016" Version="1.2">
  <ofd:DocBody>
    <ofd:DocRoot>Doc_0/Document.xml</ofd:DocRoot>
    <ofd:CheckValue FileLoc="Doc_0/Document.xml" HashMethod="SM3">0000000000000000000000000000000000000000000000000000000000000000</ofd:CheckValue>
  </ofd:DocBody>
</ofd:OFD>"#;
        let bytes = build_zip(xml);
        let report = verify_integrity(&bytes).unwrap();
        assert!(!report.passed);
        assert!(
            report
                .failed_files
                .contains(&"Doc_0/Document.xml".to_string())
        );
    }

    #[test]
    fn test_missing_file_detected() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<ofd:OFD xmlns:ofd="http://www.ofdspec.org/2016" Version="1.2">
  <ofd:DocBody>
    <ofd:DocRoot>Doc_0/Document.xml</ofd:DocRoot>
    <ofd:CheckValue FileLoc="Doc_0/NonExistent.xml" HashMethod="SM3">abcdef</ofd:CheckValue>
  </ofd:DocBody>
</ofd:OFD>"#;
        let bytes = build_zip(xml);
        let report = verify_integrity(&bytes).unwrap();
        assert!(!report.passed);
        assert!(
            report
                .missing_files
                .contains(&"Doc_0/NonExistent.xml".to_string())
        );
    }

    #[test]
    fn test_sha256_check_value_passes() {
        let doc_data = b"<ofd:Document/>";
        let hash = compute_sha256(doc_data);
        let hex = hex_encode(&hash);

        let xml = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<ofd:OFD xmlns:ofd="http://www.ofdspec.org/2016" Version="1.2">
  <ofd:DocBody>
    <ofd:DocRoot>Doc_0/Document.xml</ofd:DocRoot>
    <ofd:CheckValue FileLoc="Doc_0/Document.xml" HashMethod="SHA256">{hex}</ofd:CheckValue>
  </ofd:DocBody>
</ofd:OFD>"#
        );
        let bytes = build_zip(&xml);
        let report = verify_integrity(&bytes).unwrap();
        assert!(report.passed, "SHA256 check should pass for correct hash");
    }

    #[test]
    fn test_sha256_check_value_fails_on_corruption() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<ofd:OFD xmlns:ofd="http://www.ofdspec.org/2016" Version="1.2">
  <ofd:DocBody>
    <ofd:DocRoot>Doc_0/Document.xml</ofd:DocRoot>
    <ofd:CheckValue FileLoc="Doc_0/Document.xml" HashMethod="SHA256">0000000000000000000000000000000000000000000000000000000000000000</ofd:CheckValue>
  </ofd:DocBody>
</ofd:OFD>"#;
        let bytes = build_zip(xml);
        let report = verify_integrity(&bytes).unwrap();
        assert!(!report.passed);
        assert!(
            report
                .failed_files
                .contains(&"Doc_0/Document.xml".to_string())
        );
    }

    #[test]
    fn test_invalid_zip_returns_error() {
        let result = verify_integrity(b"not a zip");
        assert!(result.is_err());
    }

    #[test]
    fn test_sm3_known_vector() {
        // SM3("abc") 的标准测试向量
        let hash = compute_sm3(b"abc");
        let hex = hex_encode(&hash);
        assert_eq!(
            hex,
            "66c7f0f462eeedd9d1f2d46bdc10e4e24167c4875cf2f7a2297da02b8f4ba8e0"
        );
    }

    #[test]
    fn test_sha256_known_vector() {
        // SHA-256("") 的标准测试向量
        let hash = compute_sha256(b"");
        let hex = hex_encode(&hash);
        assert_eq!(
            hex,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn test_sha256_known_vector_abc() {
        // SHA-256("abc") 的标准测试向量
        let hash = compute_sha256(b"abc");
        let hex = hex_encode(&hash);
        assert_eq!(
            hex,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn test_integrity_report_debug_clone() {
        let report = IntegrityReport {
            passed: true,
            failed_files: vec!["a.xml".into()],
            missing_files: vec![],
        };
        let cloned = report.clone();
        assert!(format!("{report:?}").contains("IntegrityReport"));
        assert_eq!(report, cloned);
    }

    #[test]
    fn test_integrity_entry_debug_clone_eq() {
        let entry = IntegrityEntry {
            file_path: "test.xml".into(),
            check_method: CheckMethod::SM3,
            check_value: "abc".into(),
        };
        let cloned = entry.clone();
        assert!(format!("{entry:?}").contains("IntegrityEntry"));
        assert_eq!(entry, cloned);
    }

    #[test]
    fn test_check_method_debug_clone_eq() {
        let m = CheckMethod::SHA256;
        let cloned = m;
        assert!(format!("{m:?}").contains("SHA256"));
        assert_eq!(m, cloned);
        assert_ne!(CheckMethod::SM3, CheckMethod::SHA256);
    }
}
