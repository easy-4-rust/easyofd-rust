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

    // ── 违规：深度超过 3 ───────────────────────────────────────────────

    #[test]
    fn page_block_depth_rule_fails_depth_4() {
        let entries = vec![(
            "Page_0.xml".into(),
            b"<ofd:Page><ofd:PageBlock><ofd:PageBlock><ofd:PageBlock><ofd:PageBlock/></ofd:PageBlock></ofd:PageBlock></ofd:PageBlock></ofd:Page>".to_vec(),
        )];
        let result = PageBlockDepthRule.check(&entries);
        assert!(!result.passed);
        assert!(result.message.contains('4'));
        assert!(result.message.contains('3'));
    }

    // ── 违规：深度 5 ──────────────────────────────────────────────────

    #[test]
    fn page_block_depth_rule_fails_depth_5() {
        let entries = vec![(
            "Page_0.xml".into(),
            b"<ofd:Page><ofd:PageBlock><ofd:PageBlock><ofd:PageBlock><ofd:PageBlock><ofd:PageBlock/></ofd:PageBlock></ofd:PageBlock></ofd:PageBlock></ofd:PageBlock></ofd:Page>".to_vec(),
        )];
        let result = PageBlockDepthRule.check(&entries);
        assert!(!result.passed);
    }

    // ── 边界：非 Page_ 文件应跳过 ──────────────────────────────────────

    #[test]
    fn page_block_depth_rule_ignores_non_page_file() {
        let entries = vec![(
            "Doc_0/Document.xml".into(),
            b"<ofd:Document><ofd:PageBlock><ofd:PageBlock><ofd:PageBlock><ofd:PageBlock/></ofd:PageBlock></ofd:PageBlock></ofd:PageBlock></ofd:Document>".to_vec(),
        )];
        assert!(PageBlockDepthRule.check(&entries).passed);
    }

    // ── 边界：非 XML 文件应跳过 ────────────────────────────────────────

    #[test]
    fn page_block_depth_rule_ignores_non_xml() {
        let entries = vec![("Page_0.bin".into(), b"PageBlock".to_vec())];
        assert!(PageBlockDepthRule.check(&entries).passed);
    }

    // ── 边界：空条目列表 ───────────────────────────────────────────────

    #[test]
    fn page_block_depth_rule_passes_empty() {
        assert!(PageBlockDepthRule.check(&[]).passed);
    }

    // ── 边界：非命名空间前缀 PageBlock ─────────────────────────────────

    #[test]
    fn page_block_depth_rule_passes_with_plain_pageblock_within_limit() {
        let entries = vec![(
            "Page_0.xml".into(),
            b"<Page><PageBlock><PageBlock><PageBlock/></PageBlock></PageBlock></Page>".to_vec(),
        )];
        assert!(PageBlockDepthRule.check(&entries).passed);
    }

    // ── check_violations：违规场景 ─────────────────────────────────────

    #[test]
    fn page_block_depth_violations_with_depth_4() {
        let entries = vec![(
            "Page_0.xml".into(),
            b"<ofd:Page><ofd:PageBlock><ofd:PageBlock><ofd:PageBlock><ofd:PageBlock/></ofd:PageBlock></ofd:PageBlock></ofd:PageBlock></ofd:Page>".to_vec(),
        )];
        let violations = PageBlockDepthRule.check_violations(&entries);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule_name(), "PAGEBLOCK_DEPTH");
        assert_eq!(violations[0].severity(), Severity::Error);
        assert!(violations[0].location().unwrap().contains("Page_0.xml"));
        assert_eq!(violations[0].actual_value(), Some("4"));
        assert_eq!(violations[0].expected_value(), Some("3"));
    }

    // ── check_violations：合规场景 ─────────────────────────────────────

    #[test]
    fn page_block_depth_violations_empty_on_pass() {
        let entries = vec![(
            "Page_0.xml".into(),
            b"<ofd:Page><ofd:PageBlock><ofd:PageBlock><ofd:PageBlock/></ofd:PageBlock></ofd:PageBlock></ofd:Page>".to_vec(),
        )];
        assert!(PageBlockDepthRule.check_violations(&entries).is_empty());
    }

    // ── check_violations：非 Page_ 文件跳过 ────────────────────────────

    #[test]
    fn page_block_depth_violations_ignores_non_page() {
        let entries = vec![(
            "Document.xml".into(),
            b"<ofd:PageBlock><ofd:PageBlock><ofd:PageBlock><ofd:PageBlock/></ofd:PageBlock></ofd:PageBlock></ofd:PageBlock>".to_vec(),
        )];
        assert!(PageBlockDepthRule.check_violations(&entries).is_empty());
    }
}
