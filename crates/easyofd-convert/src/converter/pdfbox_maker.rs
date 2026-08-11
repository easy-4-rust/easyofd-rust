//! PDFBox PDF 渲染器（排除）。
//!
//! 对应 Java: org.ofdrw.converter.PdfboxMaker
//!
//! # 排除原因
//!
//! Java 版 `PdfboxMaker` 依赖 Apache PDFBox 库的 `PDPageContentStream`
//! 进行 PDF 内容写入，包括字体嵌入（`PDType0Font`）、图片嵌入
//! （`PDImageXObject`）、路径绘制等 PDFBox 专有 API。
//!
//! Rust 版使用 [`crate::exporter::PdfExporter`] 替代，
//! 底层基于 `printpdf` crate 生成 PDF。
//!
//! # 对应关系
//!
//! | Java 类 | Rust 替代 |
//! |---------|-----------|
//! | `PdfboxMaker` | `PdfExporter` (基于 printpdf) |

/// PDFBox PDF 渲染器占位。
///
/// 对应 Java: `org.ofdrw.converter.PdfboxMaker`
///
/// **排除**: 依赖 Java PDFBox，使用 [`crate::exporter::PdfExporter`] 替代。
#[derive(Debug, Clone, Copy)]
pub struct PdfboxMaker;

impl PdfboxMaker {
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
    fn test_pdfbox_maker_exclusion_doc() {
        assert_eq!(
            PdfboxMaker::replacement(),
            "easyofd_convert::exporter::PdfExporter"
        );
    }
}
