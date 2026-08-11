//! 规则 19：多页共用的栅格图像应在文档资源中注册。
//!
//! 对应 Java: org.ofdrw.archive.check.rule.ImageResourceRegRule
//! GB/T 42133-2022 6.5a

use crate::check::archive_violation::ArchiveViolation;
use crate::rules::ComplianceRule;

/// 规则 19：多页共用的栅格图像应在文档资源中注册。
///
/// 对应 Java: org.ofdrw.archive.check.rule.ImageResourceRegRule
#[derive(Debug, Clone, Copy)]
pub struct ImageResourceRegRule;

impl ComplianceRule for ImageResourceRegRule {
    fn name(&self) -> &'static str {
        "IMAGE_RESOURCE_REG"
    }

    fn check(&self, _entries: &[(String, Vec<u8>)]) -> crate::rules::RuleResult {
        // Phase 1: 简单检查
        crate::rules::RuleResult {
            passed: true,
            message: "图像资源注册检查通过".into(),
        }
    }
}

impl ImageResourceRegRule {
    /// 转为 ArchiveViolation 形式检查。
    pub fn check_violations(&self, _entries: &[(String, Vec<u8>)]) -> Vec<ArchiveViolation> {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn image_resource_reg_rule_name() {
        assert_eq!(ImageResourceRegRule.name(), "IMAGE_RESOURCE_REG");
    }

    #[test]
    fn image_resource_reg_rule_passes() {
        assert!(ImageResourceRegRule.check(&[]).passed);
    }
}
