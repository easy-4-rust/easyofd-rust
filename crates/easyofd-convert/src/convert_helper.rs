//! 统一转换入口。

use std::path::Path;

use easyofd_core::OfdResult;

use crate::exporter::{Exporter, PdfExporter, SvgExporter, TextExporter};
use crate::importer::{Importer, PdfImporter};

/// 统一转换助手。
///
/// 对应 Java: org.ofdrw.converter.ConverterHelper
///
/// 提供静态方法形式的转换接口，内部委托给具体的导入器/导出器实现。
pub struct ConvertHelper;

impl ConvertHelper {
    /// OFD → PDF 转换。
    ///
    /// # 参数
    ///
    /// - `ofd`: 源 OFD 文件路径
    /// - `pdf`: 目标 PDF 文件路径
    ///
    /// # 错误
    ///
    /// 如果转换失败则返回错误。
    pub fn ofd_to_pdf(ofd: impl AsRef<Path>, pdf: impl AsRef<Path>) -> OfdResult<()> {
        let exporter = PdfExporter::with_defaults();
        exporter.convert(ofd.as_ref(), pdf.as_ref())
    }

    /// PDF → OFD 转换。
    ///
    /// # 参数
    ///
    /// - `pdf`: 源 PDF 文件路径
    /// - `ofd`: 目标 OFD 文件路径
    ///
    /// # 错误
    ///
    /// 如果转换失败则返回错误。
    pub fn pdf_to_ofd(pdf: impl AsRef<Path>, ofd: impl AsRef<Path>) -> OfdResult<()> {
        let importer = PdfImporter::with_defaults();
        importer.convert(pdf.as_ref(), ofd.as_ref())
    }

    /// OFD → 纯文本 转换。
    ///
    /// # 参数
    ///
    /// - `ofd`: 源 OFD 文件路径
    /// - `txt`: 目标文本文件路径
    ///
    /// # 错误
    ///
    /// 如果转换失败则返回错误。
    pub fn ofd_to_text(ofd: impl AsRef<Path>, txt: impl AsRef<Path>) -> OfdResult<()> {
        let exporter = TextExporter::with_defaults();
        exporter.convert(ofd.as_ref(), txt.as_ref())
    }

    /// OFD → SVG 转换。
    ///
    /// # 参数
    ///
    /// - `ofd`: 源 OFD 文件路径
    /// - `svg`: 目标 SVG 文件路径
    ///
    /// # 错误
    ///
    /// 如果转换失败则返回错误。
    pub fn ofd_to_svg(ofd: impl AsRef<Path>, svg: impl AsRef<Path>) -> OfdResult<()> {
        let exporter = SvgExporter::with_defaults();
        exporter.convert(ofd.as_ref(), svg.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use easyofd_core::{OfdPage, PathObject, TextObject};
    use easyofd_writer::OfdWriter;

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
    fn test_ofd_to_pdf() {
        let mut writer = OfdWriter::new();
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_text(TextObject::new(10.0, 20.0, "OFD to PDF 测试"));
        writer.add_page(page);
        let ofd_bytes = writer.build().unwrap();

        let ofd_path = "/tmp/test_helper_ofd_to_pdf.ofd";
        let pdf_path = "/tmp/test_helper_ofd_to_pdf.pdf";
        std::fs::write(ofd_path, &ofd_bytes).unwrap();

        let result = ConvertHelper::ofd_to_pdf(ofd_path, pdf_path);
        assert!(result.is_ok(), "OFD → PDF 应该成功: {:?}", result.err());
        assert!(Path::new(pdf_path).exists());

        let _ = std::fs::remove_file(ofd_path);
        let _ = std::fs::remove_file(pdf_path);
    }

    #[test]
    fn test_ofd_to_text() {
        let mut writer = OfdWriter::new();
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_text(TextObject::new(10.0, 20.0, "文本提取测试"));
        writer.add_page(page);
        let ofd_bytes = writer.build().unwrap();

        let ofd_path = "/tmp/test_helper_ofd_to_text.ofd";
        let txt_path = "/tmp/test_helper_ofd_to_text.txt";
        std::fs::write(ofd_path, &ofd_bytes).unwrap();

        let result = ConvertHelper::ofd_to_text(ofd_path, txt_path);
        assert!(result.is_ok(), "OFD → 纯文本应该成功: {:?}", result.err());
        assert!(Path::new(txt_path).exists());

        let output = std::fs::read_to_string(txt_path).unwrap();
        assert!(output.contains("文本提取测试"));

        let _ = std::fs::remove_file(ofd_path);
        let _ = std::fs::remove_file(txt_path);
    }

    #[test]
    fn test_ofd_to_svg() {
        let mut writer = OfdWriter::new();
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_text(TextObject::new(10.0, 20.0, "SVG 测试"));
        page.add_path(PathObject::new(0.0, 0.0, "M 10 10 L 100 100"));
        writer.add_page(page);
        let ofd_bytes = writer.build().unwrap();

        let ofd_path = "/tmp/test_helper_ofd_to_svg.ofd";
        let svg_path = "/tmp/test_helper_ofd_to_svg.svg";
        std::fs::write(ofd_path, &ofd_bytes).unwrap();

        let result = ConvertHelper::ofd_to_svg(ofd_path, svg_path);
        assert!(result.is_ok(), "OFD → SVG 应该成功: {:?}", result.err());
        assert!(Path::new(svg_path).exists());

        let output = std::fs::read_to_string(svg_path).unwrap();
        assert!(output.contains("<svg"));
        assert!(output.contains("SVG 测试"));

        let _ = std::fs::remove_file(ofd_path);
        let _ = std::fs::remove_file(svg_path);
    }

    #[test]
    fn test_pdf_to_ofd() {
        let pdf_path = "/tmp/test_helper_pdf_to_ofd.pdf";
        let ofd_path = "/tmp/test_helper_pdf_to_ofd.ofd";

        let pdf_data = create_minimal_test_pdf();
        std::fs::write(pdf_path, &pdf_data).unwrap();

        let result = ConvertHelper::pdf_to_ofd(pdf_path, ofd_path);
        assert!(result.is_ok(), "PDF → OFD 应该成功: {:?}", result.err());
        assert!(Path::new(ofd_path).exists());

        let _ = std::fs::remove_file(pdf_path);
        let _ = std::fs::remove_file(ofd_path);
    }
}
