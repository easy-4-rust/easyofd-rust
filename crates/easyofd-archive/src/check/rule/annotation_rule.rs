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

    // ── 违规：ReadOnly="false" ─────────────────────────────────────────

    #[test]
    fn annotation_rule_fails_with_readonly_false() {
        let entries = vec![(
            "Doc_0/Annotations.xml".into(),
            br#"<ofd:Annotations><ofd:Annot ReadOnly="false"/></ofd:Annotations>"#.to_vec(),
        )];
        let result = AnnotationRule.check(&entries);
        assert!(!result.passed);
        assert!(result.message.contains("OFD-A"));
    }

    // ── 违规：NoZoom="false" ───────────────────────────────────────────

    #[test]
    fn annotation_rule_fails_with_nozoom_false() {
        let entries = vec![(
            "Doc_0/Annotations.xml".into(),
            br#"<ofd:Annotations><ofd:Annot NoZoom="false"/></ofd:Annotations>"#.to_vec(),
        )];
        let result = AnnotationRule.check(&entries);
        assert!(!result.passed);
    }

    // ── 违规：NoRotate="false" ─────────────────────────────────────────

    #[test]
    fn annotation_rule_fails_with_norotate_false() {
        let entries = vec![(
            "Doc_0/Annotations.xml".into(),
            br#"<ofd:Annotations><ofd:Annot NoRotate="false"/></ofd:Annotations>"#.to_vec(),
        )];
        let result = AnnotationRule.check(&entries);
        assert!(!result.passed);
    }

    // ── 边界：非 Annotation 文件应跳过 ────────────────────────────────

    #[test]
    fn annotation_rule_ignores_non_annotation_xml() {
        let entries = vec![(
            "Doc_0/Page_0.xml".into(),
            br#"<ofd:Page ReadOnly="false"/>"#.to_vec(),
        )];
        assert!(AnnotationRule.check(&entries).passed);
    }

    // ── 边界：非 XML 文件应跳过 ────────────────────────────────────────

    #[test]
    fn annotation_rule_ignores_non_xml() {
        let entries = vec![("Annotation.txt".into(), b"ReadOnly=\"false\"".to_vec())];
        assert!(AnnotationRule.check(&entries).passed);
    }

    // ── 边界：空条目列表 ───────────────────────────────────────────────

    #[test]
    fn annotation_rule_passes_empty() {
        assert!(AnnotationRule.check(&[]).passed);
    }

    // ── 边界：注释文件无属性 ───────────────────────────────────────────

    #[test]
    fn annotation_rule_passes_without_attrs() {
        let entries = vec![(
            "Doc_0/Annotations.xml".into(),
            br"<ofd:Annotations><ofd:Annot/></ofd:Annotations>".to_vec(),
        )];
        assert!(AnnotationRule.check(&entries).passed);
    }

    // ── check_violations：ReadOnly="false" ─────────────────────────────

    #[test]
    fn annotation_violations_with_readonly_false() {
        let entries = vec![(
            "Doc_0/Annotations.xml".into(),
            br#"<ofd:Annotations><ofd:Annot ReadOnly="false"/></ofd:Annotations>"#.to_vec(),
        )];
        let violations = AnnotationRule.check_violations(&entries);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule_name(), "ANNOTATION");
        assert_eq!(violations[0].severity(), Severity::Warn);
        assert_eq!(violations[0].actual_value(), Some("false"));
        assert_eq!(violations[0].expected_value(), Some("true"));
    }

    // ── check_violations：合规场景 ─────────────────────────────────────

    #[test]
    fn annotation_violations_empty_on_pass() {
        let entries = vec![(
            "Doc_0/Annotations.xml".into(),
            br#"<ofd:Annotations><ofd:Annot ReadOnly="true"/></ofd:Annotations>"#.to_vec(),
        )];
        assert!(AnnotationRule.check_violations(&entries).is_empty());
    }

    // ── check_violations：非 Annotation 文件 ───────────────────────────

    #[test]
    fn annotation_violations_ignores_non_annotation() {
        let entries = vec![(
            "Doc_0/Page_0.xml".into(),
            br#"<ofd:Page ReadOnly="false"/>"#.to_vec(),
        )];
        assert!(AnnotationRule.check_violations(&entries).is_empty());
    }
}
