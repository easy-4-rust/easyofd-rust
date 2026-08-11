//! OFD → HTML 导出器。
//!
//! 对应 Java: org.ofdrw.converter.export.HTMLExporter
//!
//! Java 版 `HTMLExporter` 将 OFD 页面导出为 HTML 文件。
//! Rust 版提供简化实现，将 OFD 内容转换为 HTML+CSS。

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
/// 支持文本对象和路径对象的 HTML 表示。
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
                    ContentObject::Image(_) => {
                        // 图片暂不支持
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
}
