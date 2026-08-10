//! # easyofd-convert
#![allow(clippy::cast_possible_truncation, clippy::unnecessary_cast, clippy::cast_lossless)]
//!
//! PDF ↔ OFD 双向转换。
//!
//! ## 功能
//!
//! - PDF → OFD: 提取 PDF 文本和页面结构，映射到 OFD 文本对象
//! - OFD → PDF: 将 OFD 页面内容渲染为 PDF 页面
//!
//! ## 依赖
//!
//! - `lopdf`: PDF 解析
//! - `printpdf`: PDF 生成
//! - `easyofd-reader`: OFD 读取
//! - `easyofd-writer`: OFD 写入

use std::path::Path;

use easyofd_core::{OfdError, OfdPage, OfdResult, TextObject};
use easyofd_reader::OfdReader;
use easyofd_writer::OfdWriter;
use lopdf::Document;

/// 转换选项。
#[derive(Debug, Clone)]
pub struct ConvertOptions {
    /// 要转换的页面范围（0-based，空 = 所有页面）。
    pub pages: std::ops::Range<usize>,
    /// 输出页面尺寸覆盖（宽, 高）mm。None = 保留原始。
    pub page_size: Option<(f64, f64)>,
}

impl Default for ConvertOptions {
    fn default() -> Self {
        Self {
            pages: 0..0, // 空 = 所有
            page_size: None,
        }
    }
}

/// PDF → OFD 转换。
///
/// 从 PDF 提取文本内容和页面结构，映射到 OFD 文本对象。
///
/// # 错误
///
/// 如果输入文件无法读取或解析则返回错误。
pub fn pdf_to_ofd(
    pdf_path: impl AsRef<Path>,
    ofd_path: impl AsRef<Path>,
    options: &ConvertOptions,
) -> OfdResult<()> {
    let pdf_path = pdf_path.as_ref();
    let ofd_path = ofd_path.as_ref();

    // 解析 PDF
    let doc = Document::load(pdf_path).map_err(|e| OfdError::Conversion(format!("PDF 解析失败: {e}")))?;

    // 获取页面数量
    let page_count = doc.get_pages().len();
    let range = if options.pages.is_empty() {
        0..page_count
    } else {
        options.pages.start.min(page_count)..options.pages.end.min(page_count)
    };

    let mut writer = OfdWriter::new();
    let (default_w, default_h) = options.page_size.unwrap_or((210.0, 297.0));

    for page_num in range {
        let page_id = doc.get_pages().get(&(page_num as u32 + 1)).copied()
            .ok_or_else(|| OfdError::Conversion(format!("页面 {} 不存在", page_num + 1)))?;

        // 提取页面文本
        let text_content = extract_page_text(&doc, page_id)?;

        // 获取页面尺寸
        let (w, h) = get_page_size(&doc, page_id).unwrap_or((default_w, default_h));

        let mut page = OfdPage::new(w, h);

        // 将提取的文本添加到页面
        let mut y_offset = 20.0;
        for line in text_content.lines() {
            if !line.is_empty() {
                page.add_text(TextObject::new(10.0, y_offset, line));
                y_offset += 5.0; // 行间距
            }
        }

        writer.add_page(page);
    }

    // 写入 OFD 文件
    let ofd_bytes = writer.build()?;
    std::fs::write(ofd_path, ofd_bytes).map_err(OfdError::Io)?;

    Ok(())
}

/// 从 PDF 页面提取文本。
fn extract_page_text(doc: &Document, page_id: lopdf::ObjectId) -> OfdResult<String> {
    let mut text = String::new();

    // 获取页面内容流
    if let Ok(content_stream) = doc.get_page_content(page_id) {
        // 简单的文本提取：查找 Tj 和 TJ 操作符
        let content = String::from_utf8_lossy(&content_stream);

        // 查找文本字符串 (Tj 操作)
        for line in content.lines() {
            if line.ends_with(" Tj") {
                // 提取引号内的文本
                if let Some(start) = line.find('(') {
                    if let Some(end) = line.rfind(')') {
                        let text_content = &line[start + 1..end];
                        text.push_str(text_content);
                        text.push('\n');
                    }
                }
            }
        }
    }

    Ok(text)
}

/// 获取 PDF 页面尺寸（mm）。
fn get_page_size(doc: &Document, page_id: lopdf::ObjectId) -> Option<(f64, f64)> {
    if let Ok(page) = doc.get_dictionary(page_id) {
        if let Ok(media_box) = page.get(b"MediaBox") {
            if let Ok(array) = media_box.as_array() {
                if array.len() >= 4 {
                    let x1 = array[0].as_float().unwrap_or(0.0) as f64;
                    let y1 = array[1].as_float().unwrap_or(0.0) as f64;
                    let x2 = array[2].as_float().unwrap_or(612.0) as f64;
                    let y2 = array[3].as_float().unwrap_or(792.0) as f64;

                    // PDF 单位是点 (1 点 = 1/72 英寸)，转换为 mm
                    let width_pt = x2 - x1;
                    let height_pt = y2 - y1;
                    let width_mm = width_pt * 25.4 / 72.0;
                    let height_mm = height_pt * 25.4 / 72.0;

                    return Some((width_mm, height_mm));
                }
            }
        }
    }
    None
}

/// OFD → PDF 转换。
///
/// 将 OFD 页面内容渲染为 PDF 页面。
///
/// # 错误
///
/// 如果输入文件无法读取或解析则返回错误。
pub fn ofd_to_pdf(
    ofd_path: impl AsRef<Path>,
    pdf_path: impl AsRef<Path>,
    options: &ConvertOptions,
) -> OfdResult<()> {
    let ofd_path = ofd_path.as_ref();
    let pdf_path = pdf_path.as_ref();

    // 读取 OFD 文件
    let ofd_bytes = std::fs::read(ofd_path).map_err(OfdError::Io)?;
    let reader = OfdReader::from_bytes(&ofd_bytes)?;

    let pages = reader.pages();
    let range = if options.pages.is_empty() {
        0..pages.len()
    } else {
        options.pages.start.min(pages.len())..options.pages.end.min(pages.len())
    };

    // 创建 PDF 文档
    let (doc, page_id, layer_id) = printpdf::PdfDocument::new(
        "OFD Export",
        printpdf::Mm(210.0 as f32),
        printpdf::Mm(297.0 as f32),
        "Layer 1",
    );

    let mut current_layer = doc.get_page(page_id).get_layer(layer_id);

    // 添加字体
    let font = doc.add_builtin_font(printpdf::BuiltinFont::Helvetica)
        .map_err(|e| OfdError::Conversion(format!("字体加载失败: {e}")))?;

    for page_idx in range.clone() {
        let page = &pages[page_idx];

        // 设置页面尺寸
        let width_mm = page.width;
        let height_mm = page.height;

        // 添加文本对象
        for content in &page.content {
            if let easyofd_core::ContentObject::Text(text) = content {
                current_layer.use_text(
                    &text.text,
                    text.size as f32,
                    printpdf::Mm(text.x as f32),
                    printpdf::Mm((height_mm - text.y) as f32), // PDF 坐标系 Y 轴向上
                    &font,
                );
            }
        }

        // 如果不是最后一页，添加新页面
        if page_idx < range.end - 1 {
            let (new_page_id, new_layer_id) = doc.add_page(
                printpdf::Mm(width_mm as f32),
                printpdf::Mm(height_mm as f32),
                "Layer 1",
            );
            current_layer = doc.get_page(new_page_id).get_layer(new_layer_id);
        }
    }

    // 保存 PDF
    doc.save(&mut std::io::BufWriter::new(
        std::fs::File::create(pdf_path).map_err(OfdError::Io)?,
    ))
    .map_err(|e| OfdError::Conversion(format!("PDF 保存失败: {e}")))?;

    Ok(())
}

/// 图片格式转换辅助。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageConvertFormat {
    /// JPEG 格式。
    Jpeg,
    /// PNG 格式。
    Png,
    /// BMP 格式。
    Bmp,
}

/// 在格式之间转换图片（用于 OFD Resource 嵌入）。
///
/// # 错误
///
/// 如果转换失败则返回错误。
pub fn convert_image(_input: &[u8], _target_format: ImageConvertFormat) -> OfdResult<Vec<u8>> {
    Err(OfdError::Conversion(
        "图片转换需要 image crate 集成（计划中）".into(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_options_default() {
        let opts = ConvertOptions::default();
        assert!(opts.pages.is_empty()); // 0..0 = 所有页面
        assert!(opts.page_size.is_none());
    }

    #[test]
    fn test_convert_options_custom() {
        let opts = ConvertOptions {
            pages: 0..5,
            page_size: Some((210.0, 297.0)),
        };
        assert_eq!(opts.pages, (0..5));
        assert_eq!(opts.page_size, Some((210.0, 297.0)));
    }

    #[test]
    fn test_pdf_to_ofd_returns_error_for_missing_file() {
        let result = pdf_to_ofd("nonexistent.pdf", "out.ofd", &ConvertOptions::default());
        assert!(result.is_err());
    }

    #[test]
    fn test_ofd_to_pdf_returns_error_for_missing_file() {
        let result = ofd_to_pdf("nonexistent.ofd", "out.pdf", &ConvertOptions::default());
        assert!(result.is_err());
    }

    #[test]
    fn test_convert_image_returns_error() {
        let result = convert_image(&[0xFF, 0xD8], ImageConvertFormat::Png);
        assert!(result.is_err());
    }

    #[test]
    fn test_image_convert_format_enum() {
        assert_ne!(ImageConvertFormat::Jpeg, ImageConvertFormat::Png);
        assert_ne!(ImageConvertFormat::Bmp, ImageConvertFormat::Jpeg);
    }

    #[test]
    fn test_ofd_to_pdf_roundtrip() {
        use easyofd_core::OfdPage;
        use easyofd_writer::OfdWriter;

        // 创建测试 OFD
        let mut writer = OfdWriter::new();
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_text(TextObject::new(10.0, 20.0, "PDF 转换测试"));
        writer.add_page(page);
        let ofd_bytes = writer.build().unwrap();

        // 写入临时文件
        let ofd_path = "/tmp/test_convert.ofd";
        let pdf_path = "/tmp/test_convert.pdf";
        std::fs::write(ofd_path, &ofd_bytes).unwrap();

        // OFD → PDF 转换
        let result = ofd_to_pdf(ofd_path, pdf_path, &ConvertOptions::default());
        assert!(result.is_ok(), "OFD → PDF 转换应该成功: {:?}", result.err());

        // 验证 PDF 文件存在
        assert!(std::path::Path::new(pdf_path).exists(), "PDF 文件应该存在");

        // 清理
        let _ = std::fs::remove_file(ofd_path);
        let _ = std::fs::remove_file(pdf_path);
    }
}
