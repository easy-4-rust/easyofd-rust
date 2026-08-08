use std::path::PathBuf;

/// Markdown 转换时的图片处理策略。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum ImagePolicy {
    /// 忽略图片，并在损失报告中记录。
    #[default]
    Skip,
    /// 将图片提取到指定目录并生成相对链接。
    ExtractTo(PathBuf),
}
