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
}
