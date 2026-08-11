//! 规则 12：禁止音频视频资源。
//!
//! 对应 Java: org.ofdrw.archive.check.rule.AudioVideoRule
//! GB/T 42133-2022 6.2.6g

use crate::check::archive_violation::ArchiveViolation;
use crate::rules::ComplianceRule;

/// 规则 12：禁止音频视频资源。
///
/// 对应 Java: org.ofdrw.archive.check.rule.AudioVideoRule
#[derive(Debug, Clone, Copy)]
pub struct AudioVideoRule;

impl ComplianceRule for AudioVideoRule {
    fn name(&self) -> &'static str {
        "AUDIO_VIDEO"
    }

    fn check(&self, entries: &[(String, Vec<u8>)]) -> crate::rules::RuleResult {
        for (name, data) in entries {
            if name.ends_with(".xml") {
                let content = String::from_utf8_lossy(data);
                if content.contains("Type=\"Audio\"") || content.contains("Type=\"Video\"") {
                    return crate::rules::RuleResult {
                        passed: false,
                        message: "文档包含音频/视频资源，OFD-A 禁止".into(),
                    };
                }
            }
        }
        crate::rules::RuleResult {
            passed: true,
            message: "未发现音频/视频资源".into(),
        }
    }
}

impl AudioVideoRule {
    /// 转为 ArchiveViolation 形式检查。
    pub fn check_violations(&self, entries: &[(String, Vec<u8>)]) -> Vec<ArchiveViolation> {
        for (name, data) in entries {
            if name.ends_with(".xml") {
                let content = String::from_utf8_lossy(data);
                if content.contains("Type=\"Audio\"") || content.contains("Type=\"Video\"") {
                    return vec![ArchiveViolation::error(
                        self.name(),
                        "文档包含音频/视频资源，OFD-A 禁止",
                    )];
                }
            }
        }
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn audio_video_rule_name() {
        assert_eq!(AudioVideoRule.name(), "AUDIO_VIDEO");
    }

    #[test]
    fn audio_video_rule_passes_without_av() {
        let entries = vec![("Res.xml".into(), b"<ofd:Res/>".to_vec())];
        assert!(AudioVideoRule.check(&entries).passed);
    }

    #[test]
    fn audio_video_rule_fails_with_audio() {
        let entries = vec![(
            "Res.xml".into(),
            br#"<ofd:Res><ofd:MultiMedia Type="Audio"/></ofd:Res>"#.to_vec(),
        )];
        assert!(!AudioVideoRule.check(&entries).passed);
    }
}
