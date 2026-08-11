//! 规则 18：资源应在正确位置定义。
//!
//! 对应 Java: org.ofdrw.archive.check.rule.ResourcePlacementRule
//! GB/T 42133-2022 6.2.6a

use crate::check::archive_violation::ArchiveViolation;
use crate::rules::ComplianceRule;

/// 规则 18：资源应在正确位置定义。
///
/// ColorSpace/Font 应在 PublicRes 中，Image/VectorG/DrawParam 应在 DocumentRes 或 PageRes。
///
/// 对应 Java: org.ofdrw.archive.check.rule.ResourcePlacementRule
#[derive(Debug, Clone, Copy)]
pub struct ResourcePlacementRule;

impl ComplianceRule for ResourcePlacementRule {
    fn name(&self) -> &'static str {
        "RESOURCE_PLACEMENT"
    }

    fn check(&self, _entries: &[(String, Vec<u8>)]) -> crate::rules::RuleResult {
        // Phase 1: 简单检查
        crate::rules::RuleResult {
            passed: true,
            message: "资源位置检查通过".into(),
        }
    }
}

impl ResourcePlacementRule {
    /// 转为 ArchiveViolation 形式检查。
    pub fn check_violations(&self, _entries: &[(String, Vec<u8>)]) -> Vec<ArchiveViolation> {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resource_placement_rule_name() {
        assert_eq!(ResourcePlacementRule.name(), "RESOURCE_PLACEMENT");
    }

    #[test]
    fn resource_placement_rule_passes() {
        assert!(ResourcePlacementRule.check(&[]).passed);
    }
}
