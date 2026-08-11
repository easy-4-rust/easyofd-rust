//! 转换库枚举。
//!
//! 对应 Java: org.ofdrw.converter.ConvertHelper.Lib
//!
//! Java 版 `ConvertHelper` 内部枚举 `Lib` 标识使用哪个 PDF 渲染后端。
//! Rust 版提供等价的枚举。

/// PDF 渲染后端枚举。
///
/// 对应 Java: `org.ofdrw.converter.ConvertHelper.Lib`
///
/// 标识 OFD→PDF 转换时使用的 PDF 生成库。
/// Java 版支持 iText 和 PDFBox 两种后端；
/// Rust 版使用 printpdf（基于 lopdf）作为默认后端。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Lib {
    /// iText PDF 库（Java 专有，Rust 中不直接可用）。
    ///
    /// 对应 Java: `Lib.iText`
    Itext,
    /// PDFBox 库（Java 专有，Rust 中不直接可用）。
    ///
    /// 对应 Java: `Lib.PDFBox`
    PdfBox,
    /// printpdf 库（Rust 原生 PDF 生成库）。
    ///
    /// Rust 版默认使用的后端。
    PrintPdf,
}

impl Lib {
    /// 返回库的可读名称。
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::Itext => "iText",
            Self::PdfBox => "PDFBox",
            Self::PrintPdf => "printpdf",
        }
    }

    /// 返回 Rust 版默认使用的后端。
    #[must_use]
    pub fn default_backend() -> Self {
        Self::PrintPdf
    }
}

impl std::fmt::Display for Lib {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lib_name() {
        assert_eq!(Lib::Itext.name(), "iText");
        assert_eq!(Lib::PdfBox.name(), "PDFBox");
        assert_eq!(Lib::PrintPdf.name(), "printpdf");
    }

    #[test]
    fn test_lib_display() {
        assert_eq!(Lib::Itext.to_string(), "iText");
        assert_eq!(Lib::PrintPdf.to_string(), "printpdf");
    }

    #[test]
    fn test_default_backend() {
        assert_eq!(Lib::default_backend(), Lib::PrintPdf);
    }

    #[test]
    fn test_lib_clone_copy() {
        let l1 = Lib::Itext;
        let l2 = l1;
        assert_eq!(l1, l2);
    }

    #[test]
    fn test_lib_ne() {
        assert_ne!(Lib::Itext, Lib::PdfBox);
        assert_ne!(Lib::PdfBox, Lib::PrintPdf);
    }
}
