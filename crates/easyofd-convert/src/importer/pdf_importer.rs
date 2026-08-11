//! PDF → OFD 导入器。

use std::path::Path;

use easyofd_core::OfdResult;

use super::Importer;
use crate::ConvertOptions;

/// PDF → OFD 导入器。
///
/// 对应 Java: org.ofdrw.converter.pdfconverter.PDFConverter
///
/// 封装现有的 `pdf_to_ofd` 函数，提供面向对象的接口。
pub struct PdfImporter {
    /// 转换选项。
    options: ConvertOptions,
}

impl PdfImporter {
    /// 创建新的 PDF 导入器。
    pub fn new(options: ConvertOptions) -> Self {
        Self { options }
    }

    /// 使用默认选项创建 PDF 导入器。
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

impl Importer for PdfImporter {
    fn convert(&self, source: &Path, target: &Path) -> OfdResult<()> {
        crate::pdf_to_ofd(source, target, &self.options)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 创建一个最小的合法 PDF 文件用于测试。
    fn create_minimal_test_pdf() -> Vec<u8> {
        let mut pdf = Vec::new();

        // PDF header
        pdf.extend_from_slice(b"%PDF-1.4\n");

        // Object 1: Catalog
        let obj1_offset = pdf.len();
        pdf.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");

        // Object 2: Pages
        let obj2_offset = pdf.len();
        pdf.extend_from_slice(b"2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n");

        // Object 3: Page
        let obj3_offset = pdf.len();
        pdf.extend_from_slice(
            b"3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] >>\nendobj\n",
        );

        // Cross-reference table
        let xref_offset = pdf.len();
        pdf.extend_from_slice(b"xref\n");
        pdf.extend_from_slice(b"0 4\n");
        pdf.extend_from_slice(b"0000000000 65535 f \n");
        pdf.extend_from_slice(format!("{obj1_offset:010} 00000 n \n").as_bytes());
        pdf.extend_from_slice(format!("{obj2_offset:010} 00000 n \n").as_bytes());
        pdf.extend_from_slice(format!("{obj3_offset:010} 00000 n \n").as_bytes());

        // Trailer
        pdf.extend_from_slice(b"trailer\n<< /Size 4 /Root 1 0 R >>\n");
        pdf.extend_from_slice(format!("startxref\n{xref_offset}\n%%EOF\n").as_bytes());

        pdf
    }

    #[test]
    fn test_pdf_importer_new() {
        let options = ConvertOptions {
            pages: 0..5,
            page_size: Some((210.0, 297.0)),
        };
        let importer = PdfImporter::new(options);
        assert_eq!(importer.options().pages, 0..5);
        assert_eq!(importer.options().page_size, Some((210.0, 297.0)));
    }

    #[test]
    fn test_pdf_importer_with_defaults() {
        let importer = PdfImporter::with_defaults();
        assert!(importer.options().pages.is_empty());
        assert!(importer.options().page_size.is_none());
    }

    #[test]
    fn test_pdf_importer_convert() {
        let pdf_path = "/tmp/test_pdf_importer.pdf";
        let ofd_path = "/tmp/test_pdf_importer.ofd";

        let pdf_data = create_minimal_test_pdf();
        std::fs::write(pdf_path, &pdf_data).unwrap();

        let importer = PdfImporter::with_defaults();
        let result = importer.convert(Path::new(pdf_path), Path::new(ofd_path));
        assert!(result.is_ok(), "PDF 导入应该成功: {:?}", result.err());
        assert!(Path::new(ofd_path).exists());

        let _ = std::fs::remove_file(pdf_path);
        let _ = std::fs::remove_file(ofd_path);
    }

    #[test]
    fn test_pdf_importer_set_options() {
        let mut importer = PdfImporter::with_defaults();
        assert!(importer.options().pages.is_empty());

        let options = ConvertOptions {
            pages: 2..8,
            page_size: Some((100.0, 150.0)),
        };
        importer.set_options(options);
        assert_eq!(importer.options().pages, 2..8);
        assert_eq!(importer.options().page_size, Some((100.0, 150.0)));
    }

    #[test]
    fn test_pdf_importer_missing_file() {
        let importer = PdfImporter::with_defaults();
        let result = importer.convert(Path::new("/nonexistent.pdf"), Path::new("/tmp/out.ofd"));
        assert!(result.is_err());
    }
}
