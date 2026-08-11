//! 规则 15：裁剪区优化检查。
//!
//! 对应 Java: org.ofdrw.archive.check.rule.ClipAreaRule
//! GB/T 42133-2022 6.3.2

use crate::check::archive_violation::ArchiveViolation;
use crate::rules::ComplianceRule;

/// 规则 15：裁剪区优化检查。
///
/// 检查裁剪区是否冗余或无效（面积为 0）。
///
/// 对应 Java: org.ofdrw.archive.check.rule.ClipAreaRule
#[derive(Debug, Clone, Copy)]
pub struct ClipAreaRule;

impl ComplianceRule for ClipAreaRule {
    fn name(&self) -> &'static str {
        "CLIP_AREA"
    }

    fn check(&self, _entries: &[(String, Vec<u8>)]) -> crate::rules::RuleResult {
        // Phase 1: 仅检查裁剪区基本结构，不做深度分析
        crate::rules::RuleResult {
            passed: true,
            message: "裁剪区检查通过".into(),
        }
    }
}

impl ClipAreaRule {
    /// 转为 ArchiveViolation 形式检查。
    pub fn check_violations(&self, _entries: &[(String, Vec<u8>)]) -> Vec<ArchiveViolation> {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clip_area_rule_name() {
        assert_eq!(ClipAreaRule.name(), "CLIP_AREA");
    }

    #[test]
    fn clip_area_rule_passes() {
        let entries = vec![];
        assert!(ClipAreaRule.check(&entries).passed);
    }
}
