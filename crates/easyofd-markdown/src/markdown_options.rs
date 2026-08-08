use easyofd_layout::LayoutOptions;
use easyofd_package::PackageLimits;

use crate::{ImagePolicy, OcrPolicy, PageBreakStyle};

/// OFD 到 Markdown 的转换选项。
#[derive(Debug, Clone, Default, PartialEq)]
pub struct MarkdownOptions {
    /// 第一个转换页码，从 1 开始。
    pub first_page: Option<usize>,
    /// 最后一个转换页码，从 1 开始。
    pub last_page: Option<usize>,
    /// 页面分隔策略。
    pub page_break_style: PageBreakStyle,
    /// 图片处理策略。
    pub image_policy: ImagePolicy,
    /// 可选 OCR 的触发策略；仅设置策略而没有 Provider 时不会调用外部能力。
    pub ocr_policy: OcrPolicy,
    /// 几何布局分析参数。
    pub layout: LayoutOptions,
    /// OFD ZIP 包安全限制。
    pub package_limits: PackageLimits,
}
