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
}
