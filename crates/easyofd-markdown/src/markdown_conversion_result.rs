use crate::ConversionReport;

/// 内存转换的 Markdown 正文与转换报告。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MarkdownConversionResult {
    /// 生成的 UTF-8 Markdown。
    pub markdown: String,
    /// 资源、警告和损失信息。
    pub report: ConversionReport,
}
