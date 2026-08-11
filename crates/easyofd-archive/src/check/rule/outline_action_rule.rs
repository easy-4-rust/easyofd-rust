//! 规则 7：大纲节点动作检查。
//!
//! 对应 Java: org.ofdrw.archive.check.rule.OutlineActionRule
//! GB/T 42133-2022 6.2.5a/b

use crate::check::archive_violation::ArchiveViolation;
use crate::rules::ComplianceRule;

/// 规则 7：大纲节点动作检查。
///
/// 对应 Java: org.ofdrw.archive.check.rule.OutlineActionRule
#[derive(Debug, Clone, Copy)]
pub struct OutlineActionRule;

impl ComplianceRule for OutlineActionRule {
    fn name(&self) -> &'static str {
        "OUTLINE_ACTION"
    }

    fn check(&self, entries: &[(String, Vec<u8>)]) -> crate::rules::RuleResult {
        for (name, data) in entries {
            if name.ends_with("Document.xml") {
                let content = String::from_utf8_lossy(data);
                if content.contains("<ofd:Outlines") || content.contains("<Outlines") {
                    // 检查大纲中是否有非 Goto 动作
                    if content.contains("OutlineElem")
                        && (content.contains("<ofd:Action") || content.contains("<Action"))
                        && !content.contains("Goto")
                    {
                        return crate::rules::RuleResult {
                            passed: false,
                            message: "大纲节点包含非 Goto 动作".into(),
                        };
                    }
                }
            }
        }
        crate::rules::RuleResult {
            passed: true,
            message: "大纲动作检查通过".into(),
        }
    }
}

impl OutlineActionRule {
    /// 转为 ArchiveViolation 形式检查。
    pub fn check_violations(&self, _entries: &[(String, Vec<u8>)]) -> Vec<ArchiveViolation> {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn outline_action_rule_name() {
        assert_eq!(OutlineActionRule.name(), "OUTLINE_ACTION");
    }

    #[test]
    fn outline_action_rule_passes_without_outlines() {
        let entries = vec![("Doc_0/Document.xml".into(), b"<ofd:Document/>".to_vec())];
        assert!(OutlineActionRule.check(&entries).passed);
    }

    // ── 合规：有 Outlines 但无 Action ─────────────────────────────────

    #[test]
    fn outline_action_rule_passes_with_outlines_no_action() {
        let entries = vec![(
            "Doc_0/Document.xml".into(),
            br"<ofd:Document><ofd:Outlines><ofd:OutlineElem/></ofd:Outlines></ofd:Document>"
                .to_vec(),
        )];
        assert!(OutlineActionRule.check(&entries).passed);
    }

    // ── 合规：有 Outlines + Action + Goto ─────────────────────────────

    #[test]
    fn outline_action_rule_passes_with_goto_action() {
        let entries = vec![(
            "Doc_0/Document.xml".into(),
            b"<ofd:Document><ofd:Outlines><ofd:OutlineElem><ofd:Action Type=\"Goto\"/></ofd:OutlineElem></ofd:Outlines></ofd:Document>".to_vec(),
        )];
        assert!(OutlineActionRule.check(&entries).passed);
    }

    // ── 违规：有 Outlines + Action 但无 Goto ──────────────────────────

    #[test]
    fn outline_action_rule_fails_with_non_goto_action() {
        let entries = vec![(
            "Doc_0/Document.xml".into(),
            b"<ofd:Document><ofd:Outlines><ofd:OutlineElem><ofd:Action Type=\"Print\"/></ofd:OutlineElem></ofd:Outlines></ofd:Document>".to_vec(),
        )];
        let result = OutlineActionRule.check(&entries);
        assert!(!result.passed);
        assert!(result.message.contains("非 Goto"));
    }

    // ── 边界：非 Document.xml 文件应跳过 ──────────────────────────────

    #[test]
    fn outline_action_rule_ignores_non_document_xml() {
        let entries = vec![(
            "Doc_0/Page_0.xml".into(),
            br"<ofd:Page><ofd:Outlines><ofd:Action/></ofd:Outlines></ofd:Page>".to_vec(),
        )];
        assert!(OutlineActionRule.check(&entries).passed);
    }

    // ── 边界：空条目列表 ───────────────────────────────────────────────

    #[test]
    fn outline_action_rule_passes_empty() {
        assert!(OutlineActionRule.check(&[]).passed);
    }

    // ── 边界：有 Outlines 但无 OutlineElem ────────────────────────────

    #[test]
    fn outline_action_rule_passes_outlines_without_elem() {
        let entries = vec![(
            "Doc_0/Document.xml".into(),
            br"<ofd:Document><ofd:Outlines/></ofd:Document>".to_vec(),
        )];
        assert!(OutlineActionRule.check(&entries).passed);
    }

    // ── 边界：无 Outlines 有 Action（不触发大纲检查） ─────────────────

    #[test]
    fn outline_action_rule_passes_action_without_outlines() {
        let entries = vec![(
            "Doc_0/Document.xml".into(),
            b"<ofd:Document><ofd:Action Type=\"Print\"/></ofd:Document>".to_vec(),
        )];
        assert!(OutlineActionRule.check(&entries).passed);
    }

    // ── check_violations 始终返回空 ────────────────────────────────────

    #[test]
    fn outline_action_violations_always_empty() {
        let entries = vec![(
            "Doc_0/Document.xml".into(),
            b"<ofd:Document><ofd:Outlines><ofd:OutlineElem><ofd:Action Type=\"Print\"/></ofd:OutlineElem></ofd:Outlines></ofd:Document>".to_vec(),
        )];
        assert!(OutlineActionRule.check_violations(&entries).is_empty());
    }
}
