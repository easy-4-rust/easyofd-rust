//! 最终页面字形位置。
/// 单个字形在最终OFD页面空间的位置和线性变换。
#[derive(Debug, Clone, PartialEq)]
pub struct GlyphPosition {
    /// Unicode字符。
    pub character: char,
    /// 显式字体字形索引。
    pub glyph_id: Option<u32>,
    /// 页面原点，毫米。
    pub origin_mm: [f64; 2],
    /// 字形的线性变换矩阵。
    pub linear_ctm: [f64; 4],
}
