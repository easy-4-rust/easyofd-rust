//! 转换选项。

/// 转换选项。
#[derive(Debug, Clone)]
pub struct ConvertOptions {
    /// 要转换的页面范围（0-based，空 = 所有页面）。
    pub pages: std::ops::Range<usize>,
    /// 输出页面尺寸覆盖（宽, 高）mm。None = 保留原始。
    pub page_size: Option<(f64, f64)>,
}

impl Default for ConvertOptions {
    fn default() -> Self {
        Self {
            pages: 0..0, // 空 = 所有
            page_size: None,
        }
    }
}
