//! PDFBox PDF 导出器（排除）。
//!
//! 对应 Java: org.ofdrw.converter.export.PDFExporterPDFBox
//!
//! # 排除原因
//!
//! Java 版 `PDFExporterPDFBox` 使用 Apache PDFBox 库的 `PDDocument`、
//! `PDPage`、`PDPageContentStream` 等 API 将 OFD 页面导出为 PDF。
//! PDFBox 是 Java 专有库，无 Rust 等价实现。
//!
//! Rust 版使用 [`PdfExporter`] 替代，底层基于 `printpdf` crate。

/// PDFBox PDF 导出器占位。
///
/// 对应 Java: `org.ofdrw.converter.export.PDFExporterPDFBox`
///
/// **排除**: 依赖 Java PDFBox 库，使用 [`PdfExporter`] 替代。
#[derive(Debug, Clone, Copy)]
pub struct PDFExporterPDFBox;

impl PDFExporterPDFBox {
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
    fn test_pdf_exporter_pdfbox_exclusion() {
        assert_eq!(
            PDFExporterPDFBox::replacement(),
            "easyofd_convert::exporter::PdfExporter"
        );
    }
}
