use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use easyofd_core::OfdResult;
use easyofd_layout::LayoutOptions;
use easyofd_package::PackageLimits;

use crate::{
    ConversionReport, ImagePolicy, MarkdownConversionResult, MarkdownConverter, MarkdownOptions,
    OcrPolicy, OcrProvider, PageBreakStyle,
};

/// OFD 到 Markdown 的流畅配置入口。
#[derive(Clone)]
pub struct MarkdownConversionBuilder {
    source: PathBuf,
    options: MarkdownOptions,
    ocr_provider: Option<Arc<dyn OcrProvider>>,
}

impl MarkdownConversionBuilder {
    /// 从 OFD 文件创建转换构建器。
    #[must_use]
    pub fn new(source: impl AsRef<Path>) -> Self {
        Self {
            source: source.as_ref().to_path_buf(),
            options: MarkdownOptions::default(),
            ocr_provider: None,
        }
    }

    /// 限制转换页码范围，页码从 1 开始且包含边界。
    #[must_use]
    pub fn page_range(mut self, first: usize, last: usize) -> Self {
        self.options.first_page = Some(first);
        self.options.last_page = Some(last);
        self
    }

    /// 配置页面之间的 Markdown 分隔样式。
    #[must_use]
    pub fn page_breaks(mut self, style: PageBreakStyle) -> Self {
        self.options.page_break_style = style;
        self
    }

    /// 配置图片提取或忽略策略。
    #[must_use]
    pub fn images(mut self, policy: ImagePolicy) -> Self {
        self.options.image_policy = policy;
        self
    }

    /// 配置扫描页 OCR 回退实现。
    #[must_use]
    pub fn ocr(mut self, policy: OcrPolicy, provider: impl OcrProvider + 'static) -> Self {
        self.options.ocr_policy = policy;
        self.ocr_provider = Some(Arc::new(provider));
        self
    }

    /// 配置几何布局分析参数。
    #[must_use]
    pub fn layout(mut self, options: LayoutOptions) -> Self {
        self.options.layout = options;
        self
    }

    /// 配置 OFD 包安全限制。
    #[must_use]
    pub fn package_limits(mut self, limits: PackageLimits) -> Self {
        self.options.package_limits = limits;
        self
    }

    /// 转换并在内存中返回 Markdown。
    ///
    /// # Errors
    ///
    /// OFD 读取、资源导出或 Markdown 写入失败时返回错误。
    pub fn do_convert(self) -> OfdResult<MarkdownConversionResult> {
        MarkdownConverter::new(self.options)
            .with_ocr_provider_option(self.ocr_provider)
            .convert_path(self.source)
    }

    /// 将 Markdown 逐页写入输出，避免在内存中保留完整正文。
    ///
    /// # Errors
    ///
    /// OFD 读取、资源导出或输出写入失败时返回错误。
    pub fn convert_to(self, output: impl Write) -> OfdResult<ConversionReport> {
        MarkdownConverter::new(self.options)
            .with_ocr_provider_option(self.ocr_provider)
            .convert_path_to(self.source, output)
    }
}
