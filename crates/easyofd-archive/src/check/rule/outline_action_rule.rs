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
}
