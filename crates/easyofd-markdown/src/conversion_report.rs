use crate::{ConversionLoss, ConversionWarning, ConvertedAsset};

/// 不包含 Markdown 正文的流式转换报告。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ConversionReport {
    /// 已转换页面数量。
    pub pages_converted: usize,
    /// 导出的图片等资源。
    pub assets: Vec<ConvertedAsset>,
    /// 非致命诊断。
    pub warnings: Vec<ConversionWarning>,
    /// 固定版式到 Markdown 的语义损失。
    pub losses: Vec<ConversionLoss>,
}
