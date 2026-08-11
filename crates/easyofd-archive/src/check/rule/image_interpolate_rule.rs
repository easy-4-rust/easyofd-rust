//! 规则 17：图像对象插值绘制必须为 false。
//!
//! 对应 Java: org.ofdrw.archive.check.rule.ImageInterpolateRule
//! GB/T 42133-2022 6.5b

use crate::check::archive_violation::{ArchiveViolation, Severity};
use crate::rules::ComplianceRule;

/// 规则 17：图像对象插值绘制必须为 false。
///
/// 对应 Java: org.ofdrw.archive.check.rule.ImageInterpolateRule
#[derive(Debug, Clone, Copy)]
pub struct ImageInterpolateRule;

impl ComplianceRule for ImageInterpolateRule {
    fn name(&self) -> &'static str {
        "IMAGE_INTERPOLATE"
    }

    fn check(&self, entries: &[(String, Vec<u8>)]) -> crate::rules::RuleResult {
        for (name, data) in entries {
            if name.ends_with(".xml") {
                let content = String::from_utf8_lossy(data);
                // 检查 ImageObject 的 Interpolate 属性
                if content.contains("Interpolate=\"true\"")
                    || content.contains("Interpolate=\"True\"")
                {
                    return crate::rules::RuleResult {
                        passed: false,
                        message: "图像对象 Interpolate=true，OFD-A 要求设置为 false".into(),
                    };
                }
            }
        }
        crate::rules::RuleResult {
            passed: true,
            message: "图像插值检查通过".into(),
        }
    }
}

impl ImageInterpolateRule {
    /// 转为 ArchiveViolation 形式检查。
    pub fn check_violations(&self, entries: &[(String, Vec<u8>)]) -> Vec<ArchiveViolation> {
        let mut violations = Vec::new();
        for (name, data) in entries {
            if name.ends_with(".xml") {
                let content = String::from_utf8_lossy(data);
                if content.contains("Interpolate=\"true\"")
                    || content.contains("Interpolate=\"True\"")
                {
                    violations.push(ArchiveViolation::new(
                        self.name(),
                        Severity::Warn,
                        "图像对象 Interpolate=true，OFD-A 要求设置为 false",
                        Some(name.as_str()),
                        Some("true"),
                        Some("false"),
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
    fn image_interpolate_rule_name() {
        assert_eq!(ImageInterpolateRule.name(), "IMAGE_INTERPOLATE");
    }

    #[test]
    fn image_interpolate_rule_passes_without_interpolate() {
        let entries = vec![(
            "Page_0.xml".into(),
            br"<ofd:Page><ofd:ImageObject/></ofd:Page>".to_vec(),
        )];
        assert!(ImageInterpolateRule.check(&entries).passed);
    }

    #[test]
    fn image_interpolate_rule_fails_with_true() {
        let entries = vec![(
            "Page_0.xml".into(),
            br#"<ofd:Page><ofd:ImageObject Interpolate="true"/></ofd:Page>"#.to_vec(),
        )];
        assert!(!ImageInterpolateRule.check(&entries).passed);
    }

    // ── 违规：大写 True ───────────────────────────────────────────────

    #[test]
    fn image_interpolate_rule_fails_with_true_uppercase() {
        let entries = vec![(
            "Page_0.xml".into(),
            br#"<ofd:Page><ofd:ImageObject Interpolate="True"/></ofd:Page>"#.to_vec(),
        )];
        let result = ImageInterpolateRule.check(&entries);
        assert!(!result.passed);
        assert!(result.message.contains("Interpolate"));
    }

    // ── 合规：Interpolate="false" ─────────────────────────────────────

    #[test]
    fn image_interpolate_rule_passes_with_false() {
        let entries = vec![(
            "Page_0.xml".into(),
            br#"<ofd:Page><ofd:ImageObject Interpolate="false"/></ofd:Page>"#.to_vec(),
        )];
        assert!(ImageInterpolateRule.check(&entries).passed);
    }

    // ── 边界：非 XML 文件应跳过 ────────────────────────────────────────

    #[test]
    fn image_interpolate_rule_ignores_non_xml() {
        let entries = vec![("image.png".into(), b"Interpolate=\"true\"".to_vec())];
        assert!(ImageInterpolateRule.check(&entries).passed);
    }

    // ── 边界：空条目列表 ───────────────────────────────────────────────

    #[test]
    fn image_interpolate_rule_passes_empty() {
        assert!(ImageInterpolateRule.check(&[]).passed);
    }

    // ── check_violations：true 违规 ────────────────────────────────────

    #[test]
    fn image_interpolate_violations_with_true() {
        let entries = vec![(
            "Page_0.xml".into(),
            br#"<ofd:Page><ofd:ImageObject Interpolate="true"/></ofd:Page>"#.to_vec(),
        )];
        let violations = ImageInterpolateRule.check_violations(&entries);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule_name(), "IMAGE_INTERPOLATE");
        assert_eq!(violations[0].severity(), Severity::Warn);
        assert_eq!(violations[0].actual_value(), Some("true"));
        assert_eq!(violations[0].expected_value(), Some("false"));
    }

    // ── check_violations：True 违规（大写） ────────────────────────────

    #[test]
    fn image_interpolate_violations_with_true_uppercase() {
        let entries = vec![(
            "Page_0.xml".into(),
            br#"<ofd:Page><ofd:ImageObject Interpolate="True"/></ofd:Page>"#.to_vec(),
        )];
        let violations = ImageInterpolateRule.check_violations(&entries);
        assert_eq!(violations.len(), 1);
    }

    // ── check_violations：合规 ─────────────────────────────────────────

    #[test]
    fn image_interpolate_violations_empty_on_pass() {
        let entries = vec![(
            "Page_0.xml".into(),
            br#"<ofd:Page><ofd:ImageObject Interpolate="false"/></ofd:Page>"#.to_vec(),
        )];
        assert!(ImageInterpolateRule.check_violations(&entries).is_empty());
    }

    // ── check_violations：非 XML 跳过 ─────────────────────────────────

    #[test]
    fn image_interpolate_violations_ignores_non_xml() {
        let entries = vec![("data.bin".into(), b"Interpolate=\"true\"".to_vec())];
        assert!(ImageInterpolateRule.check_violations(&entries).is_empty());
    }
}
