//! # `easyofd-archive`
//!
//! OFD 归档合规规则引擎，对应 Java 版 [`ofdrw-archive`](https://github.com/ofdrw/ofdrw) 子项目。
//!
//! ## 功能模块
//!
//! - [`integrity`] — OFD 文件完整性保护（SM3 / SHA-256 摘要校验）
//! - [`rules`] — 合规规则引擎（DocType、Version、DocRoot、Pages、外部资源）
//!
//! ## 快速使用
//!
//! ```rust
//! use easyofd_archive::{check_compliance, verify_integrity};
//!
//! # fn example(ofd_bytes: &[u8]) -> easyofd_core::OfdResult<()> {
//! // 完整性校验
//! let report = verify_integrity(ofd_bytes)?;
//! assert!(report.passed, "完整性校验失败: {:?}", report);
//!
//! // 合规规则校验
//! let results = check_compliance(ofd_bytes)?;
//! for r in &results {
//!     assert!(r.passed, "规则 {} 未通过: {}", "", r.message);
//! }
//! # Ok(())
//! # }
//! ```

pub mod check;
pub mod convert;
pub mod integrity;
pub mod pkg;
pub mod rules;

use std::io::{Cursor, Read};

use easyofd_core::{OfdError, OfdResult};

pub use check::{
    ArchiveRule, ArchiveViolation, OfdArchiveChecker, Severity,
    rule::{
        AnnotationRule, AttachmentRule, AudioVideoRule, ClipAreaRule, ColorProfileRule,
        ColorSpaceRule, ExtensionRule, ExternalResourceRule, FontSubsetRule, ImageExtensionRule,
        ImageFormatRule, ImageInterpolateRule, ImageResourceRegRule, NonGotoActionRule,
        OutlineActionRule, PageBlockDepthRule, PermissionRule, ResourcePlacementRule,
        SingleDocRule, TextHScaleRule, TextSizeRule,
    },
};
pub use convert::{
    ArchiveHandler, OfdArchiveConverter,
    handler::{
        AnnotationHandler as ConvertAnnotationHandler,
        AttachmentHandler as ConvertAttachmentHandler,
        AudioVideoHandler as ConvertAudioVideoHandler, CleanFillAttrHandler,
        CleanStrokeAttrHandler, ClipAreaHandler as ConvertClipAreaHandler,
        DocTypeHandler as ConvertDocTypeHandler, EncryptionHandler,
        ExtensionHandler as ConvertExtensionHandler,
        ExternalResourceHandler as ConvertExternalResourceHandler, ImageConvertHandler,
        ImageExtensionHandler as ConvertImageExtensionHandler, ImageInterpolateHandler,
        ImageResourceRegHandler as ConvertImageResourceRegHandler, LayerNameHandler,
        NonGotoActionHandler as ConvertNonGotoActionHandler,
        OutlineActionHandler as ConvertOutlineActionHandler, PageBlockFlattenHandler,
        PermissionHandler as ConvertPermissionHandler,
        ResourcePlacementHandler as ConvertResourcePlacementHandler, SignatureHandler,
        SingleDocHandler as ConvertSingleDocHandler, VPrefsHandler,
    },
};
pub use integrity::{CheckMethod, IntegrityEntry, IntegrityReport, verify_integrity};
pub use pkg::container::{
    AnnotsDir, OfdPkgDir, PageDir, PagesDir, ResDir, TempsDir, VirtualContainer,
};
pub use pkg::tool::{ElemCup, OfdNameSpaceModifier};
pub use rules::font_rule::FontRule;
pub use rules::image_rule::ImageRule;
pub use rules::path_rule::PathRule;
pub use rules::signature_rule::SignatureRule;
pub use rules::text_rule::TextRule;
pub use rules::{
    ComplianceRule, DocRootRule, DocTypeRule, NoExternalResourceRule, PagesExistRule, RuleResult,
    VersionRule,
};

/// 返回模块标识，用于运行时识别。
#[must_use]
pub fn module_name() -> &'static str {
    "easyofd-archive"
}

/// 运行全部合规规则。
///
/// 打开 OFD ZIP 归档，提取所有文件条目，依次执行五条基础规则：
/// [`DocTypeRule`]、[`VersionRule`]、[`DocRootRule`]、[`PagesExistRule`]、
/// [`NoExternalResourceRule`]。
///
/// 返回每个规则的 [`RuleResult`] 列表。
///
/// # Errors
///
/// 当 ZIP 解析失败时返回错误。
pub fn check_compliance(ofd_bytes: &[u8]) -> OfdResult<Vec<RuleResult>> {
    let entries = read_all_entries(ofd_bytes)?;

    let rule_list: Vec<Box<dyn ComplianceRule>> = vec![
        Box::new(DocTypeRule),
        Box::new(VersionRule),
        Box::new(DocRootRule),
        Box::new(PagesExistRule),
        Box::new(NoExternalResourceRule),
    ];

    Ok(rule_list.iter().map(|rule| rule.check(&entries)).collect())
}

/// 读取 ZIP 归档中所有文件条目。
fn read_all_entries(ofd_bytes: &[u8]) -> OfdResult<Vec<(String, Vec<u8>)>> {
    let mut archive =
        zip::ZipArchive::new(Cursor::new(ofd_bytes)).map_err(|e| OfdError::Zip(e.to_string()))?;

    let mut entries = Vec::new();
    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| OfdError::Zip(e.to_string()))?;
        let name = file.name().to_string();
        let mut data = Vec::new();
        file.read_to_end(&mut data).map_err(OfdError::Io)?;
        entries.push((name, data));
    }
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_module_name() {
        assert_eq!(module_name(), "easyofd-archive");
    }

    /// 构建一个合规的最小 OFD ZIP。
    fn build_compliant_zip() -> Vec<u8> {
        let mut buf = Cursor::new(Vec::new());
        {
            let mut zip = zip::ZipWriter::new(&mut buf);
            let options = zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Stored);

            zip.start_file("OFD.xml", options).unwrap();
            zip.write_all(
                br#"<?xml version="1.0" encoding="UTF-8"?>
<ofd:OFD xmlns:ofd="http://www.ofdspec.org/2016" DocType="OFD" Version="1.2">
  <ofd:DocBody><ofd:DocRoot>Doc_0/Document.xml</ofd:DocRoot></ofd:DocBody>
</ofd:OFD>"#,
            )
            .unwrap();

            zip.start_file("Doc_0/Document.xml", options).unwrap();
            zip.write_all(
                br#"<?xml version="1.0" encoding="UTF-8"?>
<ofd:Document xmlns:ofd="http://www.ofdspec.org/2016">
  <ofd:Pages><ofd:Page ID="1" BaseLoc="Pages/Page_0.xml"/></ofd:Pages>
</ofd:Document>"#,
            )
            .unwrap();

            zip.start_file("Doc_0/Pages/Page_0.xml", options).unwrap();
            zip.write_all(b"<ofd:Page/>").unwrap();

            zip.finish().unwrap();
        }
        buf.into_inner()
    }

    /// 构建一个不合规的 OFD ZIP（缺少 DocType、版本错误）。
    fn build_non_compliant_zip() -> Vec<u8> {
        let mut buf = Cursor::new(Vec::new());
        {
            let mut zip = zip::ZipWriter::new(&mut buf);
            let options = zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Stored);

            zip.start_file("OFD.xml", options).unwrap();
            zip.write_all(
                br#"<?xml version="1.0" encoding="UTF-8"?>
<ofd:OFD xmlns:ofd="http://www.ofdspec.org/2016" Version="1.0">
  <ofd:DocBody><ofd:DocRoot>Doc_0/Document.xml</ofd:DocRoot></ofd:DocBody>
</ofd:OFD>"#,
            )
            .unwrap();

            zip.start_file("Doc_0/Document.xml", options).unwrap();
            zip.write_all(
                br#"<?xml version="1.0" encoding="UTF-8"?>
<ofd:Document xmlns:ofd="http://www.ofdspec.org/2016">
  <ofd:Pages><ofd:Page ID="1" BaseLoc="Pages/Page_0.xml"/></ofd:Pages>
</ofd:Document>"#,
            )
            .unwrap();

            zip.start_file("Doc_0/Pages/Page_0.xml", options).unwrap();
            zip.write_all(b"<ofd:Page/>").unwrap();

            zip.finish().unwrap();
        }
        buf.into_inner()
    }

    #[test]
    fn check_compliance_passes_for_compliant_ofd() {
        let bytes = build_compliant_zip();
        let results = check_compliance(&bytes).unwrap();
        assert_eq!(results.len(), 5);
        for r in &results {
            assert!(r.passed, "rule failed: {}", r.message);
        }
    }

    #[test]
    fn check_compliance_detects_non_compliant_ofd() {
        let bytes = build_non_compliant_zip();
        let results = check_compliance(&bytes).unwrap();
        assert_eq!(results.len(), 5);

        // DocTypeRule 应失败
        assert!(!results[0].passed, "DocTypeRule should fail");
        // VersionRule 应失败（1.0 != 1.2）
        assert!(!results[1].passed, "VersionRule should fail");
        // DocRootRule 应通过
        assert!(results[2].passed, "DocRootRule should pass");
        // PagesExistRule 应通过
        assert!(results[3].passed, "PagesExistRule should pass");
        // NoExternalResourceRule 应通过
        assert!(results[4].passed, "NoExternalResourceRule should pass");
    }

    #[test]
    fn check_compliance_fails_on_invalid_zip() {
        let result = check_compliance(b"not a zip");
        assert!(result.is_err());
    }

    #[test]
    fn verify_integrity_works_through_lib() {
        let bytes = build_compliant_zip();
        let report = verify_integrity(&bytes).unwrap();
        assert!(report.passed);
    }
}
