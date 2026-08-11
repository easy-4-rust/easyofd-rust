//! OFD → HTML 导出器。
//!
//! 对应 Java: org.ofdrw.converter.export.HTMLExporter
//!
//! Java 版 `HTMLExporter` 将 OFD 页面导出为 HTML 文件。
//! Rust 版提供简化实现，将 OFD 内容转换为 HTML+CSS。
//! 支持文本、路径和图片（data URI + base64 编码）对象。

use std::fmt::Write;
use std::path::Path;

use easyofd_core::{ContentObject, OfdError, OfdResult};
use easyofd_reader::OfdReader;

use super::Exporter;
use crate::ConvertOptions;

/// OFD → HTML 导出器。
///
/// 对应 Java: `org.ofdrw.converter.export.HTMLExporter`
///
/// 将 OFD 页面内容转换为 HTML 文件。
/// 支持文本对象、路径对象和图片对象（data URI + base64 编码）的 HTML 表示。
pub struct HTMLExporter {
    /// 转换选项。
    options: ConvertOptions,
}

impl HTMLExporter {
    /// 创建新的 HTML 导出器。
    #[must_use]
    pub fn new(options: ConvertOptions) -> Self {
        Self { options }
    }

    /// 使用默认选项创建 HTML 导出器。
    #[must_use]
    pub fn with_defaults() -> Self {
        Self {
            options: ConvertOptions::default(),
        }
    }

    /// 获取转换选项的引用。
    #[must_use]
    pub fn options(&self) -> &ConvertOptions {
        &self.options
    }

    /// 设置转换选项。
    pub fn set_options(&mut self, options: ConvertOptions) {
        self.options = options;
    }
}

impl Exporter for HTMLExporter {
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

        let mut html = String::new();
        let _ = write!(
            html,
            r#"<!DOCTYPE html>
<html>
<head>
<meta charset="UTF-8">
<style>
  .page {{ position: relative; border: 1px solid #ccc; margin: 10px; }}
  .text {{ position: absolute; white-space: nowrap; }}
</style>
</head>
<body>
"#
        );

        for page_idx in range {
            let page = &pages[page_idx];
            let _ = write!(
                html,
                r#"<div class="page" style="width:{w}mm;height:{h}mm;">"#,
                w = page.width,
                h = page.height
            );

            for content in &page.content {
                match content {
                    ContentObject::Text(text) => {
                        let escaped = text
                            .text
                            .replace('&', "&amp;")
                            .replace('<', "&lt;")
                            .replace('>', "&gt;");
                        let _ = write!(
                            html,
                            r#"<div class="text" style="left:{x}mm;top:{y}mm;font-size:{s}mm;font-family:{f};">{t}</div>"#,
                            x = text.x,
                            y = text.y,
                            s = text.size,
                            f = text.font,
                            t = escaped,
                        );
                    }
                    ContentObject::Path(path) => {
                        // 路径对象用 SVG 内联表示
                        let r = (path.stroke_color >> 16) & 0xFF;
                        let g = (path.stroke_color >> 8) & 0xFF;
                        let b = path.stroke_color & 0xFF;
                        let _ = write!(
                            html,
                            "<svg style=\"position:absolute;left:{}mm;top:{}mm;\" width=\"100%\" height=\"100%\"><path d=\"{}\" stroke=\"#{r:02X}{g:02X}{b:02X}\" stroke-width=\"{}\" fill=\"none\"/></svg>",
                            path.x, path.y, path.path_data, path.stroke_width,
                        );
                    }
                    ContentObject::Image(img) => {
                        let fmt_str = match img.format {
                            easyofd_core::ImageFormat::Png => "png",
                            easyofd_core::ImageFormat::Jpeg => "jpeg",
                            easyofd_core::ImageFormat::Bmp => "bmp",
                            easyofd_core::ImageFormat::Tiff => "tiff",
                        };
                        let b64 = base64_encode(&img.data);
                        let _ = write!(
                            html,
                            r#"<img src="data:image/{fmt_str};base64,{b64}" style="position:absolute;left:{}mm;top:{}mm;width:{}mm;height:{}mm;" />"#,
                            img.x, img.y, img.width, img.height,
                        );
                    }
                }
            }

            html.push_str("</div>\n");
        }

        html.push_str("</body>\n</html>\n");
        std::fs::write(target, &html).map_err(OfdError::Io)?;
        Ok(())
    }
}

/// 将原始字节编码为 base64 字符串（标准字母表，含 `+`、`/`、`=` 填充）。
///
/// 内联实现，避免引入额外依赖。
fn base64_encode(data: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b0 = u32::from(chunk[0]);
        let b1 = if chunk.len() > 1 {
            u32::from(chunk[1])
        } else {
            0
        };
        let b2 = if chunk.len() > 2 {
            u32::from(chunk[2])
        } else {
            0
        };
        let triple = (b0 << 16) | (b1 << 8) | b2;
        result.push(char::from(TABLE[((triple >> 18) & 0x3F) as usize]));
        result.push(char::from(TABLE[((triple >> 12) & 0x3F) as usize]));
        if chunk.len() > 1 {
            result.push(char::from(TABLE[((triple >> 6) & 0x3F) as usize]));
        } else {
            result.push('=');
        }
        if chunk.len() > 2 {
            result.push(char::from(TABLE[(triple & 0x3F) as usize]));
        } else {
            result.push('=');
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use easyofd_core::{OfdPage, PathObject, TextObject};
    use easyofd_writer::OfdWriter;

    #[test]
    fn test_html_exporter_new() {
        let options = ConvertOptions {
            pages: 0..5,
            page_size: Some((210.0, 297.0)),
        };
        let exporter = HTMLExporter::new(options);
        assert_eq!(exporter.options().pages, 0..5);
    }

    #[test]
    fn test_html_exporter_with_defaults() {
        let exporter = HTMLExporter::with_defaults();
        assert!(exporter.options().pages.is_empty());
    }

    #[test]
    fn test_html_exporter_convert() {
        let mut writer = OfdWriter::new();
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_text(TextObject::new(10.0, 20.0, "HTML 导出测试"));
        page.add_path(PathObject::new(0.0, 0.0, "M 10 10 L 100 100"));
        writer.add_page(page);
        let ofd_bytes = writer.build().unwrap();

        let ofd_path = "/tmp/test_html_exporter.ofd";
        let html_path = "/tmp/test_html_exporter.html";
        std::fs::write(ofd_path, &ofd_bytes).unwrap();

        let exporter = HTMLExporter::with_defaults();
        let result = exporter.convert(Path::new(ofd_path), Path::new(html_path));
        assert!(result.is_ok(), "HTML 导出应该成功: {:?}", result.err());
        assert!(Path::new(html_path).exists());

        let output = std::fs::read_to_string(html_path).unwrap();
        assert!(output.contains("<!DOCTYPE html>"));
        assert!(output.contains("HTML 导出测试"));
        assert!(output.contains("<path"));

        let _ = std::fs::remove_file(ofd_path);
        let _ = std::fs::remove_file(html_path);
    }

    #[test]
    fn test_html_exporter_set_options() {
        let mut exporter = HTMLExporter::with_defaults();
        let options = ConvertOptions {
            pages: 1..3,
            page_size: None,
        };
        exporter.set_options(options);
        assert_eq!(exporter.options().pages, 1..3);
    }

    #[test]
    fn test_html_exporter_missing_file() {
        let exporter = HTMLExporter::with_defaults();
        let result = exporter.convert(Path::new("/nonexistent.ofd"), Path::new("/tmp/out.html"));
        assert!(result.is_err());
    }

    #[test]
    fn test_base64_encode_html() {
        assert_eq!(base64_encode(b""), "");
        assert_eq!(base64_encode(b"f"), "Zg==");
        assert_eq!(base64_encode(b"foobar"), "Zm9vYmFy");
    }

    #[test]
    fn test_html_exporter_with_image() {
        use easyofd_core::{ImageFormat, ImageObject};

        let mut writer = OfdWriter::new();
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_text(TextObject::new(10.0, 20.0, "HTML 图片测试"));
        // 创建一个最小的 1x1 PNG
        let min_png = create_minimal_png_for_html();
        page.add_image(ImageObject::new(
            50.0,
            50.0,
            30.0,
            20.0,
            min_png,
            ImageFormat::Png,
        ));
        writer.add_page(page);
        let ofd_bytes = writer.build().unwrap();

        let ofd_path = "/tmp/test_html_image.ofd";
        let html_path = "/tmp/test_html_image.html";
        std::fs::write(ofd_path, &ofd_bytes).unwrap();

        let exporter = HTMLExporter::with_defaults();
        let result = exporter.convert(Path::new(ofd_path), Path::new(html_path));
        assert!(result.is_ok(), "HTML 导出含图应该成功: {:?}", result.err());

        let output = std::fs::read_to_string(html_path).unwrap();
        assert!(output.contains("<!DOCTYPE html>"));
        assert!(output.contains("HTML 图片测试"));
        assert!(output.contains("<img"), "HTML 应包含 <img> 元素");
        assert!(
            output.contains("data:image/png;base64,"),
            "图片应以 base64 data URI 嵌入"
        );

        let _ = std::fs::remove_file(ofd_path);
        let _ = std::fs::remove_file(html_path);
    }

    /// 生成一个最小合法 PNG（1x1 白色像素），用于测试。
    fn create_minimal_png_for_html() -> Vec<u8> {
        let img_buf = image::ImageBuffer::<image::Rgb<u8>, _>::from_fn(1, 1, |_x, _y| {
            image::Rgb([255u8, 255, 255])
        });
        let mut cursor = std::io::Cursor::new(Vec::new());
        img_buf
            .write_to(&mut cursor, image::ImageFormat::Png)
            .unwrap();
        cursor.into_inner()
    }
}
