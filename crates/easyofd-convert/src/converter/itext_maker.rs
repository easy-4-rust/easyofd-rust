//! iText PDF 渲染器（排除）。
//!
//! 对应 Java: org.ofdrw.converter.ItextMaker
//!
//! # 排除原因
//!
//! Java 版 `ItextMaker` 依赖 iText 7 库的 `PdfCanvas`、`PdfFont`
//! 等 API 进行 PDF 内容写入和字体嵌入。iText 是 Java/G# 商业库，
//! 无 Rust 等价实现。
//!
//! Rust 版使用 [`crate::exporter::PdfExporter`] 替代，
//! 底层基于 `printpdf` crate 生成 PDF。

/// iText PDF 渲染器占位。
///
/// 对应 Java: `org.ofdrw.converter.ItextMaker`
///
/// **排除**: 依赖 Java iText 库，使用 [`crate::exporter::PdfExporter`] 替代。
#[derive(Debug, Clone, Copy)]
pub struct ItextMaker;

impl ItextMaker {
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
    fn test_itext_maker_exclusion_doc() {
        assert_eq!(
            ItextMaker::replacement(),
            "easyofd_convert::exporter::PdfExporter"
        );
    }
}
