//! OFD → 纯文本导出器。

use std::path::Path;

use easyofd_core::{ContentObject, OfdError, OfdResult};
use easyofd_reader::OfdReader;

use super::Exporter;
use crate::ConvertOptions;

/// OFD → 纯文本导出器。
///
/// 对应 Java: org.ofdrw.converter.ofdconverter.TextConverter
///
/// 从 OFD 文件提取所有文本内容，按页面分隔输出。
pub struct TextExporter {
    /// 转换选项。
    options: ConvertOptions,
}

impl TextExporter {
    /// 创建新的文本导出器。
    pub fn new(options: ConvertOptions) -> Self {
        Self { options }
    }

    /// 使用默认选项创建文本导出器。
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

impl Exporter for TextExporter {
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

        let mut output = String::new();

        for (idx, page_idx) in range.enumerate() {
            let page = &pages[page_idx];
            let page_text = extract_page_text(page);

            if idx > 0 {
                output.push_str("\n---\n");
            }
            output.push_str(&page_text);
        }

        std::fs::write(target, &output).map_err(OfdError::Io)?;
        Ok(())
    }
}

/// 从单个页面提取文本内容。
///
/// 遍历页面中的所有文本对象，按 Y 坐标排序后拼接。
fn extract_page_text(page: &easyofd_core::OfdPage) -> String {
    let mut text_entries: Vec<(f64, &str)> = page
        .content
        .iter()
        .filter_map(|obj| {
            if let ContentObject::Text(t) = obj {
                Some((t.y, t.text.as_str()))
            } else {
                None
            }
        })
        .collect();

    // 按 Y 坐标排序（从上到下）
    text_entries.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

    text_entries
        .iter()
        .map(|(_, text)| *text)
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use easyofd_core::{OfdPage, TextObject};
    use easyofd_writer::OfdWriter;

    #[test]
    fn test_text_exporter_new() {
        let options = ConvertOptions {
            pages: 0..5,
            page_size: Some((210.0, 297.0)),
        };
        let exporter = TextExporter::new(options);
        assert_eq!(exporter.options().pages, 0..5);
        assert_eq!(exporter.options().page_size, Some((210.0, 297.0)));
    }

    #[test]
    fn test_text_exporter_with_defaults() {
        let exporter = TextExporter::with_defaults();
        assert!(exporter.options().pages.is_empty());
        assert!(exporter.options().page_size.is_none());
    }

    #[test]
    fn test_text_exporter_convert() {
        let mut writer = OfdWriter::new();
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_text(TextObject::new(10.0, 20.0, "第一行文本"));
        page.add_text(TextObject::new(10.0, 40.0, "第二行文本"));
        writer.add_page(page);
        let ofd_bytes = writer.build().unwrap();

        let ofd_path = "/tmp/test_text_exporter.ofd";
        let txt_path = "/tmp/test_text_exporter.txt";
        std::fs::write(ofd_path, &ofd_bytes).unwrap();

        let exporter = TextExporter::with_defaults();
        let result = exporter.convert(Path::new(ofd_path), Path::new(txt_path));
        assert!(result.is_ok(), "文本导出应该成功: {:?}", result.err());
        assert!(Path::new(txt_path).exists());

        let output = std::fs::read_to_string(txt_path).unwrap();
        assert!(output.contains("第一行文本"));
        assert!(output.contains("第二行文本"));

        let _ = std::fs::remove_file(ofd_path);
        let _ = std::fs::remove_file(txt_path);
    }

    #[test]
    fn test_text_exporter_multi_page() {
        let mut writer = OfdWriter::new();
        let mut page1 = OfdPage::new(210.0, 297.0);
        page1.add_text(TextObject::new(10.0, 20.0, "页面一"));
        writer.add_page(page1);

        let mut page2 = OfdPage::new(210.0, 297.0);
        page2.add_text(TextObject::new(10.0, 20.0, "页面二"));
        writer.add_page(page2);

        let ofd_bytes = writer.build().unwrap();

        let ofd_path = "/tmp/test_text_multi_page.ofd";
        let txt_path = "/tmp/test_text_multi_page.txt";
        std::fs::write(ofd_path, &ofd_bytes).unwrap();

        let exporter = TextExporter::with_defaults();
        let result = exporter.convert(Path::new(ofd_path), Path::new(txt_path));
        assert!(result.is_ok(), "多页文本导出应该成功: {:?}", result.err());

        let output = std::fs::read_to_string(txt_path).unwrap();
        assert!(output.contains("页面一"));
        assert!(output.contains("页面二"));
        assert!(output.contains("---")); // 页面分隔符

        let _ = std::fs::remove_file(ofd_path);
        let _ = std::fs::remove_file(txt_path);
    }

    #[test]
    fn test_text_exporter_set_options() {
        let mut exporter = TextExporter::with_defaults();
        assert!(exporter.options().pages.is_empty());

        let options = ConvertOptions {
            pages: 1..3,
            page_size: None,
        };
        exporter.set_options(options);
        assert_eq!(exporter.options().pages, 1..3);
    }

    #[test]
    fn test_text_exporter_missing_file() {
        let exporter = TextExporter::with_defaults();
        let result = exporter.convert(Path::new("/nonexistent.ofd"), Path::new("/tmp/out.txt"));
        assert!(result.is_err());
    }
}
