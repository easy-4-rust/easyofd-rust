/// 页面布局分析选项。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LayoutOptions {
    /// 纵坐标差值不超过该值的文本被视为同一行，单位毫米。
    pub line_tolerance: f64,
    /// 字号达到该值时推断为一级标题。
    pub heading_size: f64,
    /// 相邻文字对象横向间隔达到该值时插入空格，单位毫米。
    pub word_gap: f64,
}

impl Default for LayoutOptions {
    fn default() -> Self {
        Self {
            line_tolerance: 1.5,
            heading_size: 18.0,
            word_gap: 1.5,
        }
    }
}
