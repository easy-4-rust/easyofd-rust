//! 规则 8：去除扩展信息。
//!
//! 对应 Java: org.ofdrw.archive.check.rule.ExtensionRule
//! GB/T 42133-2022 6.2.2e

use crate::check::archive_violation::{ArchiveViolation, Severity};
use crate::rules::ComplianceRule;

/// 规则 8：去除扩展信息。
///
/// 检查 OFD 文档是否包含扩展信息，OFD-A 要求去除。
///
/// 对应 Java: org.ofdrw.archive.check.rule.ExtensionRule
#[derive(Debug, Clone, Copy)]
pub struct ExtensionRule;

impl ComplianceRule for ExtensionRule {
    fn name(&self) -> &'static str {
        "EXTENSION"
    }

    fn check(&self, entries: &[(String, Vec<u8>)]) -> crate::rules::RuleResult {
        // 检查 Document.xml 中是否包含 Extensions 元素
        for (name, data) in entries {
            if name.ends_with("Document.xml") {
                let content = String::from_utf8_lossy(data);
                if content.contains("<ofd:Extensions") || content.contains("<Extensions") {
                    return crate::rules::RuleResult {
                        passed: false,
                        message: "文档包含扩展信息，OFD-A 要求去除".into(),
                    };
                }
            }
        }
        crate::rules::RuleResult {
            passed: true,
            message: "未发现扩展信息".into(),
        }
    }
}

impl ExtensionRule {
    /// 转为 ArchiveViolation 形式检查。
    pub fn check_violations(&self, entries: &[(String, Vec<u8>)]) -> Vec<ArchiveViolation> {
        for (name, data) in entries {
            if name.ends_with("Document.xml") {
                let content = String::from_utf8_lossy(data);
                if content.contains("<ofd:Extensions") || content.contains("<Extensions") {
                    return vec![ArchiveViolation::new(
                        self.name(),
                        Severity::Warn,
                        "文档包含扩展信息，OFD-A 要求去除",
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
    fn extension_rule_name() {
        assert_eq!(ExtensionRule.name(), "EXTENSION");
    }

    #[test]
    fn extension_rule_passes_without_extensions() {
        let entries = vec![(
            "Doc_0/Document.xml".into(),
            br#"<ofd:Document xmlns:ofd="http://www.ofdspec.org/2016">
                <ofd:Pages><ofd:Page ID="1" BaseLoc="Pages/Page_0.xml"/></ofd:Pages>
            </ofd:Document>"#
                .to_vec(),
        )];
        let result = ExtensionRule.check(&entries);
        assert!(result.passed);
    }

    #[test]
    fn extension_rule_fails_with_extensions() {
        let entries = vec![(
            "Doc_0/Document.xml".into(),
            br#"<ofd:Document xmlns:ofd="http://www.ofdspec.org/2016">
                <ofd:Extensions><ofd:Ext/></ofd:Extensions>
            </ofd:Document>"#
                .to_vec(),
        )];
        let result = ExtensionRule.check(&entries);
        assert!(!result.passed);
    }

    #[test]
    fn extension_rule_violations_empty_when_pass() {
        let entries = vec![("Doc_0/Document.xml".into(), b"<ofd:Document/>".to_vec())];
        assert!(ExtensionRule.check_violations(&entries).is_empty());
    }
}
