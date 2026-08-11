//! 规则 14：PageBlock 嵌套深度不超过 3。
//!
//! 对应 Java: org.ofdrw.archive.check.rule.PageBlockDepthRule
//! GB/T 42133-2022 6.2.3e

use crate::check::archive_violation::{ArchiveViolation, Severity};
use crate::rules::ComplianceRule;

/// 最大允许嵌套深度。
const MAX_DEPTH: usize = 3;

/// 规则 14：PageBlock 嵌套深度不超过 3。
///
/// 对应 Java: org.ofdrw.archive.check.rule.PageBlockDepthRule
#[derive(Debug, Clone, Copy)]
pub struct PageBlockDepthRule;

impl ComplianceRule for PageBlockDepthRule {
    fn name(&self) -> &'static str {
        "PAGEBLOCK_DEPTH"
    }

    fn check(&self, entries: &[(String, Vec<u8>)]) -> crate::rules::RuleResult {
        for (name, data) in entries {
            if name.contains("Page_") && name.ends_with(".xml") {
                let content = String::from_utf8_lossy(data);
                let depth = content.matches("<ofd:PageBlock").count()
                    + content.matches("<PageBlock").count();
                if depth > MAX_DEPTH {
                    return crate::rules::RuleResult {
                        passed: false,
                        message: format!("PageBlock 嵌套深度 {depth} 超过限制 {MAX_DEPTH}"),
                    };
                }
            }
        }
        crate::rules::RuleResult {
            passed: true,
            message: "PageBlock 嵌套深度检查通过".into(),
        }
    }
}

impl PageBlockDepthRule {
    /// 转为 ArchiveViolation 形式检查。
    pub fn check_violations(&self, entries: &[(String, Vec<u8>)]) -> Vec<ArchiveViolation> {
        let mut violations = Vec::new();
        for (name, data) in entries {
            if name.contains("Page_") && name.ends_with(".xml") {
                let content = String::from_utf8_lossy(data);
                let depth = content.matches("<ofd:PageBlock").count()
                    + content.matches("<PageBlock").count();
                if depth > MAX_DEPTH {
                    violations.push(ArchiveViolation::new(
                        self.name(),
                        Severity::Error,
                        format!("PageBlock 嵌套深度 {depth} 超过限制 {MAX_DEPTH}"),
                        Some(name.as_str()),
                        Some(depth.to_string()),
                        Some(MAX_DEPTH.to_string()),
                    ));
                }
            }
        }
        violations
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_block_depth_rule_name() {
        assert_eq!(PageBlockDepthRule.name(), "PAGEBLOCK_DEPTH");
    }

    #[test]
    fn page_block_depth_rule_passes_without_page_blocks() {
        let entries = vec![("Page_0.xml".into(), b"<ofd:Page/>".to_vec())];
        assert!(PageBlockDepthRule.check(&entries).passed);
    }

    #[test]
    fn page_block_depth_rule_passes_within_limit() {
        let entries = vec![(
            "Page_0.xml".into(),
            b"<ofd:Page><ofd:PageBlock><ofd:PageBlock><ofd:PageBlock/></ofd:PageBlock></ofd:PageBlock></ofd:Page>".to_vec(),
        )];
        assert!(PageBlockDepthRule.check(&entries).passed);
    }
}
