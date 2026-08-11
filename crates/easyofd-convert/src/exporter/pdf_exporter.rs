//! OFD → PDF 导出器。

use std::path::Path;

use easyofd_core::OfdResult;

use super::Exporter;
use crate::ConvertOptions;

/// OFD → PDF 导出器。
///
/// 对应 Java: org.ofdrw.converter.ofdconverter.PDFConverter
///
/// 封装现有的 `ofd_to_pdf` 函数，提供面向对象的接口。
pub struct PdfExporter {
    /// 转换选项。
    options: ConvertOptions,
}

impl PdfExporter {
    /// 创建新的 PDF 导出器。
    pub fn new(options: ConvertOptions) -> Self {
        Self { options }
    }

    /// 使用默认选项创建 PDF 导出器。
    pub fn with_defaults() -> Self {
        Self {
            options: ConvertOptions::default(),
        }
    }

    /// 获取转换选项的引用。
    pub fn options(&self) -> &ConvertOptions {
        &self.options
    }

    /// 设置转换选项。
    pub fn set_options(&mut self, options: ConvertOptions) {
        self.options = options;
    }
}

impl Exporter for PdfExporter {
    fn convert(&self, source: &Path, target: &Path) -> OfdResult<()> {
        crate::ofd_to_pdf(source, target, &self.options)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use easyofd_core::{OfdPage, TextObject};
    use easyofd_writer::OfdWriter;

    #[test]
    fn test_pdf_exporter_new() {
        let options = ConvertOptions {
            pages: 0..5,
            page_size: Some((210.0, 297.0)),
        };
        let exporter = PdfExporter::new(options.clone());
        assert_eq!(exporter.options().pages, 0..5);
        assert_eq!(exporter.options().page_size, Some((210.0, 297.0)));
    }

    #[test]
    fn test_pdf_exporter_with_defaults() {
        let exporter = PdfExporter::with_defaults();
        assert!(exporter.options().pages.is_empty());
        assert!(exporter.options().page_size.is_none());
    }

    #[test]
    fn test_pdf_exporter_convert() {
        let mut writer = OfdWriter::new();
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_text(TextObject::new(10.0, 20.0, "PDF Exporter 测试"));
        writer.add_page(page);
        let ofd_bytes = writer.build().unwrap();

        let ofd_path = "/tmp/test_pdf_exporter.ofd";
        let pdf_path = "/tmp/test_pdf_exporter.pdf";
        std::fs::write(ofd_path, &ofd_bytes).unwrap();

        let exporter = PdfExporter::with_defaults();
        let result = exporter.convert(Path::new(ofd_path), Path::new(pdf_path));
        assert!(result.is_ok(), "PDF 导出应该成功: {:?}", result.err());
        assert!(Path::new(pdf_path).exists());

        let _ = std::fs::remove_file(ofd_path);
        let _ = std::fs::remove_file(pdf_path);
    }

    #[test]
    fn test_pdf_exporter_set_options() {
        let mut exporter = PdfExporter::with_defaults();
        assert!(exporter.options().pages.is_empty());

        let options = ConvertOptions {
            pages: 2..8,
            page_size: Some((100.0, 150.0)),
        };
        exporter.set_options(options);
        assert_eq!(exporter.options().pages, 2..8);
        assert_eq!(exporter.options().page_size, Some((100.0, 150.0)));
    }
}
