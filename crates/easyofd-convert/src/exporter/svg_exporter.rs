//! OFD → SVG 导出器。

use std::fmt::Write;
use std::path::Path;

use easyofd_core::{ContentObject, OfdError, OfdResult};
use easyofd_reader::OfdReader;

use super::Exporter;
use crate::ConvertOptions;

/// OFD → SVG 导出器。
///
/// 对应 Java: org.ofdrw.converter.ofdconverter.SVGConverter
///
/// 将 OFD 页面内容转换为 SVG 矢量图形。
/// 支持 TextObject → `<text>` 和 PathObject → `<path>` 的映射。
pub struct SvgExporter {
    /// 转换选项。
    options: ConvertOptions,
}

impl SvgExporter {
    /// 创建新的 SVG 导出器。
    pub fn new(options: ConvertOptions) -> Self {
        Self { options }
    }

    /// 使用默认选项创建 SVG 导出器。
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

impl Exporter for SvgExporter {
    fn convert(&self, source: &Path, target: &Path) -> OfdResult<()> {
        let ofd_bytes = std::fs::read(source).map_err(OfdError::Io)?;
        let reader = OfdReader::from_bytes(&ofd_bytes)?;

        let pages = reader.pages();
        let range = if self.options.pages.is_empty() {
            0..pages.len()
        } else {
            self.options.pages.start.min(pages.len())..self.options.pages.end.min(pages.len())
        };

        if range.is_empty() {
            return Err(OfdError::Conversion("没有可转换的页面".into()));
        }

        // 对于单页导出，直接输出到 target
        // 对于多页导出，使用 target 作为基础文件名，添加页码后缀
        let is_single_page = range.len() == 1;

        for page_idx in range {
            let page = &pages[page_idx];
            let svg_content = render_page_to_svg(page)?;

            let output_path = if is_single_page {
                target.to_path_buf()
            } else {
                let stem = target
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("page");
                let ext = target.extension().and_then(|s| s.to_str()).unwrap_or("svg");
                target.with_file_name(format!("{stem}_{page_idx}.{ext}"))
            };

            std::fs::write(&output_path, &svg_content).map_err(OfdError::Io)?;
        }

        Ok(())
    }
}

/// 将 OFD 页面渲染为 SVG 字符串。
///
/// 遍历页面中的所有内容对象：
/// - TextObject → `<text>` 元素
/// - PathObject → `<path>` 元素
/// - ImageObject → 跳过（当前版本不支持图片嵌入）
fn render_page_to_svg(page: &easyofd_core::OfdPage) -> OfdResult<String> {
    let width = page.width;
    let height = page.height;

    let mut svg = String::new();
    let _ = write!(
        svg,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {width} {height}" width="{width}mm" height="{height}mm">
"#
    );

    for content in &page.content {
        match content {
            ContentObject::Text(text) => {
                // SVG Y 轴向下，与 OFD 一致
                let x = text.x;
                let y = text.y;
                let font_size = text.size;
                let font_family = &text.font;
                let fill_color = format_color(text.color);
                let weight = if text.weight >= 700 { "bold" } else { "normal" };
                let style = if text.italic { "italic" } else { "normal" };

                // XML 转义文本内容
                let escaped_text = xml_escape(&text.text);

                let _ = writeln!(
                    svg,
                    r#"  <text x="{x}" y="{y}" font-family="{font_family}" font-size="{font_size}" fill="{fill_color}" font-weight="{weight}" font-style="{style}">{escaped_text}</text>"#
                );
            }
            ContentObject::Path(path) => {
                let stroke_color = format_color(path.stroke_color);
                let stroke_width = path.stroke_width;
                let fill_color = match path.fill_color {
                    Some(color) => format_color(color),
                    None => "none".to_string(),
                };
                let d = convert_path_data(&path.path_data, path.x, path.y);

                let _ = writeln!(
                    svg,
                    r#"  <path d="{d}" stroke="{stroke_color}" stroke-width="{stroke_width}" fill="{fill_color}" />"#
                );
            }
            ContentObject::Image(_) => {
                // 图片对象当前版本不支持 SVG 嵌入
                // TODO: 支持图片嵌入为 <image> 元素
            }
        }
    }

    svg.push_str("</svg>\n");
    Ok(svg)
}

/// 将 OFD 路径数据转换为 SVG 路径数据。
///
/// OFD 路径命令与 SVG 路径命令基本相同（M/L/C/Q/Z），
/// 但需要加上对象的偏移量 (x, y)。
fn convert_path_data(path_data: &str, offset_x: f64, offset_y: f64) -> String {
    let tokens: Vec<&str> = path_data.split_whitespace().collect();
    let mut result = String::new();
    let mut i = 0;

    while i < tokens.len() {
        match tokens[i] {
            "M" if i + 2 < tokens.len() => {
                if let (Ok(x), Ok(y)) = (tokens[i + 1].parse::<f64>(), tokens[i + 2].parse::<f64>())
                {
                    let _ = write!(result, "M{} {} ", x + offset_x, y + offset_y);
                    i += 3;
                } else {
                    i += 1;
                }
            }
            "L" if i + 2 < tokens.len() => {
                if let (Ok(x), Ok(y)) = (tokens[i + 1].parse::<f64>(), tokens[i + 2].parse::<f64>())
                {
                    let _ = write!(result, "L{} {} ", x + offset_x, y + offset_y);
                    i += 3;
                } else {
                    i += 1;
                }
            }
            "C" if i + 6 < tokens.len() => {
                if let (Ok(x1), Ok(y1), Ok(x2), Ok(y2), Ok(x), Ok(y)) = (
                    tokens[i + 1].parse::<f64>(),
                    tokens[i + 2].parse::<f64>(),
                    tokens[i + 3].parse::<f64>(),
                    tokens[i + 4].parse::<f64>(),
                    tokens[i + 5].parse::<f64>(),
                    tokens[i + 6].parse::<f64>(),
                ) {
                    let _ = write!(
                        result,
                        "C{} {} {} {} {} {} ",
                        x1 + offset_x,
                        y1 + offset_y,
                        x2 + offset_x,
                        y2 + offset_y,
                        x + offset_x,
                        y + offset_y
                    );
                    i += 7;
                } else {
                    i += 1;
                }
            }
            "Q" if i + 4 < tokens.len() => {
                if let (Ok(x1), Ok(y1), Ok(x), Ok(y)) = (
                    tokens[i + 1].parse::<f64>(),
                    tokens[i + 2].parse::<f64>(),
                    tokens[i + 3].parse::<f64>(),
                    tokens[i + 4].parse::<f64>(),
                ) {
                    let _ = write!(
                        result,
                        "Q{} {} {} {} ",
                        x1 + offset_x,
                        y1 + offset_y,
                        x + offset_x,
                        y + offset_y
                    );
                    i += 5;
                } else {
                    i += 1;
                }
            }
            "Z" => {
                result.push_str("Z ");
                i += 1;
            }
            _ => i += 1,
        }
    }

    result.trim().to_string()
}

/// 将 RGB 颜色值格式化为 CSS 颜色字符串。
fn format_color(color: u32) -> String {
    let r = (color >> 16) & 0xFF;
    let g = (color >> 8) & 0xFF;
    let b = color & 0xFF;
    format!("#{r:02X}{g:02X}{b:02X}")
}

/// XML 特殊字符转义。
fn xml_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use easyofd_core::{OfdPage, PathObject, TextObject};
    use easyofd_writer::OfdWriter;

    #[test]
    fn test_svg_exporter_new() {
        let options = ConvertOptions {
            pages: 0..5,
            page_size: Some((210.0, 297.0)),
        };
        let exporter = SvgExporter::new(options);
        assert_eq!(exporter.options().pages, 0..5);
        assert_eq!(exporter.options().page_size, Some((210.0, 297.0)));
    }

    #[test]
    fn test_svg_exporter_with_defaults() {
        let exporter = SvgExporter::with_defaults();
        assert!(exporter.options().pages.is_empty());
        assert!(exporter.options().page_size.is_none());
    }

    #[test]
    fn test_svg_exporter_convert() {
        let mut writer = OfdWriter::new();
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_text(TextObject::new(10.0, 20.0, "SVG 测试"));
        page.add_path(PathObject::new(0.0, 0.0, "M 10 10 L 100 100"));
        writer.add_page(page);
        let ofd_bytes = writer.build().unwrap();

        let ofd_path = "/tmp/test_svg_exporter.ofd";
        let svg_path = "/tmp/test_svg_exporter.svg";
        std::fs::write(ofd_path, &ofd_bytes).unwrap();

        let exporter = SvgExporter::with_defaults();
        let result = exporter.convert(Path::new(ofd_path), Path::new(svg_path));
        assert!(result.is_ok(), "SVG 导出应该成功: {:?}", result.err());
        assert!(Path::new(svg_path).exists());

        let output = std::fs::read_to_string(svg_path).unwrap();
        assert!(output.contains("<svg"));
        assert!(output.contains("SVG 测试"));
        assert!(output.contains("<path"));

        let _ = std::fs::remove_file(ofd_path);
        let _ = std::fs::remove_file(svg_path);
    }

    #[test]
    fn test_svg_exporter_set_options() {
        let mut exporter = SvgExporter::with_defaults();
        assert!(exporter.options().pages.is_empty());

        let options = ConvertOptions {
            pages: 2..8,
            page_size: Some((100.0, 150.0)),
        };
        exporter.set_options(options);
        assert_eq!(exporter.options().pages, 2..8);
        assert_eq!(exporter.options().page_size, Some((100.0, 150.0)));
    }

    #[test]
    fn test_svg_exporter_missing_file() {
        let exporter = SvgExporter::with_defaults();
        let result = exporter.convert(Path::new("/nonexistent.ofd"), Path::new("/tmp/out.svg"));
        assert!(result.is_err());
    }

    #[test]
    fn test_format_color() {
        assert_eq!(format_color(0x0000_0000), "#000000");
        assert_eq!(format_color(0x00FF_0000), "#FF0000");
        assert_eq!(format_color(0x0000_FF00), "#00FF00");
        assert_eq!(format_color(0x0000_00FF), "#0000FF");
        assert_eq!(format_color(0x00FF_FFFF), "#FFFFFF");
    }

    #[test]
    fn test_xml_escape() {
        assert_eq!(xml_escape("hello"), "hello");
        assert_eq!(xml_escape("a<b"), "a&lt;b");
        assert_eq!(xml_escape("a>b"), "a&gt;b");
        assert_eq!(xml_escape("a&b"), "a&amp;b");
        assert_eq!(xml_escape(r#"a"b"#), "a&quot;b");
    }

    #[test]
    fn test_convert_path_data() {
        let d = convert_path_data("M 0 0 L 10 10", 5.0, 5.0);
        assert_eq!(d, "M5 5 L15 15");
    }

    #[test]
    fn test_convert_path_data_with_curve() {
        let d = convert_path_data("M 0 0 C 10 20 30 40 50 60", 1.0, 2.0);
        assert_eq!(d, "M1 2 C11 22 31 42 51 62");
    }
}
