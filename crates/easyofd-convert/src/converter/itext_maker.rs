//! iText PDF 渲染器。
//!
//! 对应 Java: org.ofdrw.converter.ItextMaker
//!
//! Java 版 `ItextMaker` 依赖 iText 7 库的 `PdfCanvas`、`PdfFont`
//! 等 API 进行 PDF 内容写入和字体嵌入。iText 是 Java/.NET 商业库，
//! 无 Rust 等价实现。
//!
//! Rust 版使用 `printpdf` crate 实现 PDF 生成，
//! 复用 [`crate::ofd_to_pdf`] 核心逻辑。

use std::path::{Path, PathBuf};

use easyofd_core::OfdResult;

use crate::ConvertOptions;

/// iText PDF 渲染器。
///
/// 对应 Java: `org.ofdrw.converter.ItextMaker`
///
/// 持有 OFD 源文件路径和转换选项，调用 [`crate::ofd_to_pdf`]
/// 将 OFD 渲染为 PDF（文本 + 图片 + 路径）。
/// 底层基于 `printpdf` 纯 Rust crate，替代 Java iText 商业库。
#[derive(Debug, Clone)]
pub struct ItextMaker {
    /// OFD 源文件路径。
    input: PathBuf,
    /// 转换选项。
    options: ConvertOptions,
}

impl ItextMaker {
    /// 创建新的 ItextMaker。
    ///
    /// # 参数
    ///
    /// - `input`: OFD 源文件路径
    /// - `options`: 转换选项（页面范围、页面尺寸覆盖等）
    pub fn new(input: impl Into<PathBuf>, options: ConvertOptions) -> Self {
        Self {
            input: input.into(),
            options,
        }
    }

    /// 返回替代实现的名称。
    ///
    /// 此类型已提供完整的 OFD→PDF 转换功能，底层基于 `printpdf` crate。
    #[must_use]
    pub fn replacement() -> &'static str {
        "easyofd_convert::converter::ItextMaker (基于 printpdf)"
    }

    /// 执行 OFD → PDF 转换。
    ///
    /// 委托给 [`crate::ofd_to_pdf`]，渲染文本（含 CJK 降级）、图片、路径。
    ///
    /// # 错误
    ///
    /// 输入文件不存在或格式错误时返回错误。
    pub fn convert(&self, target: &Path) -> OfdResult<()> {
        crate::ofd_to_pdf(&self.input, target, &self.options)
    }

    /// 获取源文件路径。
    pub fn input(&self) -> &Path {
        &self.input
    }

    /// 获取转换选项。
    pub fn options(&self) -> &ConvertOptions {
        &self.options
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use easyofd_core::{OfdPage, TextObject};
    use easyofd_writer::OfdWriter;

    #[test]
    fn test_itext_maker_replacement() {
        assert!(ItextMaker::replacement().contains("ItextMaker"));
    }

    #[test]
    fn test_itext_maker_convert() {
        let mut writer = OfdWriter::new();
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_text(TextObject::new(10.0, 20.0, "ItextMaker 测试"));
        writer.add_page(page);
        let ofd_bytes = writer.build().unwrap();

        let ofd_path = "/tmp/test_itext_maker.ofd";
        let pdf_path = "/tmp/test_itext_maker.pdf";
        std::fs::write(ofd_path, &ofd_bytes).unwrap();

        let maker = ItextMaker::new(ofd_path, ConvertOptions::default());
        let result = maker.convert(Path::new(pdf_path));
        assert!(
            result.is_ok(),
            "ItextMaker 转换应该成功: {:?}",
            result.err()
        );

        // 验证 PDF 文件存在且以 %PDF 开头
        let pdf_data = std::fs::read(pdf_path).unwrap();
        assert!(pdf_data.starts_with(b"%PDF"), "输出文件应为合法 PDF");

        let _ = std::fs::remove_file(ofd_path);
        let _ = std::fs::remove_file(pdf_path);
    }
}
