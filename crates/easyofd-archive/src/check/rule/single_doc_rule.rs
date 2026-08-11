//! 规则 2：OFD 文件只能包含一个文档。
//!
//! 对应 Java: org.ofdrw.archive.check.rule.SingleDocRule
//! GB/T 42133-2022 6.2.1c

use crate::check::archive_violation::{ArchiveViolation, Severity};
use crate::rules::ComplianceRule;

/// 规则 2：OFD 文件只能包含一个文档。
///
/// 检查 OFD.xml 中是否存在多个 DocBody。
///
/// 对应 Java: org.ofdrw.archive.check.rule.SingleDocRule
#[derive(Debug, Clone, Copy)]
pub struct SingleDocRule;

impl ComplianceRule for SingleDocRule {
    fn name(&self) -> &'static str {
        "SINGLE_DOC"
    }

    fn check(&self, entries: &[(String, Vec<u8>)]) -> crate::rules::RuleResult {
        let ofd_xml = match entries.iter().find(|(name, _)| name == "OFD.xml") {
            Some((_, data)) => data,
            None => {
                return crate::rules::RuleResult {
                    passed: false,
                    message: "OFD.xml 不存在".into(),
                };
            }
        };

        let content = String::from_utf8_lossy(ofd_xml);
        let doc_body_count =
            content.matches("<ofd:DocBody").count() + content.matches("<DocBody").count();

        if doc_body_count > 1 {
            crate::rules::RuleResult {
                passed: false,
                message: format!("OFD 文件包含 {doc_body_count} 个文档体，OFD-A 只允许单个文档"),
            }
        } else {
            crate::rules::RuleResult {
                passed: true,
                message: "单文档检查通过".into(),
            }
        }
    }
}

impl SingleDocRule {
    /// 转为 ArchiveViolation 形式检查。
    pub fn check_violations(&self, entries: &[(String, Vec<u8>)]) -> Vec<ArchiveViolation> {
        let ofd_xml = match entries.iter().find(|(name, _)| name == "OFD.xml") {
            Some((_, data)) => data,
            None => {
                return vec![ArchiveViolation::error(self.name(), "OFD.xml 不存在")];
            }
        };

        let content = String::from_utf8_lossy(ofd_xml);
        let doc_body_count =
            content.matches("<ofd:DocBody").count() + content.matches("<DocBody").count();

        if doc_body_count > 1 {
            vec![ArchiveViolation::new(
                self.name(),
                Severity::Error,
                format!("OFD 文件包含 {doc_body_count} 个文档体，OFD-A 只允许单个文档"),
                Some("OFD.xml"),
                Some(doc_body_count.to_string()),
                Some("1"),
            )]
        } else {
            Vec::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_doc_rule_name() {
        assert_eq!(SingleDocRule.name(), "SINGLE_DOC");
    }

    #[test]
    fn single_doc_rule_passes() {
        let entries = vec![(
            "OFD.xml".into(),
            br"<ofd:OFD><ofd:DocBody><ofd:DocRoot>Doc_0/Document.xml</ofd:DocRoot></ofd:DocBody></ofd:OFD>".to_vec(),
        )];
        assert!(SingleDocRule.check(&entries).passed);
    }

    #[test]
    fn single_doc_rule_fails_multiple() {
        let entries = vec![(
            "OFD.xml".into(),
            br"<ofd:OFD>
                <ofd:DocBody><ofd:DocRoot>Doc_0/Document.xml</ofd:DocRoot></ofd:DocBody>
                <ofd:DocBody><ofd:DocRoot>Doc_1/Document.xml</ofd:DocRoot></ofd:DocBody>
            </ofd:OFD>"
                .to_vec(),
        )];
        let result = SingleDocRule.check(&entries);
        assert!(!result.passed);
        assert!(result.message.contains('2'));
    }

    #[test]
    fn single_doc_rule_fails_no_ofd_xml() {
        let entries = vec![];
        assert!(!SingleDocRule.check(&entries).passed);
    }
}
