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

    // ── 违规：无命名空间前缀的 Permissions ────────────────────────────

    #[test]
    fn permission_rule_fails_with_plain_permissions() {
        let entries = vec![(
            "Doc_0/Document.xml".into(),
            br"<Document><Permissions/></Document>".to_vec(),
        )];
        let result = PermissionRule.check(&entries);
        assert!(!result.passed);
        assert!(result.message.contains("权限声明"));
    }

    // ── 边界：非 Document.xml 文件应跳过 ──────────────────────────────

    #[test]
    fn permission_rule_ignores_non_document_xml() {
        let entries = vec![(
            "Doc_0/Page_0.xml".into(),
            br"<ofd:Page><ofd:Permissions/></ofd:Page>".to_vec(),
        )];
        assert!(PermissionRule.check(&entries).passed);
    }

    // ── 边界：空条目列表 ───────────────────────────────────────────────

    #[test]
    fn permission_rule_passes_empty() {
        assert!(PermissionRule.check(&[]).passed);
    }

    // ── 边界：Document.xml 无 Permissions 元素 ─────────────────────────

    #[test]
    fn permission_rule_passes_with_only_pages() {
        let entries = vec![(
            "Doc_0/Document.xml".into(),
            br"<ofd:Document><ofd:Pages/></ofd:Document>".to_vec(),
        )];
        assert!(PermissionRule.check(&entries).passed);
    }

    // ── check_violations：违规场景 ─────────────────────────────────────

    #[test]
    fn permission_violations_with_permissions() {
        let entries = vec![(
            "Doc_0/Document.xml".into(),
            br"<ofd:Document><ofd:Permissions/></ofd:Document>".to_vec(),
        )];
        let violations = PermissionRule.check_violations(&entries);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule_name(), "PERMISSION");
        assert_eq!(violations[0].severity(), Severity::Warn);
        assert!(violations[0].location().unwrap().contains("Document.xml"));
        assert_eq!(violations[0].actual_value(), Some("存在"));
        assert_eq!(violations[0].expected_value(), Some("无"));
    }

    // ── check_violations：合规场景 ─────────────────────────────────────

    #[test]
    fn permission_violations_empty_on_pass() {
        let entries = vec![("Doc_0/Document.xml".into(), b"<ofd:Document/>".to_vec())];
        assert!(PermissionRule.check_violations(&entries).is_empty());
    }

    // ── check_violations：非 Document.xml 跳过 ─────────────────────────

    #[test]
    fn permission_violations_ignores_non_document() {
        let entries = vec![(
            "Doc_0/Page_0.xml".into(),
            br"<ofd:Page><ofd:Permissions/></ofd:Page>".to_vec(),
        )];
        assert!(PermissionRule.check_violations(&entries).is_empty());
    }
}
