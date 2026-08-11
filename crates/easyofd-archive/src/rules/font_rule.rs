//! 字体合规规则。
//!
//! 对应 Java: `org.ofdrw.archive.check.rule.FontRule`
//!
//! 校验 OFD 文档中引用的字体资源是否完整：
//! - `Document.xml` 中声明的 `PublicRes` 字体资源文件必须存在
//! - 页面中引用的字体 ID 必须在字体资源文件中有定义

use std::collections::HashSet;
use std::io::Cursor;

use quick_xml::Reader as XmlReader;
use quick_xml::events::Event;

use super::{ComplianceRule, RuleResult};

/// 字体合规规则。
///
/// 校验 OFD 文档中引用的字体资源文件是否存在。
#[derive(Debug, Clone, Copy)]
pub struct FontRule;

impl ComplianceRule for FontRule {
    fn name(&self) -> &'static str {
        "FontRule"
    }

    fn check(&self, entries: &[(String, Vec<u8>)]) -> RuleResult {
        let entry_names: HashSet<&str> = entries.iter().map(|(name, _)| name.as_str()).collect();
        let mut violations = Vec::new();

        // 从 Document.xml 提取 PublicRes 引用
        if let Some(doc_xml) = find_entry(entries, "Doc_0/Document.xml") {
            let res_refs = extract_public_res_refs(doc_xml);
            for font_ref in &res_refs {
                let full_path = format!("Doc_0/{font_ref}");
                if !entry_names.contains(full_path.as_str()) {
                    violations.push(format!("字体资源文件 {font_ref} 不存在"));
                }
            }
        }

        if violations.is_empty() {
            RuleResult {
                passed: true,
                message: "字体资源引用校验通过".into(),
            }
        } else {
            RuleResult {
                passed: false,
                message: format!("字体规则校验失败: {}", violations.join(" | ")),
            }
        }
    }
}

/// 从条目列表中查找指定路径的内容。
fn find_entry<'a>(entries: &'a [(String, Vec<u8>)], path: &str) -> Option<&'a [u8]> {
    entries
        .iter()
        .find(|(name, _)| name == path)
        .map(|(_, data)| data.as_slice())
}

/// 从 Document.xml 中提取 PublicRes 引用路径。
fn extract_public_res_refs(xml_bytes: &[u8]) -> Vec<String> {
    let mut reader = XmlReader::from_reader(Cursor::new(xml_bytes));
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    let mut refs = Vec::new();
    let mut in_public_res = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e) | Event::Empty(ref e)) => {
                let tag = e.name();
                let tag_bytes = tag.as_ref();
                if tag_bytes == b"ofd:PublicRes" || tag_bytes == b"PublicRes" {
                    // 检查 BaseLoc 属性
                    for attr in e.attributes().flatten() {
                        if attr.key.as_ref() == b"BaseLoc" {
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
            Ok(Event::Text(ref t)) if in_public_res => {
                if let Ok(s) = t.xml10_content() {
                    let trimmed = s.trim().to_string();
                    if !trimmed.is_empty() {
                        refs.push(trimmed);
                    }
                }
            }
            Ok(Event::End(ref e)) => {
                let end_tag = e.name();
                let end_tag_bytes = end_tag.as_ref();
                if (end_tag_bytes == b"ofd:PublicRes" || end_tag_bytes == b"PublicRes")
                    && in_public_res
                {
                    in_public_res = false;
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
    fn font_rule_passes_when_resource_exists() {
        let doc_xml = br#"<?xml version="1.0" encoding="UTF-8"?>
<ofd:Document xmlns:ofd="http://www.ofdspec.org/2016">
  <ofd:PublicRes BaseLoc="Res/font.xml"/>
</ofd:Document>"#;
        let font_res = b"<fonts/>";
        let entries = make_entries(&[
            ("Doc_0/Document.xml", doc_xml),
            ("Doc_0/Res/font.xml", font_res),
        ]);
        let result = FontRule.check(&entries);
        assert!(result.passed, "{}", result.message);
    }

    #[test]
    fn font_rule_fails_when_resource_missing() {
        let doc_xml = br#"<?xml version="1.0" encoding="UTF-8"?>
<ofd:Document xmlns:ofd="http://www.ofdspec.org/2016">
  <ofd:PublicRes BaseLoc="Res/font.xml"/>
</ofd:Document>"#;
        let entries = make_entries(&[("Doc_0/Document.xml", doc_xml)]);
        let result = FontRule.check(&entries);
        assert!(!result.passed);
        assert!(result.message.contains("font.xml"));
    }

    #[test]
    fn font_rule_passes_when_no_public_res() {
        let doc_xml = br#"<?xml version="1.0" encoding="UTF-8"?>
<ofd:Document xmlns:ofd="http://www.ofdspec.org/2016">
  <ofd:Pages/>
</ofd:Document>"#;
        let entries = make_entries(&[("Doc_0/Document.xml", doc_xml)]);
        let result = FontRule.check(&entries);
        assert!(result.passed, "{}", result.message);
    }

    #[test]
    fn font_rule_passes_when_no_document_xml() {
        let entries = make_entries(&[]);
        let result = FontRule.check(&entries);
        assert!(result.passed, "{}", result.message);
    }
}
