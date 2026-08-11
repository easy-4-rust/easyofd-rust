//! 规则 4：去除权限声明。
//!
//! 对应 Java: org.ofdrw.archive.check.rule.PermissionRule
//! GB/T 42133-2022 6.2.2a

use crate::check::archive_violation::{ArchiveViolation, Severity};
use crate::rules::ComplianceRule;

/// 规则 4：去除权限声明。
///
/// 对应 Java: org.ofdrw.archive.check.rule.PermissionRule
#[derive(Debug, Clone, Copy)]
pub struct PermissionRule;

impl ComplianceRule for PermissionRule {
    fn name(&self) -> &'static str {
        "PERMISSION"
    }

    fn check(&self, entries: &[(String, Vec<u8>)]) -> crate::rules::RuleResult {
        for (name, data) in entries {
            if name.ends_with("Document.xml") {
                let content = String::from_utf8_lossy(data);
                if content.contains("<ofd:Permissions") || content.contains("<Permissions") {
                    return crate::rules::RuleResult {
                        passed: false,
                        message: "文档包含权限声明，OFD-A 要求去除".into(),
                    };
                }
            }
        }
        crate::rules::RuleResult {
            passed: true,
            message: "权限声明检查通过".into(),
        }
    }
}

impl PermissionRule {
    /// 转为 ArchiveViolation 形式检查。
    pub fn check_violations(&self, entries: &[(String, Vec<u8>)]) -> Vec<ArchiveViolation> {
        for (name, data) in entries {
            if name.ends_with("Document.xml") {
                let content = String::from_utf8_lossy(data);
                if content.contains("<ofd:Permissions") || content.contains("<Permissions") {
                    return vec![ArchiveViolation::new(
                        self.name(),
                        Severity::Warn,
                        "文档包含权限声明，OFD-A 要求去除",
                        Some(name.as_str()),
                        Some("存在"),
                        Some("无"),
                    )];
                }
            }
        }
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn permission_rule_name() {
        assert_eq!(PermissionRule.name(), "PERMISSION");
    }

    #[test]
    fn permission_rule_passes_without_permissions() {
        let entries = vec![("Doc_0/Document.xml".into(), b"<ofd:Document/>".to_vec())];
        assert!(PermissionRule.check(&entries).passed);
    }

    #[test]
    fn permission_rule_fails_with_permissions() {
        let entries = vec![(
            "Doc_0/Document.xml".into(),
            br"<ofd:Document><ofd:Permissions/></ofd:Document>".to_vec(),
        )];
        assert!(!PermissionRule.check(&entries).passed);
    }
}
