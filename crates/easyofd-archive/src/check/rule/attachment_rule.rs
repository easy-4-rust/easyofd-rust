//! 规则：附件合规检查。
//!
//! 对应 Java: org.ofdrw.archive.check.rule.AttachmentRule
//! GB/T 42133-2022 6.15

use crate::check::archive_violation::{ArchiveViolation, Severity};
use crate::rules::ComplianceRule;

/// OFD-A 中可保留的附件格式（文本类）。
const KEEP_FORMATS: &[&str] = &["TXT", "XML"];

/// 规则：附件合规检查。
///
/// OFD-A 中附件应仅保留文本格式（TXT/XML）或已归档的技术文档。
///
/// 对应 Java: org.ofdrw.archive.check.rule.AttachmentRule
#[derive(Debug, Clone, Copy)]
pub struct AttachmentRule;

impl ComplianceRule for AttachmentRule {
    fn name(&self) -> &'static str {
        "ATTACHMENT"
    }

    fn check(&self, entries: &[(String, Vec<u8>)]) -> crate::rules::RuleResult {
        for (name, data) in entries {
            if name.contains("Attachment") && name.ends_with(".xml") {
                let content = String::from_utf8_lossy(data);
                // 检查附件格式属性
                if content.contains("Format=") {
                    for line in content.split("Format=\"").skip(1) {
                        if let Some(end) = line.find('"') {
                            let format = &line[..end];
                            if !KEEP_FORMATS.contains(&format) {
                                return crate::rules::RuleResult {
                                    passed: false,
                                    message: format!("附件格式 {format} 不符合 OFD-A 要求"),
                                };
                            }
                        }
                    }
                }
            }
        }
        crate::rules::RuleResult {
            passed: true,
            message: "附件合规检查通过".into(),
        }
    }
}

impl AttachmentRule {
    /// 转为 ArchiveViolation 形式检查。
    pub fn check_violations(&self, entries: &[(String, Vec<u8>)]) -> Vec<ArchiveViolation> {
        let mut violations = Vec::new();
        for (name, data) in entries {
            if name.contains("Attachment") && name.ends_with(".xml") {
                let content = String::from_utf8_lossy(data);
                if content.contains("Format=") {
                    for line in content.split("Format=\"").skip(1) {
                        if let Some(end) = line.find('"') {
                            let format = &line[..end];
                            if !KEEP_FORMATS.contains(&format) {
                                violations.push(ArchiveViolation::new(
                                    self.name(),
                                    Severity::Warn,
                                    format!("附件格式 {format} 不符合 OFD-A 要求"),
                                    Some(name.as_str()),
                                    Some(format),
                                    Some("TXT/XML"),
                                ));
                            }
                        }
                    }
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
    fn attachment_rule_name() {
        assert_eq!(AttachmentRule.name(), "ATTACHMENT");
    }

    #[test]
    fn attachment_rule_passes_without_attachments() {
        let entries = vec![("OFD.xml".into(), b"<ofd:OFD/>".to_vec())];
        assert!(AttachmentRule.check(&entries).passed);
    }

    #[test]
    fn attachment_rule_passes_with_txt() {
        let entries = vec![(
            "Doc_0/Attachments.xml".into(),
            br#"<ofd:Attachments><ofd:Attachment Format="TXT"/></ofd:Attachments>"#.to_vec(),
        )];
        assert!(AttachmentRule.check(&entries).passed);
    }

    #[test]
    fn attachment_rule_fails_with_pdf() {
        let entries = vec![(
            "Doc_0/Attachments.xml".into(),
            br#"<ofd:Attachments><ofd:Attachment Format="PDF"/></ofd:Attachments>"#.to_vec(),
        )];
        assert!(!AttachmentRule.check(&entries).passed);
    }
}
