use std::io::Write;
use std::path::Path;
use std::sync::Arc;

use easyofd_core::{ContentObject, ImageFormat, OfdError, OfdResult};
use easyofd_layout::{LayoutAnalyzer, LayoutBlock};
use easyofd_package::atomic_write;
use easyofd_reader::{OfdReader, ReadOptions};

use crate::{
    ConversionLoss, ConversionReport, ConversionWarning, ConvertedAsset, ImagePolicy,
    MarkdownConversionResult, MarkdownOptions, OcrPolicy, OcrProvider, PageBreakStyle,
};

/// OFD 页面到 Markdown 块的转换器。
#[derive(Clone)]
pub struct MarkdownConverter {
    options: MarkdownOptions,
    ocr_provider: Option<Arc<dyn OcrProvider>>,
}

impl MarkdownConverter {
    /// 使用指定转换选项创建转换器。
    #[must_use]
    pub fn new(options: MarkdownOptions) -> Self {
        Self {
            options,
            ocr_provider: None,
        }
    }

    /// 设置 OCR Provider；当策略不是 `Disabled` 时用于扫描页回退。
    #[must_use]
    pub fn with_ocr_provider(mut self, provider: impl OcrProvider + 'static) -> Self {
        self.ocr_provider = Some(Arc::new(provider));
        self
    }

    pub(crate) fn with_ocr_provider_option(
        mut self,
        provider: Option<Arc<dyn OcrProvider>>,
    ) -> Self {
        self.ocr_provider = provider;
        self
    }

    /// 转换文件并在内存中返回 Markdown。
    ///
    /// # Errors
    ///
    /// OFD 解析、资源导出或 UTF-8 转换失败时返回错误。
    pub fn convert_path(&self, path: impl AsRef<Path>) -> OfdResult<MarkdownConversionResult> {
        let mut bytes = Vec::new();
        let report = self.convert_path_to(path, &mut bytes)?;
        let markdown =
            String::from_utf8(bytes).map_err(|error| OfdError::Conversion(error.to_string()))?;
        Ok(MarkdownConversionResult { markdown, report })
    }

    /// 转换文件并将 Markdown 逐页写入输出。
    ///
    /// # Errors
    ///
    /// OFD 解析、资源导出或输出失败时返回错误。
    #[allow(clippy::too_many_lines)]
    pub fn convert_path_to(
        &self,
        path: impl AsRef<Path>,
        mut output: impl Write,
    ) -> OfdResult<ConversionReport> {
        let analyzer = LayoutAnalyzer::new(self.options.layout);
        let image_policy = self.options.image_policy.clone();
        let page_break_style = self.options.page_break_style;
        let ocr_policy = self.options.ocr_policy;
        let ocr_provider = self.ocr_provider.clone();
        let mut report = ConversionReport::default();
        let read_options = ReadOptions {
            first_page: self.options.first_page,
            last_page: self.options.last_page,
            package_limits: self.options.package_limits,
        };
        OfdReader::visit_path(path, read_options, |page_number, page| {
            if report.pages_converted > 0 {
                write_page_break(&mut output, page_break_style, page_number)?;
            }
            let layout = analyzer.analyze_page(page_number, &page);
            let page_has_text = page.content.iter().any(
                |object| matches!(object, ContentObject::Text(text) if !text.text.trim().is_empty()),
            );
            for warning in layout.warnings {
                report.warnings.push(ConversionWarning {
                    page: page_number,
                    code: "UNREPRESENTED_VECTOR_PATH",
                    message: warning,
                });
            }
            for (object_index, object) in page.content.iter().enumerate() {
                if matches!(object, ContentObject::Path(_)) {
                    report.losses.push(ConversionLoss {
                        page: page_number,
                        object_index: Some(object_index),
                        feature: "VECTOR_PATH",
                        policy: "OMIT_WITH_WARNING",
                    });
                }
            }
            for block in layout.blocks {
                match block {
                    LayoutBlock::Heading { level, text, .. } => {
                        writeln!(
                            output,
                            "{} {}\n",
                            "#".repeat(usize::from(level)),
                            escape_markdown(&text)
                        )?;
                    }
                    LayoutBlock::Paragraph { text, .. } if !text.trim().is_empty() => {
                        writeln!(output, "{}\n", escape_markdown(&text))?;
                    }
                    LayoutBlock::Paragraph { .. } => {}
                    LayoutBlock::Image { source_index } => {
                        let Some(ContentObject::Image(image)) = page.content.get(source_index)
                        else {
                            continue;
                        };
                        match &image_policy {
                            ImagePolicy::Skip => report.losses.push(ConversionLoss {
                                page: page_number,
                                object_index: Some(source_index),
                                feature: "IMAGE",
                                policy: "SKIP",
                            }),
                            ImagePolicy::ExtractTo(_directory) if image.data.is_empty() => {
                                report.warnings.push(ConversionWarning {
                                    page: page_number,
                                    code: "IMAGE_RESOURCE_MISSING",
                                    message: format!("page {page_number} image {source_index} has no embedded bytes"),
                                });
                                report.losses.push(ConversionLoss {
                                    page: page_number,
                                    object_index: Some(source_index),
                                    feature: "IMAGE",
                                    policy: "MISSING_RESOURCE",
                                });
                            }
                            ImagePolicy::ExtractTo(directory) => {
                                std::fs::create_dir_all(directory)?;
                                let name = format!(
                                    "page-{page_number}-image-{}.{}",
                                    source_index + 1,
                                    image_extension(image.format)
                                );
                                let asset_path = directory.join(&name);
                                atomic_write(&asset_path, |file| {
                                    file.write_all(&image.data)?;
                                    Ok(())
                                })?;
                                let link = directory.file_name().map_or_else(
                                    || name.clone(),
                                    |folder| {
                                        Path::new(folder).join(&name).to_string_lossy().into_owned()
                                    },
                                );
                                writeln!(output, "![OFD page {page_number} image]({link})\n")?;
                                report.assets.push(ConvertedAsset {
                                    page: page_number,
                                    object_index: source_index,
                                    path: asset_path,
                                });
                            }
                        }
                        let should_ocr = matches!(ocr_policy, OcrPolicy::AllImages)
                            || matches!(ocr_policy, OcrPolicy::WhenPageHasNoText) && !page_has_text;
                        if should_ocr
                            && !image.data.is_empty()
                            && let Some(provider) = &ocr_provider
                        {
                            match provider.recognize(&image.data, image.format) {
                                Ok(Some(text)) if !text.trim().is_empty() => {
                                    writeln!(output, "{}\n", escape_markdown(text.trim()))?;
                                }
                                Ok(_) => report.losses.push(ConversionLoss {
                                    page: page_number,
                                    object_index: Some(source_index),
                                    feature: "OCR_TEXT",
                                    policy: "NO_TEXT_RECOGNIZED",
                                }),
                                Err(error) => report.warnings.push(ConversionWarning {
                                    page: page_number,
                                    code: "OCR_FAILED",
                                    message: error.to_string(),
                                }),
                            }
                        }
                    }
                }
            }
            report.pages_converted += 1;
            Ok(())
        })?;
        Ok(report)
    }
}

fn write_page_break(
    output: &mut impl Write,
    style: PageBreakStyle,
    page_number: usize,
) -> OfdResult<()> {
    match style {
        PageBreakStyle::HtmlComment => writeln!(output, "<!-- OFD page {page_number} -->\n")?,
        PageBreakStyle::HorizontalRule => writeln!(output, "---\n")?,
        PageBreakStyle::None => {}
    }
    Ok(())
}

fn image_extension(format: ImageFormat) -> &'static str {
    match format {
        ImageFormat::Jpeg => "jpg",
        ImageFormat::Png => "png",
        ImageFormat::Bmp => "bmp",
        ImageFormat::Tiff => "tiff",
    }
}

fn escape_markdown(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        if matches!(character, '\\' | '*' | '_' | '[' | ']' | '`') {
            escaped.push('\\');
        }
        escaped.push(character);
    }
    escaped
}

#[cfg(test)]
mod tests {
    use super::*;
    use easyofd_core::{ImageObject, OfdPage, PathObject, TextObject};
    use easyofd_writer::OfdWriter;

    struct TestOcr;

    impl OcrProvider for TestOcr {
        fn recognize(&self, _image: &[u8], _format: ImageFormat) -> OfdResult<Option<String>> {
            Ok(Some("recognized scan".to_string()))
        }
    }

    #[test]
    fn converts_heading_text_and_image_with_report() {
        let root = std::env::temp_dir().join("easyofd_markdown_test");
        let source = root.join("source.ofd");
        let assets = root.join("assets");
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_text(TextObject::new(10.0, 10.0, "Invoice").size(24.0));
        page.add_text(TextObject::new(10.0, 30.0, "Amount: 100"));
        page.add_image(ImageObject::png(10.0, 50.0, 20.0, 20.0, vec![1, 2, 3]));
        page.add_path(PathObject::hline(10.0, 80.0, 100.0));
        let mut writer = OfdWriter::new();
        writer.add_page(page);
        writer.build_to_file(&source).unwrap();

        let result = MarkdownConverter::new(MarkdownOptions {
            image_policy: ImagePolicy::ExtractTo(assets.clone()),
            ..MarkdownOptions::default()
        })
        .convert_path(&source)
        .unwrap();
        assert!(result.markdown.contains("# Invoice") || result.markdown.contains("## Invoice"));
        assert!(result.markdown.contains("Amount: 100"));
        assert_eq!(result.report.pages_converted, 1);
        assert_eq!(result.report.assets.len(), 1);
        assert!(
            result
                .report
                .losses
                .iter()
                .any(|loss| loss.feature == "VECTOR_PATH")
        );
        assert!(result.report.assets[0].path.exists());
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn uses_ocr_provider_for_image_only_page() {
        let root = std::env::temp_dir().join("easyofd_markdown_ocr_test");
        let source = root.join("scan.ofd");
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_image(ImageObject::png(0.0, 0.0, 210.0, 297.0, vec![1, 2, 3]));
        let mut writer = OfdWriter::new();
        writer.add_page(page);
        writer.build_to_file(&source).unwrap();

        let result = MarkdownConverter::new(MarkdownOptions {
            ocr_policy: OcrPolicy::WhenPageHasNoText,
            ..MarkdownOptions::default()
        })
        .with_ocr_provider(TestOcr)
        .convert_path(&source)
        .unwrap();
        assert!(result.markdown.contains("recognized scan"));
        let _ = std::fs::remove_dir_all(root);
    }
}
