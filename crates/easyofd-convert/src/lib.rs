//! # easyofd-convert
#![allow(
    clippy::cast_possible_truncation,
    clippy::unnecessary_cast,
    clippy::cast_lossless,
    clippy::similar_names,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clippy::many_single_char_names
)]
//!
//! PDF ↔ OFD 双向转换。
//!
//! ## 功能
//!
//! - PDF → OFD: 提取 PDF 文本(Tj/TJ 操作符)、图片(XObject/Image)和页面结构
//! - OFD → PDF: 渲染文本(含 CJK 降级检测)、图片、路径(含贝塞尔曲线近似)对象到 PDF
//!
//! ## 依赖
//!
//! - `lopdf`: PDF 解析
//! - `printpdf`: PDF 生成
//! - `easyofd-reader`: OFD 读取
//! - `easyofd-writer`: OFD 写入

mod cjk_font;
mod convert_helper;
mod convert_options;
pub mod converter;
pub mod error;
pub mod exporter;
pub mod font;
pub mod html;
pub mod image;
mod image_convert_format;
pub mod importer;
pub mod point;
pub mod utils;

pub use convert_helper::ConvertHelper;
pub use convert_options::ConvertOptions;
pub use converter::{
    AWTMaker, CgTransformEntry, CgTransformMap, Config, DocConverter, HtmlMaker, ItextMaker, Lib,
    PdfboxMaker, SVGMaker,
};
pub use error::GeneralConvertError;
pub use exporter::{
    Exporter, HTMLExporter, ImageExporter, OFDExporter, PDFExporterIText, PDFExporterPDFBox,
    PdfExporter, SvgExporter, TextExporter,
};
pub use font::{
    BoundingBox, CmapSubtable, FontDrawPathProvider, FontLoader, FontWrapper, GlyfCompositeComp,
    GlyfCompositeDescript, GlyfDescript, GlyfSimpleDescript, GlyphData, GlyphDataProvider,
    GlyphPath, GlyphPoint, HorizontalHeaderTable, HorizontalMetricsTable, MemoryTTFDataStream,
    NameRecord, NamingTable, PdfFontWrapper, Type1Seg, Type1SegSplitParser, Type1SegType,
};
pub use html::Element;
pub use image::ImageMedia;
pub use image_convert_format::ImageConvertFormat;
pub use importer::{Importer, PdfImporter};
pub use point::{PathPoint, TextCodePoint, Tuple2};
pub use utils::{EPlatform, Matrix3x3};

// ── ofdrw Java 类名别名 ──

/// 对应 Java: `org.ofdrw.converter.ConvertHelper`（Lib 枚举）
///
/// Java 原始类名为 `ConvertHelper`，Rust 版的 Lib 枚举在 [`converter::lib_enum`] 中。
pub use converter::Lib as ConvertHelperLib;

/// 对应 Java: `org.ofdrw.converter.font.FontDrawPathProvider`
///
/// trait 别名，保持 Java 接口名兼容。
/// 详细实现见 [`font::FontDrawPathProvider`]。
use std::path::Path;

use easyofd_core::{
    ContentObject, ImageFormat, ImageObject, OfdError, OfdPage, OfdResult, TextObject,
};
use easyofd_reader::OfdReader;
use easyofd_writer::OfdWriter;
use lopdf::Document;

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

    let doc =
        Document::load(pdf_path).map_err(|e| OfdError::Conversion(format!("PDF 解析失败: {e}")))?;

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
        let images = extract_page_images(&doc, page_id);
        let (w, h) = get_page_size(&doc, page_id).unwrap_or((default_w, default_h));

        let mut page = OfdPage::new(w, h);
        let mut y_offset = 20.0;

        for line in text_lines {
            if !line.is_empty() {
                page.add_text(TextObject::new(10.0, y_offset, &line));
                y_offset += 5.0;
            }
        }

        // 将提取的图片放入页面（按序号垂直排列）
        for (idx, img) in images.into_iter().enumerate() {
            let img_x = 10.0;
            let img_y = y_offset + (idx as f64) * (img.height + 2.0);
            page.add_image(ImageObject::new(
                img_x, img_y, img.width, img.height, img.data, img.format,
            ));
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
            else if (trimmed.ends_with(" Td") || trimmed.ends_with(" TD"))
                && !current_line.is_empty()
            {
                lines.push(current_line.clone());
                current_line.clear();
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

/// 从 PDF 页面提取图片列表。
///
/// 遍历页面 Resources/XObject 中的 Image 对象，
/// 支持 DCTDecode（JPEG 直传）和 FlateDecode（原始像素→BMP 编码）。
fn extract_page_images(doc: &Document, page_id: lopdf::ObjectId) -> Vec<ExtractedPdfImage> {
    let mut result = Vec::new();

    let images = match doc.get_page_images(page_id) {
        Ok(imgs) => imgs,
        Err(_) => return result,
    };

    for pdf_img in images {
        let width_mm = pdf_img.width as f64 * 25.4 / 72.0;
        let height_mm = pdf_img.height as f64 * 25.4 / 72.0;

        // 判断图片格式
        let is_dct = pdf_img
            .filters
            .as_ref()
            .is_some_and(|filters| filters.iter().any(|f| f == "DCTDecode"));

        if is_dct {
            // JPEG 直传
            result.push(ExtractedPdfImage {
                width: width_mm,
                height: height_mm,
                data: pdf_img.content.to_vec(),
                format: ImageFormat::Jpeg,
            });
        } else {
            // FlateDecode 或无压缩：原始像素数据，编码为 BMP
            let color_space = pdf_img.color_space.as_deref().unwrap_or("DeviceRGB");
            let bpc = pdf_img.bits_per_component.unwrap_or(8) as u32;
            if let Some(bmp_data) = encode_raw_pixels_to_bmp(
                pdf_img.content,
                pdf_img.width,
                pdf_img.height,
                color_space,
                bpc,
            ) {
                result.push(ExtractedPdfImage {
                    width: width_mm,
                    height: height_mm,
                    data: bmp_data,
                    format: ImageFormat::Bmp,
                });
            }
        }
    }

    result
}

/// 从 PDF 提取的图片中间结构。
struct ExtractedPdfImage {
    /// 图片宽度（mm）。
    width: f64,
    /// 图片高度（mm）。
    height: f64,
    /// 图片数据。
    data: Vec<u8>,
    /// 图片格式。
    format: ImageFormat,
}

/// 将原始像素数据编码为 BMP 格式。
///
/// 支持 DeviceRGB（24 位）和 DeviceGray（8 位）色彩空间。
/// 如果色彩空间不支持则返回 None。
fn encode_raw_pixels_to_bmp(
    raw: &[u8],
    width: i64,
    height: i64,
    color_space: &str,
    bits_per_component: u32,
) -> Option<Vec<u8>> {
    let w = width as u32;
    let h = height as u32;

    // 根据色彩空间计算每行字节数
    let (channels, bpp) = match (color_space, bits_per_component) {
        ("DeviceRGB", 8) => (3u32, 24u32),
        ("DeviceGray", 8) => (1u32, 8u32),
        _ => return None, // 不支持的色彩空间
    };

    let row_stride = w * channels;
    // BMP 每行需要 4 字节对齐
    let row_padded = (row_stride + 3) & !3;
    let image_size = row_padded * h;
    let file_size = 54 + if bpp == 8 { 256 * 4 } else { 0 } + image_size as u32;

    let mut bmp = Vec::with_capacity(file_size as usize);

    // ── BMP 文件头（14 字节）──
    bmp.extend_from_slice(b"BM");
    bmp.extend_from_slice(&file_size.to_le_bytes());
    bmp.extend_from_slice(&[0u8; 4]); // 保留
    let pixel_offset: u32 = 54 + if bpp == 8 { 256 * 4 } else { 0 };
    bmp.extend_from_slice(&pixel_offset.to_le_bytes());

    // ── DIB 信息头（BITMAPINFOHEADER，40 字节）──
    bmp.extend_from_slice(&40u32.to_le_bytes());
    bmp.extend_from_slice(&(w as i32).to_le_bytes());
    bmp.extend_from_slice(&(h as i32).to_le_bytes());
    bmp.extend_from_slice(&1u16.to_le_bytes()); // planes
    bmp.extend_from_slice(&(bpp as u16).to_le_bytes());
    bmp.extend_from_slice(&0u32.to_le_bytes()); // compression = BI_RGB
    bmp.extend_from_slice(&image_size.to_le_bytes());
    bmp.extend_from_slice(&2835u32.to_le_bytes()); // X pixels per meter
    bmp.extend_from_slice(&2835u32.to_le_bytes()); // Y pixels per meter
    bmp.extend_from_slice(&0u32.to_le_bytes()); // colors used
    bmp.extend_from_slice(&0u32.to_le_bytes()); // important colors

    // ── 调色板（仅 8 位灰度需要）──
    if bpp == 8 {
        for i in 0..=255u8 {
            bmp.extend_from_slice(&[i, i, i, 0]); // 灰度调色板
        }
    }

    // ── 像素数据（BMP 从底行开始，BGR 顺序）──
    for row in (0..h).rev() {
        let row_start = (row * row_stride) as usize;
        let row_end = row_start + row_stride as usize;
        if row_end > raw.len() {
            break;
        }
        let row_data = &raw[row_start..row_end];

        if channels == 3 {
            // RGB → BGR
            for px in row_data.chunks(3) {
                if px.len() >= 3 {
                    bmp.extend_from_slice(&[px[2], px[1], px[0]]);
                }
            }
        } else {
            bmp.extend_from_slice(row_data);
        }

        // 填充到 4 字节对齐
        let padding = (row_padded - row_stride) as usize;
        bmp.extend_from_slice(&vec![0u8; padding]);
    }

    Some(bmp)
}

/// 检测文本是否包含 CJK（中日韩）字符。
///
/// 通过检查 Unicode 码点范围判断：CJK 统一表意文字 (U+4E00–U+9FFF)、
/// CJK 扩展 A (U+3400–U+4DBF)、兼容表意文字 (U+F900–U+FAFF)。
fn contains_cjk(text: &str) -> bool {
    text.chars().any(|c| {
        let cp = c as u32;
        (0x4E00..=0x9FFF).contains(&cp)
            || (0x3400..=0x4DBF).contains(&cp)
            || (0xF900..=0xFAFF).contains(&cp)
            || (0x2E80..=0x2EFF).contains(&cp)  // CJK 部首扩展
            || (0x3000..=0x303F).contains(&cp)  // CJK 符号和标点
            || (0xFF00..=0xFFEF).contains(&cp) // 全角 ASCII
    })
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

    // 收集文档中所有文本用到的字符（用于子集化统计）
    let used_chars = cjk_font::collect_used_chars(&pages[range.clone()]);

    // 探测系统 CJK 字体，用于渲染中日韩文本
    let cjk_font = if let Some(info) = cjk_font::find_cjk_font() {
        eprintln!(
            "[easyofd-convert] 发现 CJK 字体: {} ({})",
            info.name,
            info.path.display()
        );
        // 如果字体文件可读，尝试子集化分析
        if let Ok(face_data) = std::fs::read(&info.path) {
            match cjk_font::subset_font(&face_data, &used_chars) {
                Ok(stats) => {
                    eprintln!(
                        "[easyofd-convert] 字体子集化分析: 原始 {} KB, 文档用 {} 字符 \
                         (CJK {}), 可映射 glyph {}",
                        stats.original_size / 1024,
                        stats.used_char_count,
                        stats.cjk_char_count,
                        stats.mapped_glyph_count,
                    );
                }
                Err(e) => {
                    eprintln!("[easyofd-convert] 字体子集化分析失败（不影响转换）: {e}");
                }
            }
        }
        match std::fs::File::open(&info.path) {
            Ok(file) => match doc.add_external_font_with_subsetting(file, true) {
                Ok(font_ref) => Some(font_ref),
                Err(e) => {
                    eprintln!("[easyofd-convert] CJK 字体加载失败，回退到 Helvetica: {e}");
                    None
                }
            },
            Err(e) => {
                eprintln!("[easyofd-convert] CJK 字体文件读取失败，回退到 Helvetica: {e}");
                None
            }
        }
    } else {
        eprintln!("[easyofd-convert] 未找到系统 CJK 字体，使用 Helvetica（CJK 字形可能丢失）");
        None
    };

    let mut current_layer = doc.get_page(page_id).get_layer(layer_id);

    for (idx, page_idx) in range.clone().enumerate() {
        let page = &pages[page_idx];
        let height_mm = page.height;

        for content in &page.content {
            match content {
                ContentObject::Text(text) => {
                    render_text_to_pdf(&current_layer, text, height_mm, &font, cjk_font.as_ref());
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
///
/// 如果文本包含 CJK 字符且提供了 CJK 字体，则使用嵌入的 CJK 字体渲染；
/// 否则回退到 Helvetica（内置字体，不支持 CJK 字形）。
fn render_text_to_pdf(
    layer: &printpdf::PdfLayerReference,
    text: &easyofd_core::TextObject,
    page_height: f64,
    font: &printpdf::IndirectFontRef,
    cjk_font: Option<&printpdf::IndirectFontRef>,
) {
    let effective_font = if contains_cjk(&text.text) {
        if let Some(cjk) = cjk_font {
            cjk
        } else {
            eprintln!(
                "[easyofd-convert] 警告：文本 \"{}\" 包含 CJK 字符，\
                 使用 Helvetica 替代渲染（字形可能丢失）",
                if text.text.len() > 20 {
                    &text.text[..20]
                } else {
                    &text.text
                }
            );
            font
        }
    } else {
        font
    };
    let x = printpdf::Mm(text.x as f32);
    let y = printpdf::Mm((page_height - text.y) as f32); // PDF Y 轴向上
    layer.use_text(&text.text, text.size as f32, x, y, effective_font);
}

/// 渲染图片对象到 PDF 层。
///
/// 使用 `image` crate 解码图片（支持 PNG/JPEG/BMP/TIFF），提取原始 RGB8 像素，
/// 构造 `printpdf::ImageXObject` 并通过 `Image::add_to_layer` 嵌入 PDF。
///
/// 坐标换算：OFD 坐标系左上原点，PDF 坐标系左下原点，
/// `translate_y = page_height - img.y - img.height`。
/// DPI 与 scale_x/scale_y 联合计算，确保输出尺寸（mm）与 ImageObject 宽高一致。
fn render_image_to_pdf(
    layer: &printpdf::PdfLayerReference,
    img: &easyofd_core::ImageObject,
    page_height: f64,
) {
    use printpdf::{ColorBits, ColorSpace, Image, ImageTransform, ImageXObject, Mm, Px};

    // 用 image crate 解码图片数据（自动识别 PNG/JPEG/BMP/TIFF 格式）
    let dyn_img = match image::load_from_memory(&img.data) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("[easyofd-convert] 图片解码失败，跳过: {e}");
            return;
        }
    };
    let rgb_img = dyn_img.to_rgb8();
    let (w_px, h_px) = rgb_img.dimensions();
    if w_px == 0 || h_px == 0 {
        return;
    }

    let xobj = ImageXObject {
        width: Px(w_px as usize),
        height: Px(h_px as usize),
        color_space: ColorSpace::Rgb,
        bits_per_component: ColorBits::Bit8,
        interpolate: true,
        image_data: rgb_img.into_raw(),
        image_filter: None,
        smask: None,
        clipping_bbox: None,
    };
    let image = Image::from(xobj);

    // 默认 DPI = 300（printpdf 默认），计算 scale 使输出尺寸 = img.width/height mm
    let dpi: f32 = 300.0;
    let scale_x = (img.width as f32) * dpi / (w_px as f32 * 25.4);
    let scale_y = (img.height as f32) * dpi / (h_px as f32 * 25.4);

    // OFD 左上原点 → PDF 左下原点
    let translate_x = Mm(img.x as f32);
    let translate_y = Mm((page_height - img.y - img.height) as f32);

    image.add_to_layer(
        layer.clone(),
        ImageTransform {
            translate_x: Some(translate_x),
            translate_y: Some(translate_y),
            scale_x: Some(scale_x),
            scale_y: Some(scale_y),
            dpi: Some(dpi),
            ..Default::default()
        },
    );
}

/// 渲染路径对象到 PDF 层。
///
/// 支持的路径命令：
/// - `M x y`：移动到指定坐标
/// - `L x y`：直线到指定坐标
/// - `Z`：闭合路径（对应 PDF `h` 操作符）
/// - `C x1 y1 x2 y2 x y`：三次贝塞尔曲线（用直线段近似）
/// - `Q x1 y1 x y`：二次贝塞尔曲线（用直线段近似）
///
/// 支持 FillColor → PDF 填充操作。
fn render_path_to_pdf(
    layer: &printpdf::PdfLayerReference,
    path: &easyofd_core::PathObject,
    page_height: f64,
) {
    use printpdf::{Color, Line, Mm, Point, Rgb};

    let stroke_color = path.stroke_color;
    let r = ((stroke_color >> 16) & 0xFF) as f64 / 255.0;
    let g = ((stroke_color >> 8) & 0xFF) as f64 / 255.0;
    let b = (stroke_color & 0xFF) as f64 / 255.0;

    layer.set_outline_color(Color::Rgb(Rgb::new(r as f32, g as f32, b as f32, None)));
    layer.set_outline_thickness(path.stroke_width as f32);

    // 如果有填充色，设置填充颜色
    if let Some(fill_rgb) = path.fill_color {
        let fr = ((fill_rgb >> 16) & 0xFF) as f64 / 255.0;
        let fg = ((fill_rgb >> 8) & 0xFF) as f64 / 255.0;
        let fb = (fill_rgb & 0xFF) as f64 / 255.0;
        layer.set_fill_color(Color::Rgb(Rgb::new(fr as f32, fg as f32, fb as f32, None)));
    }

    // 解析路径数据
    let path_data = &path.path_data;
    let tokens: Vec<&str> = path_data.split_whitespace().collect();

    let mut i = 0;
    let mut current_x = path.x;
    let mut current_y = path.y;
    let mut subpath_start_x = current_x;
    let mut subpath_start_y = current_y;

    // 收集所有线段点（用于最终绘制）
    let mut all_points: Vec<(Point, bool)> = Vec::new();
    let mut has_path = false;

    while i < tokens.len() {
        match tokens[i] {
            "M" if i + 2 < tokens.len() => {
                if let (Ok(x), Ok(y)) = (tokens[i + 1].parse::<f64>(), tokens[i + 2].parse::<f64>())
                {
                    // 如果已有路径点，先绘制之前的路径
                    if all_points.len() >= 2 {
                        layer.add_line(Line {
                            points: std::mem::take(&mut all_points),
                            is_closed: false,
                        });
                    }
                    current_x = x;
                    current_y = y;
                    subpath_start_x = x;
                    subpath_start_y = y;
                    all_points.push((
                        Point::new(Mm(x as f32), Mm((page_height - y) as f32)),
                        false,
                    ));
                    has_path = true;
                    i += 3;
                } else {
                    i += 1;
                }
            }
            "L" if i + 2 < tokens.len() => {
                if let (Ok(x), Ok(y)) = (tokens[i + 1].parse::<f64>(), tokens[i + 2].parse::<f64>())
                {
                    all_points.push((
                        Point::new(Mm(x as f32), Mm((page_height - y) as f32)),
                        false,
                    ));
                    current_x = x;
                    current_y = y;
                    i += 3;
                } else {
                    i += 1;
                }
            }
            // Z：闭合路径，回到子路径起点
            "Z" => {
                if has_path {
                    all_points.push((
                        Point::new(
                            Mm(subpath_start_x as f32),
                            Mm((page_height - subpath_start_y) as f32),
                        ),
                        false,
                    ));
                    layer.add_line(Line {
                        points: std::mem::take(&mut all_points),
                        is_closed: true,
                    });
                    current_x = subpath_start_x;
                    current_y = subpath_start_y;
                }
                i += 1;
            }
            // C x1 y1 x2 y2 x y：三次贝塞尔曲线（用 8 段直线近似）
            "C" if i + 6 < tokens.len() => {
                if let (Ok(x1), Ok(y1), Ok(x2), Ok(y2), Ok(x), Ok(y)) = (
                    tokens[i + 1].parse::<f64>(),
                    tokens[i + 2].parse::<f64>(),
                    tokens[i + 3].parse::<f64>(),
                    tokens[i + 4].parse::<f64>(),
                    tokens[i + 5].parse::<f64>(),
                    tokens[i + 6].parse::<f64>(),
                ) {
                    // B(t) = (1-t)^3*P0 + 3*(1-t)^2*t*P1 + 3*(1-t)*t^2*P2 + t^3*P3
                    let segments = 8;
                    for s in 1..=segments {
                        let t = f64::from(s) / f64::from(segments);
                        let one_t = 1.0 - t;
                        let bx = one_t.powi(3) * current_x
                            + 3.0 * one_t.powi(2) * t * x1
                            + 3.0 * one_t * t.powi(2) * x2
                            + t.powi(3) * x;
                        let by = one_t.powi(3) * current_y
                            + 3.0 * one_t.powi(2) * t * y1
                            + 3.0 * one_t * t.powi(2) * y2
                            + t.powi(3) * y;
                        all_points.push((
                            Point::new(Mm(bx as f32), Mm((page_height - by) as f32)),
                            false,
                        ));
                    }
                    current_x = x;
                    current_y = y;
                    i += 7;
                } else {
                    i += 1;
                }
            }
            // Q x1 y1 x y：二次贝塞尔曲线（用 8 段直线近似）
            "Q" if i + 4 < tokens.len() => {
                if let (Ok(x1), Ok(y1), Ok(x), Ok(y)) = (
                    tokens[i + 1].parse::<f64>(),
                    tokens[i + 2].parse::<f64>(),
                    tokens[i + 3].parse::<f64>(),
                    tokens[i + 4].parse::<f64>(),
                ) {
                    // B(t) = (1-t)^2*P0 + 2*(1-t)*t*P1 + t^2*P2
                    let segments = 8;
                    for s in 1..=segments {
                        let t = f64::from(s) / f64::from(segments);
                        let one_t = 1.0 - t;
                        let bx = one_t.powi(2) * current_x + 2.0 * one_t * t * x1 + t.powi(2) * x;
                        let by = one_t.powi(2) * current_y + 2.0 * one_t * t * y1 + t.powi(2) * y;
                        all_points.push((
                            Point::new(Mm(bx as f32), Mm((page_height - by) as f32)),
                            false,
                        ));
                    }
                    current_x = x;
                    current_y = y;
                    i += 5;
                } else {
                    i += 1;
                }
            }
            _ => i += 1,
        }
    }

    // 绘制剩余路径
    if all_points.len() >= 2 {
        layer.add_line(Line {
            points: all_points,
            is_closed: false,
        });
    }
}

// ─── 图片格式转换 ────────────────────────────────────────────────────────────

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

/// 对应 Java: CGTransformMap（Rust 命名别名）。
pub type CGTransformMap = CgTransformMap;

/// 对应 Java: SVGExporter（Rust 命名别名）。
pub type SVGExporter = SvgExporter;

/// 对应 Java: ImageConverter（Rust 命名别名）。
pub type ImageConverter = ImageExporter;

/// 对应 Java: PDFConverter（Rust 命名别名）。
pub type PDFConverter = PdfExporter;

/// 对应 Java: TextConverter（Rust 命名别名）。
pub type TextConverter = TextExporter;

#[cfg(test)]
#[allow(clippy::items_after_statements)]
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

    // ─── 新增测试：CJK 检测 ─────────────────────────────────────────────────

    #[test]
    fn test_contains_cjk_chinese() {
        assert!(contains_cjk("你好世界"));
        assert!(contains_cjk("Hello 你好"));
        assert!(contains_cjk("測試")); // 繁体
    }

    #[test]
    fn test_contains_cjk_pure_ascii() {
        assert!(!contains_cjk("Hello World"));
        assert!(!contains_cjk("12345"));
        assert!(!contains_cjk(""));
    }

    // ─── 新增测试：BMP 编码 ─────────────────────────────────────────────────

    #[test]
    fn test_encode_raw_pixels_to_bmp_rgb() {
        // 2x2 RGB 图片（12 字节原始数据）
        let raw: Vec<u8> = vec![
            255, 0, 0, // 红
            0, 255, 0, // 绿
            0, 0, 255, // 蓝
            255, 255, 0, // 黄
        ];
        let bmp = encode_raw_pixels_to_bmp(&raw, 2, 2, "DeviceRGB", 8);
        assert!(bmp.is_some());
        let bmp = bmp.unwrap();
        // BMP 文件头：BM 签名
        assert_eq!(&bmp[0..2], b"BM");
        // 宽度 = 2
        assert_eq!(u32::from_le_bytes([bmp[18], bmp[19], bmp[20], bmp[21]]), 2);
        // 高度 = 2
        assert_eq!(u32::from_le_bytes([bmp[22], bmp[23], bmp[24], bmp[25]]), 2);
        // 24 位色深
        assert_eq!(u16::from_le_bytes([bmp[28], bmp[29]]), 24);
    }

    #[test]
    fn test_encode_raw_pixels_to_bmp_gray() {
        // 2x1 灰度图片（2 字节原始数据）
        let raw: Vec<u8> = vec![0, 255];
        let bmp = encode_raw_pixels_to_bmp(&raw, 2, 1, "DeviceGray", 8);
        assert!(bmp.is_some());
        let bmp = bmp.unwrap();
        assert_eq!(&bmp[0..2], b"BM");
        // 8 位色深
        assert_eq!(u16::from_le_bytes([bmp[28], bmp[29]]), 8);
        // 应包含 256 色调色板
        assert!(bmp.len() > 54 + 256 * 4);
    }

    #[test]
    fn test_encode_raw_pixels_to_bmp_unsupported_cs() {
        let raw: Vec<u8> = vec![0; 12];
        // 不支持的色彩空间
        assert!(encode_raw_pixels_to_bmp(&raw, 2, 2, "DeviceCMYK", 8).is_none());
    }

    // ─── 新增测试：路径增强 ─────────────────────────────────────────────────

    #[test]
    fn test_ofd_to_pdf_with_closed_path() {
        use easyofd_core::{OfdPage, PathObject};
        use easyofd_writer::OfdWriter;

        let mut writer = OfdWriter::new();
        let mut page = OfdPage::new(210.0, 297.0);
        // 使用 Z 闭合路径（三角形）
        page.add_path(PathObject::new(0.0, 0.0, "M 50 50 L 100 50 L 75 100 Z"));
        writer.add_page(page);
        let ofd_bytes = writer.build().unwrap();

        let ofd_path = "/tmp/test_closed_path.ofd";
        let pdf_path = "/tmp/test_closed_path.pdf";
        std::fs::write(ofd_path, &ofd_bytes).unwrap();

        let result = ofd_to_pdf(ofd_path, pdf_path, &ConvertOptions::default());
        assert!(result.is_ok(), "闭合路径转换应该成功: {:?}", result.err());

        let _ = std::fs::remove_file(ofd_path);
        let _ = std::fs::remove_file(pdf_path);
    }

    #[test]
    fn test_ofd_to_pdf_with_bezier_curve() {
        use easyofd_core::{OfdPage, PathObject};
        use easyofd_writer::OfdWriter;

        let mut writer = OfdWriter::new();
        let mut page = OfdPage::new(210.0, 297.0);
        // 三次贝塞尔曲线
        page.add_path(PathObject::new(0.0, 0.0, "M 10 10 C 30 80 70 80 90 10"));
        // 二次贝塞尔曲线
        page.add_path(PathObject::new(0.0, 0.0, "M 10 10 Q 50 100 90 10"));
        writer.add_page(page);
        let ofd_bytes = writer.build().unwrap();

        let ofd_path = "/tmp/test_bezier.ofd";
        let pdf_path = "/tmp/test_bezier.pdf";
        std::fs::write(ofd_path, &ofd_bytes).unwrap();

        let result = ofd_to_pdf(ofd_path, pdf_path, &ConvertOptions::default());
        assert!(result.is_ok(), "贝塞尔曲线转换应该成功: {:?}", result.err());

        let _ = std::fs::remove_file(ofd_path);
        let _ = std::fs::remove_file(pdf_path);
    }

    #[test]
    fn test_ofd_to_pdf_with_fill_color() {
        use easyofd_core::{OfdPage, PathObject};
        use easyofd_writer::OfdWriter;

        let mut writer = OfdWriter::new();
        let mut page = OfdPage::new(210.0, 297.0);
        // 带填充色的矩形
        let rect = PathObject::rect(20.0, 20.0, 80.0, 40.0).fill_color(0xFF_0000);
        page.add_path(rect);
        writer.add_page(page);
        let ofd_bytes = writer.build().unwrap();

        let ofd_path = "/tmp/test_fill_color.ofd";
        let pdf_path = "/tmp/test_fill_color.pdf";
        std::fs::write(ofd_path, &ofd_bytes).unwrap();

        let result = ofd_to_pdf(ofd_path, pdf_path, &ConvertOptions::default());
        assert!(result.is_ok(), "填充色路径转换应该成功: {:?}", result.err());

        let _ = std::fs::remove_file(ofd_path);
        let _ = std::fs::remove_file(pdf_path);
    }

    // ─── 新增测试：PDF 图片嵌入 ──────────────────────────────────────────────

    #[test]
    fn test_ofd_to_pdf_with_image() {
        use easyofd_core::{ImageFormat, ImageObject, OfdPage};
        use easyofd_writer::OfdWriter;

        // 用 image crate 生成一个 2x2 红色 PNG
        let img_buf = image::ImageBuffer::<image::Rgb<u8>, _>::from_fn(2, 2, |_x, _y| {
            image::Rgb([255u8, 0, 0])
        });
        let mut png_cursor = std::io::Cursor::new(Vec::new());
        img_buf
            .write_to(&mut png_cursor, image::ImageFormat::Png)
            .unwrap();
        let png_data = png_cursor.into_inner();

        // 构建含图片的 OFD
        let mut writer = OfdWriter::new();
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_text(TextObject::new(10.0, 10.0, "带图片的页面"));
        page.add_image(ImageObject::new(
            50.0,
            50.0,
            30.0,
            20.0,
            png_data,
            ImageFormat::Png,
        ));
        writer.add_page(page);
        let ofd_bytes = writer.build().unwrap();

        let ofd_path = "/tmp/test_image_pdf.ofd";
        let pdf_path = "/tmp/test_image_pdf.pdf";
        std::fs::write(ofd_path, &ofd_bytes).unwrap();

        let result = ofd_to_pdf(ofd_path, pdf_path, &ConvertOptions::default());
        assert!(
            result.is_ok(),
            "带图片的 OFD→PDF 转换应该成功: {:?}",
            result.err()
        );

        // 验证 PDF 文件
        let pdf_data = std::fs::read(pdf_path).unwrap();
        assert!(pdf_data.starts_with(b"%PDF"), "输出应为合法 PDF");
        // 含图片的 PDF 应比纯文本的大（图片像素数据 + XObject 引用）
        assert!(
            pdf_data.len() > 500,
            "含图片的 PDF 应有合理的文件大小，实际 {} bytes",
            pdf_data.len()
        );

        let _ = std::fs::remove_file(ofd_path);
        let _ = std::fs::remove_file(pdf_path);
    }

    // ─── Java 名称别名测试 ─────────────────────────────────────────────────

    #[test]
    fn test_lib_enum() {
        assert_eq!(Lib::Itext.name(), "iText");
        assert_eq!(Lib::default_backend(), Lib::PrintPdf);
    }

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert!((config.dpi - 72.0).abs() < f32::EPSILON);
        assert!(config.anti_aliasing);
    }

    #[test]
    fn test_svg_maker_exclusion() {
        assert_eq!(
            SVGMaker::replacement(),
            "easyofd_convert::exporter::SvgExporter"
        );
    }

    #[test]
    fn test_html_maker_exclusion() {
        assert!(HtmlMaker::replacement().contains("Element"));
    }

    #[test]
    fn test_pdfbox_maker_exclusion() {
        assert!(PdfboxMaker::replacement().contains("PdfboxMaker"));
    }

    #[test]
    fn test_itext_maker_exclusion() {
        assert!(ItextMaker::replacement().contains("ItextMaker"));
    }

    #[test]
    fn test_awt_maker_exclusion() {
        assert!(AWTMaker::replacement().contains("AWTMaker"));
    }

    #[test]
    fn test_pdf_exporter_itext_exclusion() {
        assert_eq!(
            PDFExporterIText::replacement(),
            "easyofd_convert::exporter::PdfExporter"
        );
    }

    #[test]
    fn test_pdf_exporter_pdfbox_exclusion() {
        assert_eq!(
            PDFExporterPDFBox::replacement(),
            "easyofd_convert::exporter::PdfExporter"
        );
    }

    #[test]
    fn test_memory_ttf_data_stream() {
        let data = vec![0x00, 0x01, 0x00, 0x00];
        let mts = MemoryTTFDataStream::new(data);
        assert_eq!(mts.len(), 4);
        let mut stream = mts.as_stream();
        assert_eq!(stream.read_u16(), Some(1));
    }

    #[test]
    fn test_ttf_data_stream_alias() {
        let data = [0x48, 0x65, 0x6C, 0x6C, 0x6F];
        let mut stream = font::TTFDataStream::new(&data);
        let bytes = stream.read_bytes(5).unwrap();
        assert_eq!(bytes, b"Hello");
    }

    #[test]
    fn test_font_loader() {
        let mut loader = FontLoader::new();
        loader.register_font("TestFont", "/usr/share/fonts/test.ttf");
        assert!(loader.font_map().contains_key("TestFont"));
    }

    #[test]
    fn test_pdf_font_wrapper() {
        let wrapper = PdfFontWrapper::new("SimSun", "SimSun").with_subset_tag("ABCDEF");
        assert_eq!(wrapper.full_name(), "ABCDEF+SimSun");
    }

    #[test]
    fn test_type1_seg_split_parser() {
        assert!(!Type1SegSplitParser::is_pfb(&[]));
        assert!(Type1SegSplitParser::parse(&[]).is_empty());
    }

    #[test]
    fn test_font_draw_path_provider() {
        struct Mock;
        impl FontDrawPathProvider for Mock {
            fn glyph_path(&self, cp: u32) -> Option<GlyphPath> {
                if cp == 0x41 {
                    Some(GlyphPath::default())
                } else {
                    None
                }
            }
            fn glyph_path_by_id(&self, _: u32) -> Option<GlyphPath> {
                None
            }
        }
        let provider = Mock;
        assert!(provider.has_glyph(0x41));
        assert!(!provider.has_glyph(0x42));
    }

    #[test]
    fn test_font_utils() {
        assert!(font::FontUtils::is_cjk_font("SimSun"));
        assert!(!font::FontUtils::is_cjk_font("Arial"));
    }

    #[test]
    fn test_image_media() {
        let media = ImageMedia::new(
            vec![0xFF],
            image::image_media::MediaImageFormat::Jpeg,
            100,
            200,
        );
        assert_eq!(media.width, 100);
        assert_eq!(media.data_size(), 1);
    }

    #[test]
    fn test_convert_helper_lib_alias() {
        assert_eq!(ConvertHelperLib::default_backend(), Lib::PrintPdf);
    }

    #[test]
    fn test_common_util_module_alias() {
        let px = utils::CommonUtil::millimeters_to_pixel(25.4, 72.0);
        assert!((px - 72.0).abs() < 0.01);
    }

    #[test]
    fn test_point_util_module_alias() {
        let ctm = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];
        let [x, y] = utils::PointUtil::ctm_transform_point(5.0, 10.0, &ctm);
        assert!((x - 5.0).abs() < 1e-10);
        assert!((y - 10.0).abs() < 1e-10);
    }

    #[test]
    fn test_os_info_module_alias() {
        assert!(!utils::OSinfo::temp_dir_path().is_empty());
    }

    #[test]
    fn test_string_utils_module_alias() {
        assert_eq!(utils::StringUtils::escape_xml("<tag>"), "&lt;tag&gt;");
    }
}
pub mod itext_exclusions;
