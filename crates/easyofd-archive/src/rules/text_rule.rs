//! 文本合规规则。
//!
//! 对应 Java: `org.ofdrw.archive.check.rule.TextRule`
//!
//! 校验 OFD 页面中的文本对象（`TextObject`）是否具备必需属性：
//! - 文本内容（`TextCode`）不能为空
//! - 必须指定字体名称（`Font` 属性）
//! - 字号（`Size`）必须大于零

use std::io::Cursor;

use quick_xml::Reader as XmlReader;
use quick_xml::events::Event;

use super::{ComplianceRule, RuleResult};

/// 文本合规规则。
///
/// 校验页面 XML 中的文本对象是否具备完整属性。
#[derive(Debug, Clone, Copy)]
pub struct TextRule;

impl ComplianceRule for TextRule {
    fn name(&self) -> &'static str {
        "TextRule"
    }

    fn check(&self, entries: &[(String, Vec<u8>)]) -> RuleResult {
        let mut violations = Vec::new();

        for (name, data) in entries {
            if !is_page_xml(name) {
                continue;
            }
            let issues = check_text_objects(data);
            if !issues.is_empty() {
                violations.push(format!("{name}: {}", issues.join("; ")));
            }
        }

        if violations.is_empty() {
            RuleResult {
                passed: true,
                message: "全部文本对象属性校验通过".into(),
            }
        } else {
            RuleResult {
                passed: false,
                message: format!("文本规则校验失败: {}", violations.join(" | ")),
            }
        }
    }
}

/// 判断文件路径是否为页面 XML。
fn is_page_xml(path: &str) -> bool {
    path.contains("/Pages/")
        && std::path::Path::new(path)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("xml"))
}

/// 检查页面 XML 中的文本对象，返回问题列表。
fn check_text_objects(xml_bytes: &[u8]) -> Vec<String> {
    let mut reader = XmlReader::from_reader(Cursor::new(xml_bytes));
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    let mut issues = Vec::new();
    let mut in_text_object = false;
    let mut has_font = false;
    let mut has_size = false;
    let mut has_text_code = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let tag = e.name();
                let tag_bytes = tag.as_ref();
                if tag_bytes == b"ofd:TextObject" || tag_bytes == b"TextObject" {
                    in_text_object = true;
                    has_font = false;
                    has_size = false;
                    has_text_code = false;
                    // 检查 Font 和 Size 属性
                    for attr in e.attributes().flatten() {
                        let key = attr.key.as_ref();
                        if key == b"Font" {
                            has_font = true;
                        }
                        if key == b"Size" {
                            has_size = true;
                            if let Ok(val) = attr.decoded_and_normalized_value(
                                quick_xml::XmlVersion::Explicit1_0,
                                reader.decoder(),
                            ) {
                                if let Ok(size) = val.parse::<f64>() {
                                    if size <= 0.0 {
                                        issues.push("TextObject 的 Size 必须大于零".into());
                                    }
                                }
                            }
                        }
                    }
                } else if in_text_object
                    && (tag_bytes == b"ofd:TextCode" || tag_bytes == b"TextCode")
                {
                    has_text_code = true;
                }
            }
            Ok(Event::End(ref e)) => {
                let end_tag = e.name();
                let end_tag_bytes = end_tag.as_ref();
                if (end_tag_bytes == b"ofd:TextObject" || end_tag_bytes == b"TextObject")
                    && in_text_object
                {
                    if !has_font {
                        issues.push("TextObject 缺少 Font 属性".into());
                    }
                    if !has_size {
                        issues.push("TextObject 缺少 Size 属性".into());
                    }
                    if !has_text_code {
                        issues.push("TextObject 缺少 TextCode 子元素".into());
                    }
                    in_text_object = false;
                }
            }
            Ok(Event::Eof) | Err(_) => break,
            _ => {}
        }
        buf.clear();
    }

    issues
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
    fn text_rule_passes_for_valid_text() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8"?>
<ofd:Page xmlns:ofd="http://www.ofdspec.org/2016">
  <ofd:ContentBlock>
    <ofd:TextObject Font="SimSun" Size="12.0">
      <ofd:TextCode>hello</ofd:TextCode>
    </ofd:TextObject>
  </ofd:ContentBlock>
</ofd:Page>"#;
        let entries = make_entries(&[("Doc_0/Pages/Page_0.xml", xml)]);
        let result = TextRule.check(&entries);
        assert!(result.passed, "{}", result.message);
    }

    #[test]
    fn text_rule_fails_missing_font() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8"?>
<ofd:Page xmlns:ofd="http://www.ofdspec.org/2016">
  <ofd:ContentBlock>
    <ofd:TextObject Size="12.0">
      <ofd:TextCode>hello</ofd:TextCode>
    </ofd:TextObject>
  </ofd:ContentBlock>
</ofd:Page>"#;
        let entries = make_entries(&[("Doc_0/Pages/Page_0.xml", xml)]);
        let result = TextRule.check(&entries);
        assert!(!result.passed);
        assert!(result.message.contains("Font"));
    }

    #[test]
    fn text_rule_fails_missing_size() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8"?>
<ofd:Page xmlns:ofd="http://www.ofdspec.org/2016">
  <ofd:ContentBlock>
    <ofd:TextObject Font="SimSun">
      <ofd:TextCode>hello</ofd:TextCode>
    </ofd:TextObject>
  </ofd:ContentBlock>
</ofd:Page>"#;
        let entries = make_entries(&[("Doc_0/Pages/Page_0.xml", xml)]);
        let result = TextRule.check(&entries);
        assert!(!result.passed);
        assert!(result.message.contains("Size"));
    }

    #[test]
    fn text_rule_passes_for_non_page_xml() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8"?>
<ofd:Document xmlns:ofd="http://www.ofdspec.org/2016">
  <ofd:TextObject/>
</ofd:Document>"#;
        // 非页面文件不检查
        let entries = make_entries(&[("Doc_0/Document.xml", xml)]);
        let result = TextRule.check(&entries);
        assert!(result.passed, "{}", result.message);
    }

    #[test]
    fn text_rule_passes_for_empty_entries() {
        let entries = make_entries(&[]);
        let result = TextRule.check(&entries);
        assert!(result.passed, "{}", result.message);
    }
}
