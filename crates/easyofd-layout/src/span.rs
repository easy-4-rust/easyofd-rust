//! 文本块（Span）。
//!
//! 对应 Java: org.ofdrw.layout.element.Span

use easyofd_core::Weight;

/// 文本块元素（ofdrw layout Span）。
///
/// 对应 Java: ofdrw Span，实现 TextFontInfo 接口的文本片段。
// 多个布尔标志对应 ofdrw Span 的文本样式开关（bold/italic/underline/fill/linebreak）。
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    /// 文本内容。
    pub text: String,
    /// 字体名称（可选）。
    pub font: Option<String>,
    /// 字号（默认 3.0）。
    pub font_size: f64,
    /// 字间距（默认 0）。
    pub letter_spacing: f64,
    /// 是否加粗。
    pub bold: bool,
    /// 粗细值（可选）。
    pub weight: Option<Weight>,
    /// 是否斜体。
    pub italic: bool,
    /// 是否下划线。
    pub underline: bool,
    /// 下划线偏移（默认 1.2）。
    pub underline_offset: f64,
    /// 下划线宽度（默认 0，按字号比例）。
    pub underline_width: f64,
    /// 是否填充（默认 true）。
    pub fill: bool,
    /// 填充颜色（RGB，可选）。
    pub fill_color: Option<[u8; 3]>,
    /// 是否换行。
    pub linebreak: bool,
}

impl Span {
    /// 创建文本块（对应 Java: Span(String)）。
    #[must_use]
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            font: None,
            font_size: 3.0,
            letter_spacing: 0.0,
            bold: false,
            weight: None,
            italic: false,
            underline: false,
            underline_offset: 1.2,
            underline_width: 0.0,
            fill: true,
            fill_color: None,
            linebreak: false,
        }
    }

    /// 设置字体。
    #[must_use]
    pub fn font(mut self, font: impl Into<String>) -> Self {
        self.font = Some(font.into());
        self
    }

    /// 设置字号（对应 Java: Span#setFontSize）。
    #[must_use]
    pub fn font_size(mut self, size: f64) -> Self {
        self.font_size = size;
        self
    }

    /// 设置字间距（对应 Java: Span#setLetterSpacing）。
    #[must_use]
    pub fn letter_spacing(mut self, spacing: f64) -> Self {
        self.letter_spacing = spacing;
        self
    }

    /// 设置加粗（对应 Java: Span#setBold）。
    #[must_use]
    pub fn bold(mut self, bold: bool) -> Self {
        self.bold = bold;
        if bold && self.weight.is_none() {
            self.weight = Some(Weight::W700);
        }
        self
    }

    /// 设置斜体（对应 Java: Span#setItalic）。
    #[must_use]
    pub fn italic(mut self, italic: bool) -> Self {
        self.italic = italic;
        self
    }

    /// 设置下划线（对应 Java: Span#setUnderline）。
    #[must_use]
    pub fn underline(mut self, underline: bool) -> Self {
        self.underline = underline;
        self
    }

    /// 设置填充颜色（RGB，对应 Java: Span#setColor(int r, int g, int b)）。
    #[must_use]
    pub fn color(mut self, r: u8, g: u8, b: u8) -> Self {
        self.fill_color = Some([r, g, b]);
        self
    }

    /// 设置是否换行。
    #[must_use]
    pub fn linebreak(mut self, linebreak: bool) -> Self {
        self.linebreak = linebreak;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_span_new() {
        let s = Span::new("hello");
        assert_eq!(s.text, "hello");
        assert!((s.font_size - 3.0).abs() < f64::EPSILON);
        assert!(!s.bold);
        assert!(s.fill);
    }

    #[test]
    fn test_builders() {
        let s = Span::new("x")
            .font("SimSun")
            .font_size(12.0)
            .letter_spacing(1.0)
            .bold(true)
            .italic(true)
            .underline(true)
            .color(255, 0, 0);
        assert_eq!(s.font.as_deref(), Some("SimSun"));
        assert!((s.font_size - 12.0).abs() < f64::EPSILON);
        assert!(s.bold);
        assert!(s.italic);
        assert!(s.underline);
        assert_eq!(s.fill_color, Some([255, 0, 0]));
        // bold 自动设置字重
        assert_eq!(s.weight, Some(Weight::W700));
    }

    #[test]
    fn test_clone_partial_eq() {
        let a = Span::new("a").font_size(10.0);
        let b = a.clone();
        assert_eq!(a, b);
    }
}
