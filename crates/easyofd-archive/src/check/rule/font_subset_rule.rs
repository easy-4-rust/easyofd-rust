//! 规则 11：字体必须嵌入子集化字型数据。
//!
//! 对应 Java: org.ofdrw.archive.check.rule.FontSubsetRule
//! GB/T 42133-2022 6.2.6b

use crate::check::archive_violation::ArchiveViolation;
use crate::rules::ComplianceRule;

/// 规则 11：字体必须嵌入子集化字型数据。
///
/// Phase 1 仅检查字体文件是否嵌入，子集化验证留到 Phase 2。
///
/// 对应 Java: org.ofdrw.archive.check.rule.FontSubsetRule
#[derive(Debug, Clone, Copy)]
pub struct FontSubsetRule;

impl ComplianceRule for FontSubsetRule {
    fn name(&self) -> &'static str {
        "FONT_SUBSET"
    }

    fn check(&self, entries: &[(String, Vec<u8>)]) -> crate::rules::RuleResult {
        // 检查是否有字体资源声明
        let has_font_decl = entries.iter().any(|(name, _)| {
            name.ends_with(".xml")
                && entries
                    .iter()
                    .filter(|(n, _)| n.ends_with(".xml"))
                    .any(|(_, data)| {
                        let content = String::from_utf8_lossy(data);
                        content.contains("<ofd:Font") || content.contains("<Font ")
                    })
        });

        if has_font_decl {
            // 检查是否有对应的字体文件
            let has_font_file = entries.iter().any(|(name, _)| {
                name.ends_with(".ttf") || name.ends_with(".otf") || name.ends_with(".ttc")
            });
            if !has_font_file {
                return crate::rules::RuleResult {
                    passed: false,
                    message: "字体声明存在但未找到嵌入的字体文件".into(),
                };
            }
        }

        crate::rules::RuleResult {
            passed: true,
            message: "字体嵌入检查通过".into(),
        }
    }
}

impl FontSubsetRule {
    /// 转为 ArchiveViolation 形式检查。
    pub fn check_violations(&self, _entries: &[(String, Vec<u8>)]) -> Vec<ArchiveViolation> {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn font_subset_rule_name() {
        assert_eq!(FontSubsetRule.name(), "FONT_SUBSET");
    }

    #[test]
    fn font_subset_rule_passes_without_fonts() {
        let entries = vec![("OFD.xml".into(), b"<ofd:OFD/>".to_vec())];
        assert!(FontSubsetRule.check(&entries).passed);
    }

    // ── 合规：有字体声明且有字体文件（ttf） ────────────────────────────

    #[test]
    fn font_subset_rule_passes_with_ttf() {
        let entries = vec![
            (
                "Res.xml".into(),
                br"<ofd:Res><ofd:Font><ofd:FontFile>font.ttf</ofd:FontFile></ofd:Font></ofd:Res>"
                    .to_vec(),
            ),
            ("Res/font.ttf".into(), b"binary_ttf".to_vec()),
        ];
        assert!(FontSubsetRule.check(&entries).passed);
    }

    // ── 合规：有字体声明且有 otf 文件 ──────────────────────────────────

    #[test]
    fn font_subset_rule_passes_with_otf() {
        let entries = vec![
            (
                "Res.xml".into(),
                br"<ofd:Res><ofd:Font><ofd:FontFile>font.otf</ofd:FontFile></ofd:Font></ofd:Res>"
                    .to_vec(),
            ),
            ("Res/font.otf".into(), b"binary_otf".to_vec()),
        ];
        assert!(FontSubsetRule.check(&entries).passed);
    }

    // ── 合规：有字体声明且有 ttc 文件 ──────────────────────────────────

    #[test]
    fn font_subset_rule_passes_with_ttc() {
        let entries = vec![
            (
                "Res.xml".into(),
                br"<ofd:Res><ofd:Font><ofd:FontFile>font.ttc</ofd:FontFile></ofd:Font></ofd:Res>"
                    .to_vec(),
            ),
            ("Res/font.ttc".into(), b"binary_ttc".to_vec()),
        ];
        assert!(FontSubsetRule.check(&entries).passed);
    }

    // ── 违规：有字体声明但无字体文件 ───────────────────────────────────

    #[test]
    fn font_subset_rule_fails_without_font_file() {
        let entries = vec![(
            "Res.xml".into(),
            br"<ofd:Res><ofd:Font><ofd:FontFile>font.ttf</ofd:FontFile></ofd:Font></ofd:Res>"
                .to_vec(),
        )];
        let result = FontSubsetRule.check(&entries);
        assert!(!result.passed);
        assert!(result.message.contains("嵌入"));
    }

    // ── 边界：空条目列表 ───────────────────────────────────────────────

    #[test]
    fn font_subset_rule_passes_empty() {
        assert!(FontSubsetRule.check(&[]).passed);
    }

    // ── 边界：非 XML 文件中含 Font 标签不触发检查 ─────────────────────

    #[test]
    fn font_subset_rule_passes_font_in_non_xml() {
        let entries = vec![("data.bin".into(), b"<ofd:Font>".to_vec())];
        assert!(FontSubsetRule.check(&entries).passed);
    }

    // ── 边界：多 XML 文件，一个含 Font 声明 ───────────────────────────

    #[test]
    fn font_subset_rule_fails_when_font_declared_in_second_xml() {
        let entries = vec![
            ("OFD.xml".into(), b"<ofd:OFD/>".to_vec()),
            (
                "Res.xml".into(),
                br"<ofd:Res><ofd:Font/></ofd:Res>".to_vec(),
            ),
        ];
        let result = FontSubsetRule.check(&entries);
        assert!(!result.passed);
    }

    // ── check_violations 始终返回空（Phase 2） ─────────────────────────

    #[test]
    fn font_subset_violations_always_empty() {
        let entries = vec![(
            "Res.xml".into(),
            br"<ofd:Res><ofd:Font/></ofd:Res>".to_vec(),
        )];
        assert!(FontSubsetRule.check_violations(&entries).is_empty());
    }

    #[test]
    fn font_subset_violations_empty_on_empty() {
        assert!(FontSubsetRule.check_violations(&[]).is_empty());
    }
}
