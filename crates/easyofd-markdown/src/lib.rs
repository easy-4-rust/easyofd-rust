//! OFD 到 Markdown 的确定性、可流式、损失可见转换。

mod conversion_loss;
mod conversion_report;
mod conversion_warning;
mod converted_asset;
mod image_policy;
mod markdown_conversion_builder;
mod markdown_conversion_result;
mod markdown_converter;
mod markdown_options;
mod ocr_policy;
mod ocr_provider;
mod page_break_style;

pub use conversion_loss::ConversionLoss;
pub use conversion_report::ConversionReport;
pub use conversion_warning::ConversionWarning;
pub use converted_asset::ConvertedAsset;
pub use image_policy::ImagePolicy;
pub use markdown_conversion_builder::MarkdownConversionBuilder;
pub use markdown_conversion_result::MarkdownConversionResult;
pub use markdown_converter::MarkdownConverter;
pub use markdown_options::MarkdownOptions;
pub use ocr_policy::OcrPolicy;
pub use ocr_provider::OcrProvider;
pub use page_break_style::PageBreakStyle;
