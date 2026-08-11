//! 图片合规规则。
//!
//! 对应 Java: `org.ofdrw.archive.check.rule.ImageRule`
//!
//! 校验 OFD 页面中引用的图片资源是否存在于归档包内：
//! - `ImageObject` 的 `ResourceID` 必须在资源文件中有对应定义
//! - 引用的图片文件必须存在于 OFD 包中

use std::collections::HashSet;
use std::io::Cursor;

use quick_xml::Reader as XmlReader;
use quick_xml::events::Event;

use super::{ComplianceRule, RuleResult};

/// 图片合规规则。
///
/// 校验页面中引用的图片资源是否在 OFD 包中存在。
#[derive(Debug, Clone, Copy)]
pub struct ImageRule;

impl ComplianceRule for ImageRule {
    fn name(&self) -> &'static str {
        "ImageRule"
    }

    fn check(&self, entries: &[(String, Vec<u8>)]) -> RuleResult {
        let entry_names: HashSet<&str> = entries.iter().map(|(name, _)| name.as_str()).collect();
        let mut violations = Vec::new();

        // 检查所有页面 XML 中的图片引用
        for (name, data) in entries {
            if !is_page_xml(name) {
                continue;
            }
            let refs = extract_image_refs(data);
            for img_ref in &refs {
                // 图片资源通常在 Res/ 目录下
                let full_path = format!("Doc_0/Res/{img_ref}");
                if !entry_names.contains(full_path.as_str())
                    && !entry_names.contains(img_ref.as_str())
                {
                    violations.push(format!("{name}: 引用的图片资源 {img_ref} 不存在"));
                }
            }
        }

        if violations.is_empty() {
            RuleResult {
                passed: true,
                message: "全部图片资源引用校验通过".into(),
            }
        } else {
            RuleResult {
                passed: false,
                message: format!("图片规则校验失败: {}", violations.join(" | ")),
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

/// 从页面 XML 中提取图片资源引用 ID。
fn extract_image_refs(xml_bytes: &[u8]) -> Vec<String> {
    let mut reader = XmlReader::from_reader(Cursor::new(xml_bytes));
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    let mut refs = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e) | Event::Empty(ref e)) => {
                let tag = e.name();
                let tag_bytes = tag.as_ref();
                if tag_bytes == b"ofd:ImageObject" || tag_bytes == b"ImageObject" {
                    for attr in e.attributes().flatten() {
                        let key = attr.key.as_ref();
                        if key == b"ResourceID" {
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
    fn image_rule_passes_when_no_images() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8"?>
<ofd:Page xmlns:ofd="http://www.ofdspec.org/2016">
  <ofd:ContentBlock/>
</ofd:Page>"#;
        let entries = make_entries(&[("Doc_0/Pages/Page_0.xml", xml)]);
        let result = ImageRule.check(&entries);
        assert!(result.passed, "{}", result.message);
    }

    #[test]
    fn image_rule_passes_when_resource_exists() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8"?>
<ofd:Page xmlns:ofd="http://www.ofdspec.org/2016">
  <ofd:ContentBlock>
    <ofd:ImageObject ResourceID="img_001"/>
  </ofd:ContentBlock>
</ofd:Page>"#;
        let entries = make_entries(&[
            ("Doc_0/Pages/Page_0.xml", xml),
            ("Doc_0/Res/img_001", b"fake-image-data"),
        ]);
        let result = ImageRule.check(&entries);
        assert!(result.passed, "{}", result.message);
    }

    #[test]
    fn image_rule_fails_when_resource_missing() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8"?>
<ofd:Page xmlns:ofd="http://www.ofdspec.org/2016">
  <ofd:ContentBlock>
    <ofd:ImageObject ResourceID="img_999"/>
  </ofd:ContentBlock>
</ofd:Page>"#;
        let entries = make_entries(&[("Doc_0/Pages/Page_0.xml", xml)]);
        let result = ImageRule.check(&entries);
        assert!(!result.passed);
        assert!(result.message.contains("img_999"));
    }

    #[test]
    fn image_rule_passes_for_non_page_xml() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8"?>
<ofd:Document xmlns:ofd="http://www.ofdspec.org/2016">
  <ofd:ImageObject ResourceID="img_001"/>
</ofd:Document>"#;
        let entries = make_entries(&[("Doc_0/Document.xml", xml)]);
        let result = ImageRule.check(&entries);
        assert!(result.passed, "{}", result.message);
    }
}
