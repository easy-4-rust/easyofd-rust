//! iText PDF 导出器（排除）。
//!
//! 对应 Java: org.ofdrw.converter.export.PDFExporterIText
//!
//! # 排除原因
//!
//! Java 版 `PDFExporterIText` 使用 iText 7 库的 `PdfDocument`、
//! `PdfWriter`、`PdfCanvas` 等 API 将 OFD 页面导出为 PDF。
//! iText 是 Java/.NET 商业库，无 Rust 等价实现。
//!
//! Rust 版使用 [`PdfExporter`] 替代，底层基于 `printpdf` crate。

/// iText PDF 导出器占位。
///
/// 对应 Java: `org.ofdrw.converter.export.PDFExporterIText`
///
/// **排除**: 依赖 Java iText 库，使用 [`PdfExporter`] 替代。
#[derive(Debug, Clone, Copy)]
pub struct PDFExporterIText;

impl PDFExporterIText {
    /// 返回替代实现的名称。
    #[must_use]
    pub fn replacement() -> &'static str {
        "easyofd_convert::exporter::PdfExporter"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pdf_exporter_itext_exclusion() {
        assert_eq!(
            PDFExporterIText::replacement(),
            "easyofd_convert::exporter::PdfExporter"
        );
    }
}
