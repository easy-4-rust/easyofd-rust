use crate::LayoutBlock;

/// 单页布局分析结果。
#[derive(Debug, Clone, PartialEq)]
pub struct LayoutResult {
    /// 从 1 开始的页码。
    pub page_number: usize,
    /// 按阅读顺序排列的语义块。
    pub blocks: Vec<LayoutBlock>,
    /// 无法无损解释的页面内容警告。
    pub warnings: Vec<String>,
}
