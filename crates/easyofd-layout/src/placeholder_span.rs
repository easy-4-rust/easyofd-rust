//! 占位符元素。
//!
//! 对应 Java: org.ofdrw.layout.element.PlaceholderSpan

use crate::span::Span;

/// 占位符元素，用于在流式布局中占据固定宽度或固定数量的字符位置。
///
/// 对应 Java: ofdrw layout PlaceholderSpan。
#[derive(Debug, Clone, PartialEq)]
pub struct PlaceholderSpan {
    /// 占位数量（字符数）。
    hold_num: usize,
    /// 占位符宽度（mm）。
    ///
    /// `hold_width` 和 `hold_num` 仅运行一个有效；若 `hold_width` 存在则优先级高于 `hold_num`。
    hold_width: f64,
    /// 字号（mm）。
    font_size: f64,
    /// 字间距（mm）。
    letter_spacing: f64,
}

#[allow(clippy::cast_precision_loss)]
impl PlaceholderSpan {
    /// 通过指定宽度创建占位符（对应 Java: PlaceholderSpan(double holdWidth, double height)）。
    #[must_use]
    pub fn with_width(hold_width: f64, height: f64) -> Self {
        Self {
            hold_num: 0,
            hold_width,
            font_size: height,
            letter_spacing: 0.0,
        }
    }

    /// 通过指定数量和字号创建占位符（对应 Java: PlaceholderSpan(int holdNum, double fontSize)）。
    #[must_use]
    pub fn with_count(hold_num: usize, font_size: f64) -> Self {
        let hold_width = hold_num as f64 * font_size;
        Self {
            hold_num,
            hold_width,
            font_size,
            letter_spacing: 0.0,
        }
    }

    /// 通过复制 Span 的方式创建占位符（对应 Java: PlaceholderSpan(int holdNum, Span sp)）。
    #[must_use]
    pub fn from_span(hold_num: usize, sp: &Span) -> Self {
        let hold_width = hold_num as f64 * (sp.font_size + sp.letter_spacing);
        Self {
            hold_num,
            hold_width,
            font_size: sp.font_size,
            letter_spacing: sp.letter_spacing,
        }
    }

    /// 设置占位的字符数量（对应 Java: PlaceholderSpan#setHoldChars）。
    #[must_use]
    pub fn set_hold_chars(mut self, hold_num: usize) -> Self {
        self.hold_num = hold_num;
        self.hold_width = hold_num as f64 * (self.font_size + self.letter_spacing);
        self
    }

    /// 获取占位符数量。
    #[must_use]
    pub fn hold_num(&self) -> usize {
        self.hold_num
    }

    /// 获取占位符宽度（mm）。
    #[must_use]
    pub fn hold_width(&self) -> f64 {
        self.hold_width
    }

    /// 设置占位符宽度（mm）。
    ///
    /// 若设置了宽度，则 `hold_num` 将失效。
    #[must_use]
    pub fn set_hold_width(mut self, hold_width: f64) -> Self {
        self.hold_width = hold_width;
        self
    }

    /// 字号。
    #[must_use]
    pub fn font_size(&self) -> f64 {
        self.font_size
    }

    /// 字间距。
    #[must_use]
    pub fn letter_spacing(&self) -> f64 {
        self.letter_spacing
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_with_width() {
        let ps = PlaceholderSpan::with_width(50.0, 3.0);
        assert!((ps.hold_width() - 50.0).abs() < f64::EPSILON);
        assert!((ps.font_size() - 3.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_with_count() {
        let ps = PlaceholderSpan::with_count(10, 3.0);
        assert_eq!(ps.hold_num(), 10);
        assert!((ps.hold_width() - 30.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_from_span() {
        let sp = Span::new("test").font_size(5.0).letter_spacing(1.0);
        let ps = PlaceholderSpan::from_span(3, &sp);
        assert_eq!(ps.hold_num(), 3);
        // width = 3 * (5.0 + 1.0) = 18.0
        assert!((ps.hold_width() - 18.0).abs() < f64::EPSILON);
        assert!((ps.font_size() - 5.0).abs() < f64::EPSILON);
        assert!((ps.letter_spacing() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_set_hold_chars() {
        let ps = PlaceholderSpan::with_width(10.0, 3.0).set_hold_chars(5);
        assert_eq!(ps.hold_num(), 5);
        // width = 5 * (3.0 + 0.0) = 15.0
        assert!((ps.hold_width() - 15.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_set_hold_width() {
        let ps = PlaceholderSpan::with_count(10, 3.0).set_hold_width(100.0);
        assert!((ps.hold_width() - 100.0).abs() < f64::EPSILON);
        // hold_num 仍为 10 但 hold_width 已被覆盖
    }
}
