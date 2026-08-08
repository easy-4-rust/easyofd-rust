/// 固定版式语义无法表示为 Markdown 时的损失记录。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConversionLoss {
    /// 相关页码，从 1 开始。
    pub page: usize,
    /// 来源页面对象下标。
    pub object_index: Option<usize>,
    /// 丢失的能力代码。
    pub feature: &'static str,
    /// 实际采用的降级策略。
    pub policy: &'static str,
}
