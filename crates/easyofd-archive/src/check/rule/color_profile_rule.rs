//! 规则 24：颜色空间建议带有颜色配置文件。
//!
//! 对应 Java: org.ofdrw.archive.check.rule.ColorProfileRule
//! GB/T 42133-2022 6.3.1c

use crate::check::archive_violation::ArchiveViolation;
use crate::rules::ComplianceRule;

/// 规则 24：颜色空间建议带有颜色配置文件。
///
/// 仅 INFO 级别提示，不做转换。
///
/// 对应 Java: org.ofdrw.archive.check.rule.ColorProfileRule
#[derive(Debug, Clone, Copy)]
pub struct ColorProfileRule;

impl ComplianceRule for ColorProfileRule {
    fn name(&self) -> &'static str {
        "COLOR_PROFILE"
    }

    fn check(&self, entries: &[(String, Vec<u8>)]) -> crate::rules::RuleResult {
        for (name, data) in entries {
            if name.ends_with(".xml") {
                let content = String::from_utf8_lossy(data);
                if content.contains("<ofd:ColorSpace") || content.contains("<ColorSpace") {
                    if !content.contains("Profile=") && !content.contains("ICCProfile") {
                        return crate::rules::RuleResult {
                            passed: true,
                            message: "颜色空间未带配置文件（建议添加）".into(),
                        };
                    }
                }
            }
        }
        crate::rules::RuleResult {
            passed: true,
            message: "颜色空间配置文件检查通过".into(),
        }
    }
}

impl ColorProfileRule {
    /// 转为 ArchiveViolation 形式检查。
    pub fn check_violations(&self, _entries: &[(String, Vec<u8>)]) -> Vec<ArchiveViolation> {
        // 仅 INFO 级别提示
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_profile_rule_name() {
        assert_eq!(ColorProfileRule.name(), "COLOR_PROFILE");
    }

    #[test]
    fn color_profile_rule_passes_empty() {
        let entries = vec![];
        assert!(ColorProfileRule.check(&entries).passed);
    }

    // ── 合规：无 ColorSpace 元素 ───────────────────────────────────────

    #[test]
    fn color_profile_rule_passes_without_color_space() {
        let entries = vec![(
            "Res.xml".into(),
            br"<ofd:Res><ofd:MultiMedia/></ofd:Res>".to_vec(),
        )];
        assert!(ColorProfileRule.check(&entries).passed);
    }

    // ── 建议：有 ColorSpace 但无 Profile ──────────────────────────────

    #[test]
    fn color_profile_rule_info_without_profile() {
        let entries = vec![(
            "Res.xml".into(),
            b"<ofd:Res><ofd:ColorSpace Type=\"RGB\"/></ofd:Res>".to_vec(),
        )];
        let result = ColorProfileRule.check(&entries);
        assert!(result.passed);
        assert!(result.message.contains("建议"));
    }

    // ── 合规：有 ColorSpace 且带 Profile 属性 ─────────────────────────

    #[test]
    fn color_profile_rule_passes_with_profile_attr() {
        let entries = vec![(
            "Res.xml".into(),
            br#"<ofd:Res><ofd:ColorSpace Type="RGB" Profile="sRGB.icc"/></ofd:Res>"#.to_vec(),
        )];
        let result = ColorProfileRule.check(&entries);
        assert!(result.passed);
        assert!(result.message.contains("通过"));
    }

    // ── 合规：有 ICCProfile 子元素 ─────────────────────────────────────

    #[test]
    fn color_profile_rule_passes_with_icc_profile_element() {
        let entries = vec![(
            "Res.xml".into(),
            b"<ofd:Res><ofd:ColorSpace Type=\"RGB\"><ofd:ICCProfile/></ofd:ColorSpace></ofd:Res>"
                .to_vec(),
        )];
        let result = ColorProfileRule.check(&entries);
        assert!(result.passed);
        assert!(result.message.contains("通过"));
    }

    // ── 边界：非 XML 文件应跳过 ────────────────────────────────────────

    #[test]
    fn color_profile_rule_ignores_non_xml() {
        let entries = vec![("image.png".into(), b"ColorSpace".to_vec())];
        assert!(ColorProfileRule.check(&entries).passed);
    }

    // ── 边界：非 XML 文件中含 ColorSpace 也应跳过 ─────────────────────

    #[test]
    fn color_profile_rule_ignores_non_xml_even_with_keyword() {
        let entries = vec![("data.txt".into(), b"<ofd:ColorSpace>".to_vec())];
        assert!(ColorProfileRule.check(&entries).passed);
    }

    // ── check_violations 始终返回空（仅 INFO 级别） ────────────────────

    #[test]
    fn color_profile_violations_always_empty() {
        let entries = vec![(
            "Res.xml".into(),
            b"<ofd:Res><ofd:ColorSpace Type=\"RGB\"/></ofd:Res>".to_vec(),
        )];
        assert!(ColorProfileRule.check_violations(&entries).is_empty());
    }

    #[test]
    fn color_profile_violations_empty_on_empty_entries() {
        assert!(ColorProfileRule.check_violations(&[]).is_empty());
    }
}
