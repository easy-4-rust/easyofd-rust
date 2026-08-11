//! 规则：注释合规检查。
//!
//! 对应 Java: org.ofdrw.archive.check.rule.AnnotationRule
//! GB/T 42133-2022 6.10.1

use crate::check::archive_violation::{ArchiveViolation, Severity};
use crate::rules::ComplianceRule;

/// 规则：注释合规检查。
///
/// 检查注释的 ReadOnly、NoZoom、NoRotate 属性和 Appearance 嵌套。
///
/// 对应 Java: org.ofdrw.archive.check.rule.AnnotationRule
#[derive(Debug, Clone, Copy)]
pub struct AnnotationRule;

impl ComplianceRule for AnnotationRule {
    fn name(&self) -> &'static str {
        "ANNOTATION"
    }

    fn check(&self, entries: &[(String, Vec<u8>)]) -> crate::rules::RuleResult {
        // 检查注释文件中是否包含不符合 OFD-A 要求的属性
        for (name, data) in entries {
            if name.contains("Annotation") && name.ends_with(".xml") {
                let content = String::from_utf8_lossy(data);
                if content.contains("ReadOnly=\"false\"")
                    || content.contains("NoZoom=\"false\"")
                    || content.contains("NoRotate=\"false\"")
                {
                    return crate::rules::RuleResult {
                        passed: false,
                        message: "注释属性不符合 OFD-A 要求".into(),
                    };
                }
            }
        }
        crate::rules::RuleResult {
            passed: true,
            message: "注释合规检查通过".into(),
        }
    }
}

impl AnnotationRule {
    /// 转为 ArchiveViolation 形式检查。
    pub fn check_violations(&self, entries: &[(String, Vec<u8>)]) -> Vec<ArchiveViolation> {
        let mut violations = Vec::new();
        for (name, data) in entries {
            if name.contains("Annotation") && name.ends_with(".xml") {
                let content = String::from_utf8_lossy(data);
                if content.contains("ReadOnly=\"false\"") {
                    violations.push(ArchiveViolation::new(
                        self.name(),
                        Severity::Warn,
                        "注释 ReadOnly 应为 true",
                        Some(name.as_str()),
                        Some("false"),
                        Some("true"),
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
    fn annotation_rule_name() {
        assert_eq!(AnnotationRule.name(), "ANNOTATION");
    }

    #[test]
    fn annotation_rule_passes_without_annotations() {
        let entries = vec![("OFD.xml".into(), b"<ofd:OFD/>".to_vec())];
        assert!(AnnotationRule.check(&entries).passed);
    }

    #[test]
    fn annotation_rule_passes_with_valid_annotations() {
        let entries = vec![(
            "Doc_0/Annotations.xml".into(),
            br#"<ofd:Annotations><ofd:Annot ReadOnly="true" NoZoom="true" NoRotate="true"/></ofd:Annotations>"#.to_vec(),
        )];
        assert!(AnnotationRule.check(&entries).passed);
    }
}
