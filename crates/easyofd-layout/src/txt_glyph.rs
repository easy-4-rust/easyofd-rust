//! 文字字符块。
//!
//! 对应 Java: org.ofdrw.layout.element.TxtGlyph

/// 文字字符块，包含单个字符及其度量信息。
///
/// 对应 Java: ofdrw layout TxtGlyph。
#[derive(Debug, Clone, PartialEq)]
pub struct TxtGlyph {
    /// 字符。
    pub txt: char,
    /// 字号（mm）。
    pub font_size: f64,
    /// 字间距（mm）。
    pub letter_spacing: f64,
    /// 字符宽度（mm，含字间距）。由外部计算后填入。
    pub width: f64,
    /// 字符高度（mm）。由外部计算后填入。
    pub height: f64,
}

impl TxtGlyph {
    /// 创建文字字符块。
    #[must_use]
    pub fn new(txt: char, font_size: f64, letter_spacing: f64) -> Self {
        Self {
            txt,
            font_size,
            letter_spacing,
            width: 0.0,
            height: 0.0,
        }
    }

    /// 设置宽度（含字间距）。
    #[must_use]
    pub fn width(mut self, width: f64) -> Self {
        self.width = width;
        self
    }

    /// 设置高度。
    #[must_use]
    pub fn height(mut self, height: f64) -> Self {
        self.height = height;
        self
    }

    /// 获取字符宽度（含字间距）。
    ///
    /// 对应 Java: TxtGlyph#getW()。
    #[must_use]
    pub fn w(&self) -> f64 {
        self.width + self.letter_spacing
    }

    /// 获取字符高度。
    ///
    /// 对应 Java: TxtGlyph#getH()。
    #[must_use]
    pub fn h(&self) -> f64 {
        self.height
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let g = TxtGlyph::new('A', 3.0, 0.5);
        assert_eq!(g.txt, 'A');
        assert!((g.font_size - 3.0).abs() < f64::EPSILON);
        assert!((g.letter_spacing - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_builders() {
        let g = TxtGlyph::new('中', 5.0, 1.0).width(5.0).height(5.0);
        assert!((g.width - 5.0).abs() < f64::EPSILON);
        assert!((g.height - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_w_includes_letter_spacing() {
        let g = TxtGlyph::new('X', 3.0, 0.5).width(2.0).height(3.0);
        // w = width + letter_spacing = 2.0 + 0.5 = 2.5
        assert!((g.w() - 2.5).abs() < f64::EPSILON);
        assert!((g.h() - 3.0).abs() < f64::EPSILON);
    }
}
