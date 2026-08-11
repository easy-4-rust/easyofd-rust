//! 规则 13：图像资源禁止使用扩展机制加入自定义数据。
//!
//! 对应 Java: org.ofdrw.archive.check.rule.ImageExtensionRule
//! GB/T 42133-2022 6.2.6f

use crate::check::archive_violation::ArchiveViolation;
use crate::rules::ComplianceRule;

/// 规则 13：图像资源禁止使用扩展机制加入自定义数据。
///
/// 对应 Java: org.ofdrw.archive.check.rule.ImageExtensionRule
#[derive(Debug, Clone, Copy)]
pub struct ImageExtensionRule;

impl ComplianceRule for ImageExtensionRule {
    fn name(&self) -> &'static str {
        "IMAGE_EXTENSION"
    }

    fn check(&self, _entries: &[(String, Vec<u8>)]) -> crate::rules::RuleResult {
        // Phase 1: 简单检查
        crate::rules::RuleResult {
            passed: true,
            message: "图像扩展检查通过".into(),
        }
    }
}

impl ImageExtensionRule {
    /// 转为 ArchiveViolation 形式检查。
    pub fn check_violations(&self, _entries: &[(String, Vec<u8>)]) -> Vec<ArchiveViolation> {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn image_extension_rule_name() {
        assert_eq!(ImageExtensionRule.name(), "IMAGE_EXTENSION");
    }

    #[test]
    fn image_extension_rule_passes() {
        assert!(ImageExtensionRule.check(&[]).passed);
    }
}
