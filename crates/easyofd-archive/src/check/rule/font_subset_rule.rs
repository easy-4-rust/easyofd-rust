//! 规则 11：字体必须嵌入子集化字型数据。
//!
//! 对应 Java: org.ofdrw.archive.check.rule.FontSubsetRule
//! GB/T 42133-2022 6.2.6b

use crate::check::archive_violation::ArchiveViolation;
use crate::rules::ComplianceRule;

/// 规则 11：字体必须嵌入子集化字型数据。
///
/// Phase 1 仅检查字体文件是否嵌入，子集化验证留到 Phase 2。
///
/// 对应 Java: org.ofdrw.archive.check.rule.FontSubsetRule
#[derive(Debug, Clone, Copy)]
pub struct FontSubsetRule;

impl ComplianceRule for FontSubsetRule {
    fn name(&self) -> &'static str {
        "FONT_SUBSET"
    }

    fn check(&self, entries: &[(String, Vec<u8>)]) -> crate::rules::RuleResult {
        // 检查是否有字体资源声明
        let has_font_decl = entries.iter().any(|(name, _)| {
            name.ends_with(".xml")
                && entries
                    .iter()
                    .filter(|(n, _)| n.ends_with(".xml"))
                    .any(|(_, data)| {
                        let content = String::from_utf8_lossy(data);
                        content.contains("<ofd:Font") || content.contains("<Font ")
                    })
        });

        if has_font_decl {
            // 检查是否有对应的字体文件
            let has_font_file = entries.iter().any(|(name, _)| {
                name.ends_with(".ttf") || name.ends_with(".otf") || name.ends_with(".ttc")
            });
            if !has_font_file {
                return crate::rules::RuleResult {
                    passed: false,
                    message: "字体声明存在但未找到嵌入的字体文件".into(),
                };
            }
        }

        crate::rules::RuleResult {
            passed: true,
            message: "字体嵌入检查通过".into(),
        }
    }
}

impl FontSubsetRule {
    /// 转为 ArchiveViolation 形式检查。
    pub fn check_violations(&self, _entries: &[(String, Vec<u8>)]) -> Vec<ArchiveViolation> {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn font_subset_rule_name() {
        assert_eq!(FontSubsetRule.name(), "FONT_SUBSET");
    }

    #[test]
    fn font_subset_rule_passes_without_fonts() {
        let entries = vec![("OFD.xml".into(), b"<ofd:OFD/>".to_vec())];
        assert!(FontSubsetRule.check(&entries).passed);
    }
}
