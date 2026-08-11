//! 规则 6：Document + Page 中不得包含非 Goto 动作。
//!
//! 对应 Java: org.ofdrw.archive.check.rule.NonGotoActionRule
//! GB/T 42133-2022 6.2.2c/6.2.3c

use crate::check::archive_violation::{ArchiveViolation, Severity};
use crate::rules::ComplianceRule;

/// 规则 6：Document + Page 中不得包含非 Goto 动作。
///
/// 对应 Java: org.ofdrw.archive.check.rule.NonGotoActionRule
#[derive(Debug, Clone, Copy)]
pub struct NonGotoActionRule;

impl ComplianceRule for NonGotoActionRule {
    fn name(&self) -> &'static str {
        "NON_GOTO_ACTION"
    }

    fn check(&self, entries: &[(String, Vec<u8>)]) -> crate::rules::RuleResult {
        for (name, data) in entries {
            if name.ends_with("Document.xml") || name.contains("Page_") {
                let content = String::from_utf8_lossy(data);
                // 检查是否有非 Goto 动作
                if content.contains("<ofd:Action") || content.contains("<Action") {
                    // 简单检查：如果有 Action 但没有 Goto，视为不合规
                    if !content.contains("Goto") && !content.contains("goto") {
                        return crate::rules::RuleResult {
                            passed: false,
                            message: format!("文件 {name} 包含非 Goto 动作"),
                        };
                    }
                }
            }
        }
        crate::rules::RuleResult {
            passed: true,
            message: "非 Goto 动作检查通过".into(),
        }
    }
}

impl NonGotoActionRule {
    /// 转为 ArchiveViolation 形式检查。
    pub fn check_violations(&self, entries: &[(String, Vec<u8>)]) -> Vec<ArchiveViolation> {
        let mut violations = Vec::new();
        for (name, data) in entries {
            if name.ends_with("Document.xml") || name.contains("Page_") {
                let content = String::from_utf8_lossy(data);
                if (content.contains("<ofd:Action") || content.contains("<Action"))
                    && !content.contains("Goto")
                    && !content.contains("goto")
                {
                    violations.push(ArchiveViolation::new(
                        self.name(),
                        Severity::Error,
                        "包含非 Goto 动作",
                        Some(name.as_str()),
                        None::<String>,
                        Some("仅 Goto 动作"),
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
    fn non_goto_action_rule_name() {
        assert_eq!(NonGotoActionRule.name(), "NON_GOTO_ACTION");
    }

    #[test]
    fn non_goto_action_rule_passes_without_actions() {
        let entries = vec![("Doc_0/Document.xml".into(), b"<ofd:Document/>".to_vec())];
        assert!(NonGotoActionRule.check(&entries).passed);
    }

    #[test]
    fn non_goto_action_rule_passes_empty() {
        assert!(NonGotoActionRule.check(&[]).passed);
    }

    // ── 违规：Document.xml 中含非 Goto 动作 ───────────────────────────

    #[test]
    fn non_goto_action_rule_fails_with_non_goto_in_document() {
        let entries = vec![(
            "Doc_0/Document.xml".into(),
            b"<ofd:Document><ofd:Action Type=\"Print\"/></ofd:Document>".to_vec(),
        )];
        let result = NonGotoActionRule.check(&entries);
        assert!(!result.passed);
        assert!(result.message.contains("非 Goto"));
    }

    // ── 合规：Document.xml 中含 Goto 动作 ─────────────────────────────

    #[test]
    fn non_goto_action_rule_passes_with_goto_in_document() {
        let entries = vec![(
            "Doc_0/Document.xml".into(),
            b"<ofd:Document><ofd:Action Type=\"Goto\"/></ofd:Document>".to_vec(),
        )];
        assert!(NonGotoActionRule.check(&entries).passed);
    }

    // ── 违规：Page 文件中含非 Goto 动作 ────────────────────────────────

    #[test]
    fn non_goto_action_rule_fails_with_non_goto_in_page() {
        let entries = vec![(
            "Doc_0/Pages/Page_0.xml".into(),
            b"<ofd:Page><ofd:Action Type=\"Print\"/></ofd:Page>".to_vec(),
        )];
        let result = NonGotoActionRule.check(&entries);
        assert!(!result.passed);
        assert!(result.message.contains("Page_0"));
    }

    // ── 合规：Page 文件中含 Goto 动作 ──────────────────────────────────

    #[test]
    fn non_goto_action_rule_passes_with_goto_in_page() {
        let entries = vec![(
            "Doc_0/Pages/Page_0.xml".into(),
            b"<ofd:Page><ofd:Action Type=\"Goto\"/></ofd:Page>".to_vec(),
        )];
        assert!(NonGotoActionRule.check(&entries).passed);
    }

    // ── 边界：非 Document/Page 文件应跳过 ─────────────────────────────

    #[test]
    fn non_goto_action_rule_ignores_non_relevant_file() {
        let entries = vec![(
            "Doc_0/Res.xml".into(),
            b"<ofd:Res><ofd:Action Type=\"Print\"/></ofd:Res>".to_vec(),
        )];
        assert!(NonGotoActionRule.check(&entries).passed);
    }

    // ── 边界：goto（小写）也应通过 ─────────────────────────────────────

    #[test]
    fn non_goto_action_rule_passes_with_goto_lowercase() {
        let entries = vec![(
            "Doc_0/Document.xml".into(),
            b"<ofd:Document><ofd:Action Type=\"goto\"/></ofd:Document>".to_vec(),
        )];
        assert!(NonGotoActionRule.check(&entries).passed);
    }

    // ── 边界：无 Action 元素 ───────────────────────────────────────────

    #[test]
    fn non_goto_action_rule_passes_without_action_element() {
        let entries = vec![(
            "Doc_0/Document.xml".into(),
            b"<ofd:Document><ofd:Pages/></ofd:Document>".to_vec(),
        )];
        assert!(NonGotoActionRule.check(&entries).passed);
    }

    // ── check_violations：违规场景 ─────────────────────────────────────

    #[test]
    fn non_goto_action_violations_with_non_goto() {
        let entries = vec![(
            "Doc_0/Document.xml".into(),
            b"<ofd:Document><ofd:Action Type=\"Print\"/></ofd:Document>".to_vec(),
        )];
        let violations = NonGotoActionRule.check_violations(&entries);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule_name(), "NON_GOTO_ACTION");
        assert_eq!(violations[0].severity(), Severity::Error);
        assert!(violations[0].location().unwrap().contains("Document.xml"));
    }

    // ── check_violations：合规场景 ─────────────────────────────────────

    #[test]
    fn non_goto_action_violations_empty_on_pass() {
        let entries = vec![(
            "Doc_0/Document.xml".into(),
            b"<ofd:Document><ofd:Action Type=\"Goto\"/></ofd:Document>".to_vec(),
        )];
        assert!(NonGotoActionRule.check_violations(&entries).is_empty());
    }

    // ── check_violations：非相关文件跳过 ───────────────────────────────

    #[test]
    fn non_goto_action_violations_ignores_non_relevant() {
        let entries = vec![(
            "Doc_0/Res.xml".into(),
            b"<ofd:Res><ofd:Action Type=\"Print\"/></ofd:Res>".to_vec(),
        )];
        assert!(NonGotoActionRule.check_violations(&entries).is_empty());
    }
}
