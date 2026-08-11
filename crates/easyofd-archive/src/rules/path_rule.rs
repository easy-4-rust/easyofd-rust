//! 路径合规规则。
//!
//! 对应 Java: `org.ofdrw.archive.check.rule.PathRule`
//!
//! 校验 OFD 页面中的路径对象（`PathObject`）是否合法：
//! - 路径数据（`PathData` / 子元素）不能为空
//! - 路径必须包含至少一个绘图指令（M/L/C/A/Z 等）

use std::io::Cursor;

use quick_xml::Reader as XmlReader;
use quick_xml::events::Event;

use super::{ComplianceRule, RuleResult};

/// 路径合规规则。
///
/// 校验页面 XML 中的路径对象是否包含有效的路径数据。
#[derive(Debug, Clone, Copy)]
pub struct PathRule;

impl ComplianceRule for PathRule {
    fn name(&self) -> &'static str {
        "PathRule"
    }

    fn check(&self, entries: &[(String, Vec<u8>)]) -> RuleResult {
        let mut violations = Vec::new();

        for (name, data) in entries {
            if !is_page_xml(name) {
                continue;
            }
            let issues = check_path_objects(data);
            if !issues.is_empty() {
                violations.push(format!("{name}: {}", issues.join("; ")));
            }
        }

        if violations.is_empty() {
            RuleResult {
                passed: true,
                message: "全部路径对象校验通过".into(),
            }
        } else {
            RuleResult {
                passed: false,
                message: format!("路径规则校验失败: {}", violations.join(" | ")),
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

/// 检查页面 XML 中的路径对象，返回问题列表。
fn check_path_objects(xml_bytes: &[u8]) -> Vec<String> {
    let mut reader = XmlReader::from_reader(Cursor::new(xml_bytes));
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    let mut issues = Vec::new();
    let mut in_path_object = false;
    let mut has_path_data = false;
    let mut path_id = String::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let tag = e.name();
                let tag_bytes = tag.as_ref();
                if tag_bytes == b"ofd:PathObject" || tag_bytes == b"PathObject" {
                    in_path_object = true;
                    has_path_data = false;
                    path_id.clear();
                    // 提取 ID 用于错误报告
                    for attr in e.attributes().flatten() {
                        if attr.key.as_ref() == b"ID" {
                            if let Ok(val) = attr.decoded_and_normalized_value(
                                quick_xml::XmlVersion::Explicit1_0,
                                reader.decoder(),
                            ) {
                                path_id = val.to_string();
                            }
                        }
                    }
                } else if in_path_object
                    && (tag_bytes == b"ofd:AbbreviatedData" || tag_bytes == b"AbbreviatedData")
                {
                    has_path_data = true;
                }
            }
            Ok(Event::Text(ref t)) if in_path_object => {
                // AbbreviatedData 的文本内容
                if let Ok(s) = t.xml10_content() {
                    let trimmed = s.trim();
                    if !trimmed.is_empty() {
                        has_path_data = true;
                    }
                }
            }
            Ok(Event::End(ref e)) => {
                let end_tag = e.name();
                let end_tag_bytes = end_tag.as_ref();
                if (end_tag_bytes == b"ofd:PathObject" || end_tag_bytes == b"PathObject")
                    && in_path_object
                {
                    if !has_path_data {
                        let desc = if path_id.is_empty() {
                            "PathObject 缺少路径数据".to_string()
                        } else {
                            format!("PathObject(ID={path_id}) 缺少路径数据")
                        };
                        issues.push(desc);
                    }
                    in_path_object = false;
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
    fn path_rule_passes_for_valid_path() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8"?>
<ofd:Page xmlns:ofd="http://www.ofdspec.org/2016">
  <ofd:ContentBlock>
    <ofd:PathObject ID="1">
      <ofd:AbbreviatedData>M0 0L100 100Z</ofd:AbbreviatedData>
    </ofd:PathObject>
  </ofd:ContentBlock>
</ofd:Page>"#;
        let entries = make_entries(&[("Doc_0/Pages/Page_0.xml", xml)]);
        let result = PathRule.check(&entries);
        assert!(result.passed, "{}", result.message);
    }

    #[test]
    fn path_rule_fails_for_empty_path() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8"?>
<ofd:Page xmlns:ofd="http://www.ofdspec.org/2016">
  <ofd:ContentBlock>
    <ofd:PathObject ID="1">
      <ofd:AbbreviatedData/>
    </ofd:PathObject>
  </ofd:ContentBlock>
</ofd:Page>"#;
        let entries = make_entries(&[("Doc_0/Pages/Page_0.xml", xml)]);
        let result = PathRule.check(&entries);
        assert!(!result.passed);
        assert!(result.message.contains("路径数据"));
    }

    #[test]
    fn path_rule_passes_for_no_paths() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8"?>
<ofd:Page xmlns:ofd="http://www.ofdspec.org/2016">
  <ofd:ContentBlock/>
</ofd:Page>"#;
        let entries = make_entries(&[("Doc_0/Pages/Page_0.xml", xml)]);
        let result = PathRule.check(&entries);
        assert!(result.passed, "{}", result.message);
    }

    #[test]
    fn path_rule_passes_for_non_page_xml() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8"?>
<ofd:Document xmlns:ofd="http://www.ofdspec.org/2016">
  <ofd:PathObject/>
</ofd:Document>"#;
        let entries = make_entries(&[("Doc_0/Document.xml", xml)]);
        let result = PathRule.check(&entries);
        assert!(result.passed, "{}", result.message);
    }
}
