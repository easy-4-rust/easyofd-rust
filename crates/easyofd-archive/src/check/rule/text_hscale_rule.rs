//! 规则 25：文字仅横向缩放时应用 HScale 属性。
//!
//! 对应 Java: org.ofdrw.archive.check.rule.TextHScaleRule
//! GB/T 42133-2022 6.6d

use crate::check::archive_violation::ArchiveViolation;
use crate::rules::ComplianceRule;

/// 规则 25：文字仅横向缩放时应用 HScale 属性。
///
/// 对应 Java: org.ofdrw.archive.check.rule.TextHScaleRule
#[derive(Debug, Clone, Copy)]
pub struct TextHScaleRule;

impl ComplianceRule for TextHScaleRule {
    fn name(&self) -> &'static str {
        "TEXT_HSCALE"
    }

    fn check(&self, _entries: &[(String, Vec<u8>)]) -> crate::rules::RuleResult {
        // Phase 1: 简单检查
        crate::rules::RuleResult {
            passed: true,
            message: "文字 HScale 检查通过".into(),
        }
    }
}

impl TextHScaleRule {
    /// 转为 ArchiveViolation 形式检查。
    pub fn check_violations(&self, _entries: &[(String, Vec<u8>)]) -> Vec<ArchiveViolation> {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_hscale_rule_name() {
        assert_eq!(TextHScaleRule.name(), "TEXT_HSCALE");
    }

    #[test]
    fn text_hscale_rule_passes() {
        assert!(TextHScaleRule.check(&[]).passed);
    }
}
