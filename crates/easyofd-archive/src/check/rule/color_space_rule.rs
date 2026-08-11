//! 规则 9：颜色空间仅限 Gray/RGB/CMYK。
//!
//! 对应 Java: org.ofdrw.archive.check.rule.ColorSpaceRule
//! GB/T 42133-2022 6.3.1b

use crate::check::archive_violation::{ArchiveViolation, Severity};
use crate::rules::ComplianceRule;

/// 允许的颜色空间类型。
const ALLOWED_COLOR_SPACES: &[&str] = &["GRAY", "RGB", "CMYK"];

/// 规则 9：颜色空间仅限 Gray/RGB/CMYK。
///
/// 对应 Java: org.ofdrw.archive.check.rule.ColorSpaceRule
#[derive(Debug, Clone, Copy)]
pub struct ColorSpaceRule;

impl ComplianceRule for ColorSpaceRule {
    fn name(&self) -> &'static str {
        "COLOR_SPACE"
    }

    fn check(&self, entries: &[(String, Vec<u8>)]) -> crate::rules::RuleResult {
        for (name, data) in entries {
            if name.ends_with(".xml") {
                let content = String::from_utf8_lossy(data);
                // 检查 Type 属性
                for part in content.split("Type=\"").skip(1) {
                    if let Some(end) = part.find('"') {
                        let cs_type = &part[..end];
                        if cs_type.starts_with("CMYK")
                            || cs_type.starts_with("RGB")
                            || cs_type.starts_with("GRAY")
                        {
                            continue;
                        }
                        // 其他类型视为不合规
                        if !ALLOWED_COLOR_SPACES.iter().any(|&a| cs_type.starts_with(a)) {
                            return crate::rules::RuleResult {
                                passed: false,
                                message: format!("颜色空间类型 {cs_type} 不符合 OFD-A 要求"),
                            };
                        }
                    }
                }
            }
        }
        crate::rules::RuleResult {
            passed: true,
            message: "颜色空间检查通过".into(),
        }
    }
}

impl ColorSpaceRule {
    /// 转为 ArchiveViolation 形式检查。
    pub fn check_violations(&self, entries: &[(String, Vec<u8>)]) -> Vec<ArchiveViolation> {
        let mut violations = Vec::new();
        for (name, data) in entries {
            if name.ends_with(".xml") {
                let content = String::from_utf8_lossy(data);
                for part in content.split("Type=\"").skip(1) {
                    if let Some(end) = part.find('"') {
                        let cs_type = &part[..end];
                        if !ALLOWED_COLOR_SPACES.iter().any(|&a| cs_type.starts_with(a)) {
                            violations.push(ArchiveViolation::new(
                                self.name(),
                                Severity::Error,
                                format!("颜色空间类型 {cs_type} 不符合 OFD-A 要求"),
                                Some(name.as_str()),
                                Some(cs_type),
                                Some("GRAY/RGB/CMYK"),
                            ));
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
    fn color_space_rule_name() {
        assert_eq!(ColorSpaceRule.name(), "COLOR_SPACE");
    }

    #[test]
    fn color_space_rule_passes_with_rgb() {
        let entries = vec![(
            "Res.xml".into(),
            br#"<ofd:Res><ofd:ColorSpace Type="RGB"/></ofd:Res>"#.to_vec(),
        )];
        assert!(ColorSpaceRule.check(&entries).passed);
    }

    #[test]
    fn color_space_rule_passes_empty() {
        let entries = vec![];
        assert!(ColorSpaceRule.check(&entries).passed);
    }

    // ── 合规：CMYK / GRAY 均应通过 ──────────────────────────────────────

    #[test]
    fn color_space_rule_passes_with_cmyk() {
        let entries = vec![(
            "Res.xml".into(),
            br#"<ofd:Res><ofd:ColorSpace Type="CMYK"/></ofd:Res>"#.to_vec(),
        )];
        assert!(ColorSpaceRule.check(&entries).passed);
    }

    #[test]
    fn color_space_rule_passes_with_gray() {
        let entries = vec![(
            "Res.xml".into(),
            br#"<ofd:Res><ofd:ColorSpace Type="GRAY"/></ofd:Res>"#.to_vec(),
        )];
        assert!(ColorSpaceRule.check(&entries).passed);
    }

    // ── 违规：不合规颜色空间类型 ────────────────────────────────────────

    #[test]
    fn color_space_rule_fails_with_lab() {
        let entries = vec![(
            "Res.xml".into(),
            br#"<ofd:Res><ofd:ColorSpace Type="LAB"/></ofd:Res>"#.to_vec(),
        )];
        let result = ColorSpaceRule.check(&entries);
        assert!(!result.passed);
        assert!(result.message.contains("LAB"));
    }

    #[test]
    fn color_space_rule_fails_with_ycbcr() {
        let entries = vec![(
            "Res.xml".into(),
            br#"<ofd:Res><ofd:ColorSpace Type="YCbCr"/></ofd:Res>"#.to_vec(),
        )];
        let result = ColorSpaceRule.check(&entries);
        assert!(!result.passed);
        assert!(result.message.contains("YCbCr"));
    }

    // ── 边界：非 XML 文件应跳过 ────────────────────────────────────────

    #[test]
    fn color_space_rule_ignores_non_xml() {
        let entries = vec![("image.png".into(), b"binary".to_vec())];
        assert!(ColorSpaceRule.check(&entries).passed);
    }

    // ── 边界：XML 无 Type 属性应通过 ──────────────────────────────────

    #[test]
    fn color_space_rule_passes_without_type_attr() {
        let entries = vec![(
            "Res.xml".into(),
            br"<ofd:Res><ofd:ColorSpace/></ofd:Res>".to_vec(),
        )];
        assert!(ColorSpaceRule.check(&entries).passed);
    }

    // ── 边界：多个条目，首个合规第二个违规 ─────────────────────────────

    #[test]
    fn color_space_rule_fails_on_second_entry() {
        let entries = vec![
            (
                "Res1.xml".into(),
                br#"<ofd:Res><ofd:ColorSpace Type="RGB"/></ofd:Res>"#.to_vec(),
            ),
            (
                "Res2.xml".into(),
                br#"<ofd:Res><ofd:ColorSpace Type="LAB"/></ofd:Res>"#.to_vec(),
            ),
        ];
        let result = ColorSpaceRule.check(&entries);
        assert!(!result.passed);
    }

    // ── check_violations：违规场景 ─────────────────────────────────────

    #[test]
    fn color_space_violations_with_bad_type() {
        let entries = vec![(
            "Res.xml".into(),
            br#"<ofd:Res><ofd:ColorSpace Type="LAB"/></ofd:Res>"#.to_vec(),
        )];
        let violations = ColorSpaceRule.check_violations(&entries);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule_name(), "COLOR_SPACE");
        assert_eq!(violations[0].severity(), Severity::Error);
        assert!(violations[0].location().unwrap().contains("Res.xml"));
        assert!(violations[0].actual_value().unwrap().contains("LAB"));
    }

    // ── check_violations：合规场景 ─────────────────────────────────────

    #[test]
    fn color_space_violations_empty_on_pass() {
        let entries = vec![(
            "Res.xml".into(),
            br#"<ofd:Res><ofd:ColorSpace Type="RGB"/></ofd:Res>"#.to_vec(),
        )];
        assert!(ColorSpaceRule.check_violations(&entries).is_empty());
    }

    // ── check_violations：多个违规 ─────────────────────────────────────

    #[test]
    fn color_space_violations_multiple() {
        let entries = vec![
            (
                "Res1.xml".into(),
                br#"<ofd:Res><ofd:ColorSpace Type="LAB"/></ofd:Res>"#.to_vec(),
            ),
            (
                "Res2.xml".into(),
                br#"<ofd:Res><ofd:ColorSpace Type="HSV"/></ofd:Res>"#.to_vec(),
            ),
        ];
        let violations = ColorSpaceRule.check_violations(&entries);
        assert_eq!(violations.len(), 2);
    }

    // ── check_violations：非 XML 跳过 ─────────────────────────────────

    #[test]
    fn color_space_violations_ignores_non_xml() {
        let entries = vec![("data.bin".into(), b"Type=\"LAB\"".to_vec())];
        assert!(ColorSpaceRule.check_violations(&entries).is_empty());
    }
}
