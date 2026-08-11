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

    // ── 合规：XML 格式附件 ─────────────────────────────────────────────

    #[test]
    fn attachment_rule_passes_with_xml_format() {
        let entries = vec![(
            "Doc_0/Attachments.xml".into(),
            br#"<ofd:Attachments><ofd:Attachment Format="XML"/></ofd:Attachments>"#.to_vec(),
        )];
        assert!(AttachmentRule.check(&entries).passed);
    }

    // ── 违规：DOCX 格式 ───────────────────────────────────────────────

    #[test]
    fn attachment_rule_fails_with_docx() {
        let entries = vec![(
            "Doc_0/Attachments.xml".into(),
            br#"<ofd:Attachments><ofd:Attachment Format="DOCX"/></ofd:Attachments>"#.to_vec(),
        )];
        let result = AttachmentRule.check(&entries);
        assert!(!result.passed);
        assert!(result.message.contains("DOCX"));
    }

    // ── 边界：附件 XML 无 Format 属性应通过 ───────────────────────────

    #[test]
    fn attachment_rule_passes_without_format_attr() {
        let entries = vec![(
            "Doc_0/Attachments.xml".into(),
            br"<ofd:Attachments><ofd:Attachment/></ofd:Attachments>".to_vec(),
        )];
        assert!(AttachmentRule.check(&entries).passed);
    }

    // ── 边界：非 Attachment 文件含 Format 应跳过 ──────────────────────

    #[test]
    fn attachment_rule_ignores_non_attachment_xml() {
        let entries = vec![(
            "Doc_0/Res.xml".into(),
            br#"<ofd:Res><ofd:MultiMedia Format="PDF"/></ofd:Res>"#.to_vec(),
        )];
        assert!(AttachmentRule.check(&entries).passed);
    }

    // ── 边界：非 XML 附件文件应跳过 ───────────────────────────────────

    #[test]
    fn attachment_rule_ignores_non_xml_attachment() {
        let entries = vec![("Doc_0/Attachment.bin".into(), b"binary".to_vec())];
        assert!(AttachmentRule.check(&entries).passed);
    }

    // ── 边界：空条目列表 ───────────────────────────────────────────────

    #[test]
    fn attachment_rule_passes_empty() {
        assert!(AttachmentRule.check(&[]).passed);
    }

    // ── check_violations：违规场景 ─────────────────────────────────────

    #[test]
    fn attachment_violations_with_bad_format() {
        let entries = vec![(
            "Doc_0/Attachments.xml".into(),
            br#"<ofd:Attachments><ofd:Attachment Format="PDF"/></ofd:Attachments>"#.to_vec(),
        )];
        let violations = AttachmentRule.check_violations(&entries);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule_name(), "ATTACHMENT");
        assert_eq!(violations[0].severity(), Severity::Warn);
        assert!(
            violations[0]
                .location()
                .unwrap()
                .contains("Attachments.xml")
        );
        assert_eq!(violations[0].actual_value(), Some("PDF"));
        assert_eq!(violations[0].expected_value(), Some("TXT/XML"));
    }

    // ── check_violations：合规场景 ─────────────────────────────────────

    #[test]
    fn attachment_violations_empty_on_pass() {
        let entries = vec![(
            "Doc_0/Attachments.xml".into(),
            br#"<ofd:Attachments><ofd:Attachment Format="TXT"/></ofd:Attachments>"#.to_vec(),
        )];
        assert!(AttachmentRule.check_violations(&entries).is_empty());
    }

    // ── check_violations：多个违规 ─────────────────────────────────────

    #[test]
    fn attachment_violations_multiple_formats() {
        let entries = vec![(
            "Doc_0/Attachments.xml".into(),
            br#"<ofd:Attachments>
                <ofd:Attachment Format="PDF"/>
                <ofd:Attachment Format="DOCX"/>
            </ofd:Attachments>"#
                .to_vec(),
        )];
        let violations = AttachmentRule.check_violations(&entries);
        assert_eq!(violations.len(), 2);
    }

    // ── check_violations：非 XML 跳过 ─────────────────────────────────

    #[test]
    fn attachment_violations_ignores_non_xml() {
        let entries = vec![("Attachment.bin".into(), b"Format=\"PDF\"".to_vec())];
        assert!(AttachmentRule.check_violations(&entries).is_empty());
    }
}
