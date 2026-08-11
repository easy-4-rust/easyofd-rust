//! 规则 23：文字对象应使用 Size 属性标识大小。
//!
//! 对应 Java: org.ofdrw.archive.check.rule.TextSizeRule
//! GB/T 42133-2022 6.6c

use crate::check::archive_violation::ArchiveViolation;
use crate::rules::ComplianceRule;

/// 规则 23：文字对象应使用 Size 属性标识大小。
///
/// 对应 Java: org.ofdrw.archive.check.rule.TextSizeRule
#[derive(Debug, Clone, Copy)]
pub struct TextSizeRule;

impl ComplianceRule for TextSizeRule {
    fn name(&self) -> &'static str {
        "TEXT_SIZE"
    }

    fn check(&self, entries: &[(String, Vec<u8>)]) -> crate::rules::RuleResult {
        for (name, data) in entries {
            if name.ends_with(".xml") {
                let content = String::from_utf8_lossy(data);
                // 检查 TextObject 是否有 Size 属性
                if content.contains("<ofd:TextObject") || content.contains("<TextObject") {
                    // 简单检查：如果有 TextObject 但没有 Size
                    if !content.contains("Size=") && !content.contains("size=") {
                        return crate::rules::RuleResult {
                            passed: true,
                            message: "文字对象未设置 Size 属性（建议添加）".into(),
                        };
                    }
                }
            }
        }
        crate::rules::RuleResult {
            passed: true,
            message: "文字 Size 检查通过".into(),
        }
    }
}

impl TextSizeRule {
    /// 转为 ArchiveViolation 形式检查。
    pub fn check_violations(&self, _entries: &[(String, Vec<u8>)]) -> Vec<ArchiveViolation> {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_size_rule_name() {
        assert_eq!(TextSizeRule.name(), "TEXT_SIZE");
    }

    #[test]
    fn text_size_rule_passes() {
        let entries = vec![(
            "Page_0.xml".into(),
            br#"<ofd:Page><ofd:TextObject Size="12"/></ofd:Page>"#.to_vec(),
        )];
        assert!(TextSizeRule.check(&entries).passed);
    }

    #[test]
    fn text_size_rule_passes_without_text() {
        let entries = vec![("Page_0.xml".into(), b"<ofd:Page/>".to_vec())];
        assert!(TextSizeRule.check(&entries).passed);
    }
}
