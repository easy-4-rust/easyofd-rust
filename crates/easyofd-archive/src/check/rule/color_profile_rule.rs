//! 规则 24：颜色空间建议带有颜色配置文件。
//!
//! 对应 Java: org.ofdrw.archive.check.rule.ColorProfileRule
//! GB/T 42133-2022 6.3.1c

use crate::check::archive_violation::ArchiveViolation;
use crate::rules::ComplianceRule;

/// 规则 24：颜色空间建议带有颜色配置文件。
///
/// 仅 INFO 级别提示，不做转换。
///
/// 对应 Java: org.ofdrw.archive.check.rule.ColorProfileRule
#[derive(Debug, Clone, Copy)]
pub struct ColorProfileRule;

impl ComplianceRule for ColorProfileRule {
    fn name(&self) -> &'static str {
        "COLOR_PROFILE"
    }

    fn check(&self, entries: &[(String, Vec<u8>)]) -> crate::rules::RuleResult {
        for (name, data) in entries {
            if name.ends_with(".xml") {
                let content = String::from_utf8_lossy(data);
                if content.contains("<ofd:ColorSpace") || content.contains("<ColorSpace") {
                    if !content.contains("Profile=") && !content.contains("ICCProfile") {
                        return crate::rules::RuleResult {
                            passed: true,
                            message: "颜色空间未带配置文件（建议添加）".into(),
                        };
                    }
                }
            }
        }
        crate::rules::RuleResult {
            passed: true,
            message: "颜色空间配置文件检查通过".into(),
        }
    }
}

impl ColorProfileRule {
    /// 转为 ArchiveViolation 形式检查。
    pub fn check_violations(&self, _entries: &[(String, Vec<u8>)]) -> Vec<ArchiveViolation> {
        // 仅 INFO 级别提示
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_profile_rule_name() {
        assert_eq!(ColorProfileRule.name(), "COLOR_PROFILE");
    }

    #[test]
    fn color_profile_rule_passes() {
        let entries = vec![];
        assert!(ColorProfileRule.check(&entries).passed);
    }
}
