//! 签名合规规则。
//!
//! 对应 Java: `org.ofdrw.archive.check.rule.SignatureRule`
//!
//! 校验 OFD 包中的电子签名是否符合规范：
//! - 签名文件（`Signatures.xml`）如果存在，必须引用有效的签名数据
//! - 签名数据文件必须存在于 OFD 包中

use std::collections::HashSet;
use std::io::Cursor;

use quick_xml::Reader as XmlReader;
use quick_xml::events::Event;

use super::{ComplianceRule, RuleResult};

/// 签名合规规则。
///
/// 校验 OFD 包中的电子签名文件结构是否合法。
#[derive(Debug, Clone, Copy)]
pub struct SignatureRule;

impl ComplianceRule for SignatureRule {
    fn name(&self) -> &'static str {
        "SignatureRule"
    }

    fn check(&self, entries: &[(String, Vec<u8>)]) -> RuleResult {
        let entry_names: HashSet<&str> = entries.iter().map(|(name, _)| name.as_str()).collect();
        let mut violations = Vec::new();

        // 查找所有签名文件
        for (name, data) in entries {
            if !is_signature_file(name) {
                continue;
            }

            // 提取签名引用
            let sig_refs = extract_signature_refs(data);
            for sig_ref in &sig_refs {
                // 签名数据文件应与 Signatures.xml 同目录或在子目录中
                let dir = parent_dir(name);
                let full_path = format!("{dir}/{sig_ref}");
                if !entry_names.contains(full_path.as_str())
                    && !entry_names.contains(sig_ref.as_str())
                {
                    violations.push(format!("{name}: 签名数据 {sig_ref} 不存在"));
                }
            }
        }

        if violations.is_empty() {
            RuleResult {
                passed: true,
                message: "签名合规校验通过".into(),
            }
        } else {
            RuleResult {
                passed: false,
                message: format!("签名规则校验失败: {}", violations.join(" | ")),
            }
        }
    }
}

/// 判断文件路径是否为签名文件。
fn is_signature_file(path: &str) -> bool {
    path.ends_with("Signatures.xml")
}

/// 获取父目录路径。
fn parent_dir(path: &str) -> String {
    match path.rfind('/') {
        Some(pos) => path[..pos].to_string(),
        None => String::new(),
    }
}

/// 从签名 XML 中提取签名数据引用。
fn extract_signature_refs(xml_bytes: &[u8]) -> Vec<String> {
    let mut reader = XmlReader::from_reader(Cursor::new(xml_bytes));
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    let mut refs = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e) | Event::Empty(ref e)) => {
                let tag = e.name();
                let tag_bytes = tag.as_ref();
                if tag_bytes == b"ofd:Signature" || tag_bytes == b"Signature" {
                    for attr in e.attributes().flatten() {
                        let key = attr.key.as_ref();
                        if key == b"BaseLoc" {
                            if let Ok(val) = attr.decoded_and_normalized_value(
                                quick_xml::XmlVersion::Explicit1_0,
                                reader.decoder(),
                            ) {
                                refs.push(val.to_string());
                            }
                        }
                    }
                }
            }
            Ok(Event::Eof) | Err(_) => break,
            _ => {}
        }
        buf.clear();
    }

    refs
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 构建条目列表辅助函数。
    fn make_entries(pairs: &[(&str, &[u8])]) -> Vec<(String, Vec<u8>)> {
        pairs
            .iter()
            .map(|(name, data)| (name.to_string(), data.to_vec()))
            .collect()
    }

    #[test]
    fn signature_rule_passes_when_no_signatures() {
        let entries = make_entries(&[("OFD.xml", b"<ofd:OFD/>")]);
        let result = SignatureRule.check(&entries);
        assert!(result.passed, "{}", result.message);
    }

    #[test]
    fn signature_rule_passes_when_signature_exists() {
        let sig_xml = br#"<?xml version="1.0" encoding="UTF-8"?>
<ofd:Signatures xmlns:ofd="http://www.ofdspec.org/2016">
  <ofd:Signature ID="1" BaseLoc="Signature_0.xml"/>
</ofd:Signatures>"#;
        let sig_data = b"<SignatureData/>";
        let entries = make_entries(&[
            ("Doc_0/Signatures.xml", sig_xml),
            ("Doc_0/Signature_0.xml", sig_data),
        ]);
        let result = SignatureRule.check(&entries);
        assert!(result.passed, "{}", result.message);
    }

    #[test]
    fn signature_rule_fails_when_signature_data_missing() {
        let sig_xml = br#"<?xml version="1.0" encoding="UTF-8"?>
<ofd:Signatures xmlns:ofd="http://www.ofdspec.org/2016">
  <ofd:Signature ID="1" BaseLoc="Signature_0.xml"/>
</ofd:Signatures>"#;
        let entries = make_entries(&[("Doc_0/Signatures.xml", sig_xml)]);
        let result = SignatureRule.check(&entries);
        assert!(!result.passed);
        assert!(result.message.contains("Signature_0.xml"));
    }

    #[test]
    fn signature_rule_passes_for_empty_entries() {
        let entries = make_entries(&[]);
        let result = SignatureRule.check(&entries);
        assert!(result.passed, "{}", result.message);
    }
}
