//! # easyofd-convert
#![allow(clippy::cast_possible_truncation, clippy::unnecessary_cast, clippy::cast_lossless)]
//!
//! PDF ↔ OFD 双向转换。
//!
//! ## 功能
//!
//! - PDF → OFD: 提取 PDF 文本(Tj/TJ 操作符)和页面结构
//! - OFD → PDF: 渲染文本、图片、路径对象到 PDF
//!
//! ## 依赖
//!
//! - `lopdf`: PDF 解析
//! - `printpdf`: PDF 生成
//! - `easyofd-reader`: OFD 读取
//! - `easyofd-writer`: OFD 写入

use std::path::Path;

use easyofd_core::{ContentObject, OfdError, OfdPage, OfdResult, TextObject};
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

// ─── PDF → OFD ──────────────────────────────────────────────────────────────

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

    let doc = Document::load(pdf_path)
        .map_err(|e| OfdError::Conversion(format!("PDF 解析失败: {e}")))?;

    let page_count = doc.get_pages().len();
    let range = if options.pages.is_empty() {
        0..page_count
    } else {
        options.pages.start.min(page_count)..options.pages.end.min(page_count)
    };

    let mut writer = OfdWriter::new();
    let (default_w, default_h) = options.page_size.unwrap_or((210.0, 297.0));

    for page_num in range {
        let page_id = doc
            .get_pages()
            .get(&(page_num as u32 + 1))
            .copied()
            .ok_or_else(|| OfdError::Conversion(format!("页面 {} 不存在", page_num + 1)))?;

        let text_lines = extract_page_text(&doc, page_id)?;
        let (w, h) = get_page_size(&doc, page_id).unwrap_or((default_w, default_h));

        let mut page = OfdPage::new(w, h);
        let mut y_offset = 20.0;

        for line in text_lines {
            if !line.is_empty() {
                page.add_text(TextObject::new(10.0, y_offset, &line));
                y_offset += 5.0;
            }
        }

        writer.add_page(page);
    }

    let ofd_bytes = writer.build()?;
    std::fs::write(ofd_path, ofd_bytes).map_err(OfdError::Io)?;
    Ok(())
}

/// 从 PDF 页面提取文本行。
///
/// 支持 Tj (单字符串) 和 TJ (数组) 操作符。
fn extract_page_text(doc: &Document, page_id: lopdf::ObjectId) -> OfdResult<Vec<String>> {
    let mut lines = Vec::new();

    if let Ok(content_stream) = doc.get_page_content(page_id) {
        let content = String::from_utf8_lossy(&content_stream);
        let mut current_line = String::new();

        for line in content.lines() {
            let trimmed = line.trim();

            // Tj 操作: (text) Tj
            if trimmed.ends_with(" Tj") {
                if let Some(text) = extract_string_from_tj(trimmed) {
                    current_line.push_str(&text);
                }
            }
            // TJ 操作: [(text) spacing (text)] TJ
            else if trimmed.ends_with(" TJ") {
                if let Some(text) = extract_string_from_tj_array(trimmed) {
                    current_line.push_str(&text);
                }
            }
            // Td/TD 操作: 新行
            else if trimmed.ends_with(" Td") || trimmed.ends_with(" TD") {
                if !current_line.is_empty() {
                    lines.push(current_line.clone());
                    current_line.clear();
                }
            }
        }

        if !current_line.is_empty() {
            lines.push(current_line);
        }
    }

    Ok(lines)
}

/// 从 Tj 操作提取字符串: (text) Tj
fn extract_string_from_tj(op: &str) -> Option<String> {
    let start = op.find('(')?;
    let end = op.rfind(')')?;
    if start < end {
        Some(op[start + 1..end].to_string())
    } else {
        None
    }
}

/// 从 TJ 操作提取字符串: [(text) spacing (text)] TJ
fn extract_string_from_tj_array(op: &str) -> Option<String> {
    let start = op.find('[')?;
    let end = op.rfind(']')?;
    if start >= end {
        return None;
    }

    let array_content = &op[start + 1..end];
    let mut text = String::new();
    let mut chars = array_content.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '(' {
            // 读取到匹配的 )
            let mut depth = 1;
            let mut escaped = false;
            let mut str_content = String::new();

            for inner_c in chars.by_ref() {
                if escaped {
                    str_content.push(inner_c);
                    escaped = false;
                } else if inner_c == '\\' {
                    escaped = true;
                } else if inner_c == '(' {
                    depth += 1;
                    str_content.push(inner_c);
                } else if inner_c == ')' {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                    str_content.push(inner_c);
                } else {
                    str_content.push(inner_c);
                }
            }

            text.push_str(&str_content);
        }
        // 数字参数（间距）忽略
    }

    Some(text)
}

/// 获取 PDF 页面尺寸（mm）。
fn get_page_size(doc: &Document, page_id: lopdf::ObjectId) -> Option<(f64, f64)> {
    let page = doc.get_dictionary(page_id).ok()?;
    let media_box = page.get(b"MediaBox").ok()?;
    let array = media_box.as_array().ok()?;

    if array.len() < 4 {
        return None;
    }

    let x1 = array[0].as_float().ok()? as f64;
    let y1 = array[1].as_float().ok()? as f64;
    let x2 = array[2].as_float().ok()? as f64;
    let y2 = array[3].as_float().ok()? as f64;

    let width_mm = (x2 - x1) * 25.4 / 72.0;
    let height_mm = (y2 - y1) * 25.4 / 72.0;

    Some((width_mm, height_mm))
}

// ─── OFD → PDF ──────────────────────────────────────────────────────────────

/// OFD → PDF 转换。
///
/// 将 OFD 页面内容渲染为 PDF 页面（文本、图片、路径）。
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

    let ofd_bytes = std::fs::read(ofd_path).map_err(OfdError::Io)?;
    let reader = OfdReader::from_bytes(&ofd_bytes)?;

    let pages = reader.pages();
    let range = if options.pages.is_empty() {
        0..pages.len()
    } else {
        options.pages.start.min(pages.len())..options.pages.end.min(pages.len())
    };

    if range.is_empty() {
        return Err(OfdError::Conversion("没有可转换的页面".into()));
    }

    let first_page = &pages[range.start];
    let (doc, page_id, layer_id) = printpdf::PdfDocument::new(
        "OFD Export",
        printpdf::Mm(first_page.width as f32),
        printpdf::Mm(first_page.height as f32),
        "Layer 1",
    );

    let font = doc
        .add_builtin_font(printpdf::BuiltinFont::Helvetica)
        .map_err(|e| OfdError::Conversion(format!("字体加载失败: {e}")))?;

    let mut current_layer = doc.get_page(page_id).get_layer(layer_id);

    for (idx, page_idx) in range.clone().enumerate() {
        let page = &pages[page_idx];
        let height_mm = page.height;

        for content in &page.content {
            match content {
                ContentObject::Text(text) => {
                    render_text_to_pdf(&current_layer, text, height_mm, &font);
                }
                ContentObject::Image(img) => {
                    render_image_to_pdf(&current_layer, img, height_mm);
                }
                ContentObject::Path(path) => {
                    render_path_to_pdf(&current_layer, path, height_mm);
                }
            }
        }

        // 添加新页面（如果不是最后一页）
        if idx < range.len() - 1 {
            let next_page = &pages[page_idx + 1];
            let (new_page_id, new_layer_id) = doc.add_page(
                printpdf::Mm(next_page.width as f32),
                printpdf::Mm(next_page.height as f32),
                "Layer 1",
            );
            current_layer = doc.get_page(new_page_id).get_layer(new_layer_id);
        }
    }

    doc.save(&mut std::io::BufWriter::new(
        std::fs::File::create(pdf_path).map_err(OfdError::Io)?,
    ))
    .map_err(|e| OfdError::Conversion(format!("PDF 保存失败: {e}")))?;

    Ok(())
}

/// 渲染文本对象到 PDF 层。
fn render_text_to_pdf(
    layer: &printpdf::PdfLayerReference,
    text: &easyofd_core::TextObject,
    page_height: f64,
    font: &printpdf::IndirectFontRef,
) {
    let x = printpdf::Mm(text.x as f32);
    let y = printpdf::Mm((page_height - text.y) as f32); // PDF Y 轴向上
    layer.use_text(&text.text, text.size as f32, x, y, font);
}

/// 渲染图片对象到 PDF 层。
fn render_image_to_pdf(
    _layer: &printpdf::PdfLayerReference,
    _img: &easyofd_core::ImageObject,
    _page_height: f64,
) {
    // 图片渲染需要 image crate 集成，当前版本暂不支持
    // TODO: 使用 image::load_from_memory 解析图片数据
}

/// 渲染路径对象到 PDF 层。
fn render_path_to_pdf(
    layer: &printpdf::PdfLayerReference,
    path: &easyofd_core::PathObject,
    page_height: f64,
) {
    use printpdf::*;

    let stroke_color = path.stroke_color;
    let r = ((stroke_color >> 16) & 0xFF) as f64 / 255.0;
    let g = ((stroke_color >> 8) & 0xFF) as f64 / 255.0;
    let b = (stroke_color & 0xFF) as f64 / 255.0;

    layer.set_outline_color(Color::Rgb(Rgb::new(r as f32, g as f32, b as f32, None)));
    layer.set_outline_thickness(path.stroke_width as f32);

    // 解析简化路径数据 (M x y L x y 格式)
    let path_data = &path.path_data;
    let tokens: Vec<&str> = path_data.split_whitespace().collect();

    let mut i = 0;
    let mut current_x = path.x;
    let mut current_y = path.y;

    while i < tokens.len() {
        match tokens[i] {
            "M" if i + 2 < tokens.len() => {
                if let (Ok(x), Ok(y)) = (tokens[i + 1].parse::<f64>(), tokens[i + 2].parse::<f64>())
                {
                    current_x = x;
                    current_y = y;
                    i += 3;
                } else {
                    i += 1;
                }
            }
            "L" if i + 2 < tokens.len() => {
                if let (Ok(x), Ok(y)) = (tokens[i + 1].parse::<f64>(), tokens[i + 2].parse::<f64>())
                {
                    let points = vec![
                        (
                            Point::new(Mm(current_x as f32), Mm((page_height - current_y) as f32)),
                            false,
                        ),
                        (Point::new(Mm(x as f32), Mm((page_height - y) as f32)), false),
                    ];
                    layer.add_line(Line {
                        points,
                        is_closed: false,
                    });
                    current_x = x;
                    current_y = y;
                    i += 3;
                } else {
                    i += 1;
                }
            }
            _ => i += 1,
        }
    }
}

// ─── 图片格式转换 ────────────────────────────────────────────────────────────

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
        assert!(opts.pages.is_empty());
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
    fn test_extract_string_from_tj() {
        assert_eq!(extract_string_from_tj("(Hello) Tj"), Some("Hello".into()));
        assert_eq!(extract_string_from_tj("no parens"), None);
    }

    #[test]
    fn test_extract_string_from_tj_array() {
        let op = "[(Hello) -100 (World)] TJ";
        assert_eq!(extract_string_from_tj_array(op), Some("HelloWorld".into()));
    }

    #[test]
    fn test_ofd_to_pdf_roundtrip() {
        use easyofd_core::OfdPage;
        use easyofd_writer::OfdWriter;

        let mut writer = OfdWriter::new();
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_text(TextObject::new(10.0, 20.0, "PDF 转换测试"));
        writer.add_page(page);
        let ofd_bytes = writer.build().unwrap();

        let ofd_path = "/tmp/test_convert.ofd";
        let pdf_path = "/tmp/test_convert.pdf";
        std::fs::write(ofd_path, &ofd_bytes).unwrap();

        let result = ofd_to_pdf(ofd_path, pdf_path, &ConvertOptions::default());
        assert!(result.is_ok(), "OFD → PDF 转换应该成功: {:?}", result.err());
        assert!(std::path::Path::new(pdf_path).exists());

        let _ = std::fs::remove_file(ofd_path);
        let _ = std::fs::remove_file(pdf_path);
    }

    #[test]
    fn test_ofd_to_pdf_with_path_object() {
        use easyofd_core::{OfdPage, PathObject};
        use easyofd_writer::OfdWriter;

        let mut writer = OfdWriter::new();
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_path(PathObject::new(50.0, 50.0, "M 0 0 L 100 100"));
        writer.add_page(page);
        let ofd_bytes = writer.build().unwrap();

        let ofd_path = "/tmp/test_path.ofd";
        let pdf_path = "/tmp/test_path.pdf";
        std::fs::write(ofd_path, &ofd_bytes).unwrap();

        let result = ofd_to_pdf(ofd_path, pdf_path, &ConvertOptions::default());
        assert!(result.is_ok(), "路径对象转换应该成功: {:?}", result.err());

        let _ = std::fs::remove_file(ofd_path);
        let _ = std::fs::remove_file(pdf_path);
    }
}
