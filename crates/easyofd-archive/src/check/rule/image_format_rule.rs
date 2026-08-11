//! 规则 10：图像格式仅限 BMP/JPEG/PNG/JBIG2/JPEG2000/TIFF。
//!
//! 对应 Java: org.ofdrw.archive.check.rule.ImageFormatRule
//! GB/T 42133-2022 6.2.6e

use crate::check::archive_violation::{ArchiveViolation, Severity};
use crate::rules::ComplianceRule;

/// 允许的图像格式。
const ALLOWED_FORMATS: &[&str] = &["BMP", "JPEG", "PNG", "JBIG2", "JPEG2000", "TIFF"];

/// 规则 10：图像格式仅限 BMP/JPEG/PNG/JBIG2/JPEG2000/TIFF。
///
/// 对应 Java: org.ofdrw.archive.check.rule.ImageFormatRule
#[derive(Debug, Clone, Copy)]
pub struct ImageFormatRule;

impl ComplianceRule for ImageFormatRule {
    fn name(&self) -> &'static str {
        "IMAGE_FORMAT"
    }

    fn check(&self, entries: &[(String, Vec<u8>)]) -> crate::rules::RuleResult {
        for (name, data) in entries {
            if name.ends_with(".xml") {
                let content = String::from_utf8_lossy(data);
                // 检查 MultiMedia 中的 Format 属性
                for part in content.split("Format=\"").skip(1) {
                    if let Some(end) = part.find('"') {
                        let format = &part[..end];
                        if !ALLOWED_FORMATS.contains(&format) {
                            return crate::rules::RuleResult {
                                passed: false,
                                message: format!("图像格式 {format} 不符合 OFD-A 要求"),
                            };
                        }
                    }
                }
            }
        }
        crate::rules::RuleResult {
            passed: true,
            message: "图像格式检查通过".into(),
        }
    }
}

impl ImageFormatRule {
    /// 转为 ArchiveViolation 形式检查。
    pub fn check_violations(&self, entries: &[(String, Vec<u8>)]) -> Vec<ArchiveViolation> {
        let mut violations = Vec::new();
        for (name, data) in entries {
            if name.ends_with(".xml") {
                let content = String::from_utf8_lossy(data);
                for part in content.split("Format=\"").skip(1) {
                    if let Some(end) = part.find('"') {
                        let format = &part[..end];
                        if !ALLOWED_FORMATS.contains(&format) {
                            violations.push(ArchiveViolation::new(
                                self.name(),
                                Severity::Error,
                                format!("图像格式 {format} 不符合 OFD-A 要求"),
                                Some(name.as_str()),
                                Some(format),
                                Some("BMP/JPEG/PNG/JBIG2/JPEG2000/TIFF"),
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
    fn image_format_rule_name() {
        assert_eq!(ImageFormatRule.name(), "IMAGE_FORMAT");
    }

    #[test]
    fn image_format_rule_passes_with_png() {
        let entries = vec![(
            "Res.xml".into(),
            br#"<ofd:Res><ofd:MultiMedia Format="PNG"/></ofd:Res>"#.to_vec(),
        )];
        assert!(ImageFormatRule.check(&entries).passed);
    }

    #[test]
    fn image_format_rule_passes_empty() {
        assert!(ImageFormatRule.check(&[]).passed);
    }

    // ── 合规：所有允许格式 ─────────────────────────────────────────────

    #[test]
    fn image_format_rule_passes_with_bmp() {
        let entries = vec![(
            "Res.xml".into(),
            br#"<ofd:Res><ofd:MultiMedia Format="BMP"/></ofd:Res>"#.to_vec(),
        )];
        assert!(ImageFormatRule.check(&entries).passed);
    }

    #[test]
    fn image_format_rule_passes_with_jpeg() {
        let entries = vec![(
            "Res.xml".into(),
            br#"<ofd:Res><ofd:MultiMedia Format="JPEG"/></ofd:Res>"#.to_vec(),
        )];
        assert!(ImageFormatRule.check(&entries).passed);
    }

    #[test]
    fn image_format_rule_passes_with_jbig2() {
        let entries = vec![(
            "Res.xml".into(),
            br#"<ofd:Res><ofd:MultiMedia Format="JBIG2"/></ofd:Res>"#.to_vec(),
        )];
        assert!(ImageFormatRule.check(&entries).passed);
    }

    #[test]
    fn image_format_rule_passes_with_jpeg2000() {
        let entries = vec![(
            "Res.xml".into(),
            br#"<ofd:Res><ofd:MultiMedia Format="JPEG2000"/></ofd:Res>"#.to_vec(),
        )];
        assert!(ImageFormatRule.check(&entries).passed);
    }

    #[test]
    fn image_format_rule_passes_with_tiff() {
        let entries = vec![(
            "Res.xml".into(),
            br#"<ofd:Res><ofd:MultiMedia Format="TIFF"/></ofd:Res>"#.to_vec(),
        )];
        assert!(ImageFormatRule.check(&entries).passed);
    }

    // ── 违规：GIF 不允许 ───────────────────────────────────────────────

    #[test]
    fn image_format_rule_fails_with_gif() {
        let entries = vec![(
            "Res.xml".into(),
            br#"<ofd:Res><ofd:MultiMedia Format="GIF"/></ofd:Res>"#.to_vec(),
        )];
        let result = ImageFormatRule.check(&entries);
        assert!(!result.passed);
        assert!(result.message.contains("GIF"));
    }

    // ── 违规：WEBP 不允许 ──────────────────────────────────────────────

    #[test]
    fn image_format_rule_fails_with_webp() {
        let entries = vec![(
            "Res.xml".into(),
            br#"<ofd:Res><ofd:MultiMedia Format="WEBP"/></ofd:Res>"#.to_vec(),
        )];
        let result = ImageFormatRule.check(&entries);
        assert!(!result.passed);
        assert!(result.message.contains("WEBP"));
    }

    // ── 边界：非 XML 文件应跳过 ────────────────────────────────────────

    #[test]
    fn image_format_rule_ignores_non_xml() {
        let entries = vec![("image.gif".into(), b"GIF89a".to_vec())];
        assert!(ImageFormatRule.check(&entries).passed);
    }

    // ── 边界：XML 无 Format 属性 ───────────────────────────────────────

    #[test]
    fn image_format_rule_passes_without_format() {
        let entries = vec![(
            "Res.xml".into(),
            br"<ofd:Res><ofd:MultiMedia/></ofd:Res>".to_vec(),
        )];
        assert!(ImageFormatRule.check(&entries).passed);
    }

    // ── check_violations：违规场景 ─────────────────────────────────────

    #[test]
    fn image_format_violations_with_gif() {
        let entries = vec![(
            "Res.xml".into(),
            br#"<ofd:Res><ofd:MultiMedia Format="GIF"/></ofd:Res>"#.to_vec(),
        )];
        let violations = ImageFormatRule.check_violations(&entries);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule_name(), "IMAGE_FORMAT");
        assert_eq!(violations[0].severity(), Severity::Error);
        assert_eq!(violations[0].actual_value(), Some("GIF"));
    }

    // ── check_violations：合规场景 ─────────────────────────────────────

    #[test]
    fn image_format_violations_empty_on_pass() {
        let entries = vec![(
            "Res.xml".into(),
            br#"<ofd:Res><ofd:MultiMedia Format="PNG"/></ofd:Res>"#.to_vec(),
        )];
        assert!(ImageFormatRule.check_violations(&entries).is_empty());
    }

    // ── check_violations：非 XML 跳过 ─────────────────────────────────

    #[test]
    fn image_format_violations_ignores_non_xml() {
        let entries = vec![("image.gif".into(), b"Format=\"GIF\"".to_vec())];
        assert!(ImageFormatRule.check_violations(&entries).is_empty());
    }
}
