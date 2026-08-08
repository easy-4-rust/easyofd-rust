/// 不阻止转换完成的诊断信息。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConversionWarning {
    /// 相关页码，从 1 开始。
    pub page: usize,
    /// 警告代码，便于程序稳定判断。
    pub code: &'static str,
    /// 面向用户的说明。
    pub message: String,
}
