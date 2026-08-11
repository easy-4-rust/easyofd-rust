//! 艺术字元素。
//!
//! 对应 Java: org.ofdrw.layout.element.canvas.ArtWord
//!
//! 在 Span 元素效果基础上增加艺术字常见效果：斜体、文字左右拉伸、垂直拉伸。

use crate::canvas_base::CanvasBase;
use crate::paragraph::TextAlign;

/// 艺术字元素。
///
/// 对应 Java: ofdrw layout canvas ArtWord。
#[derive(Debug, Clone)]
pub struct ArtWord {
    /// 画布基类属性。
    pub canvas: CanvasBase,
    /// 字体名称。
    pub font_name: Option<String>,
    /// 字号（mm），默认 3。
    pub font_size: f64,
    /// 字间距（mm），默认 0。
    pub letter_spacing: f64,
    /// 是否加粗。
    pub bold: bool,
    /// 粗细值（100-900）。
    pub weight: Option<u32>,
    /// 是否斜体。
    pub italic: bool,
    /// 是否含下划线。
    pub underline: bool,
    /// 下划线与文字的偏移量（mm），默认 1.2。
    pub underline_offset: f64,
    /// 下划线宽度（mm），0 表示保持默认。
    pub underline_width: f64,
    /// 文本内容。
    pub text: String,
    /// 字体颜色 `[r, g, b]`。
    pub color: [u8; 3],
    /// 文本对齐方式。
    pub text_align: Option<TextAlign>,
    /// 水平缩放比例（1.0 = 正常）。
    pub horizontal_scaling: f64,
    /// 垂直缩放比例（1.0 = 正常）。
    pub vertical_scaling: f64,
    /// 水平倾斜程度。
    pub horizontal_inclination: f64,
    /// 垂直倾斜程度。
    pub vertical_inclination: f64,
    /// 水平偏移量（mm）。
    pub offset_x: f64,
    /// 垂直偏移量（mm）。
    pub offset_y: f64,
}

impl ArtWord {
    /// 创建艺术字元素（对应 Java: ArtWord(width, height)）。
    #[must_use]
    pub fn new(width: f64, height: f64) -> Self {
        Self {
            canvas: CanvasBase::new(width, height),
            font_name: None,
            font_size: 3.0,
            letter_spacing: 0.0,
            bold: false,
            weight: None,
            italic: false,
            underline: false,
            underline_offset: 1.2,
            underline_width: 0.0,
            text: String::new(),
            color: [0, 0, 0],
            text_align: None,
            horizontal_scaling: 1.0,
            vertical_scaling: 1.0,
            horizontal_inclination: 0.0,
            vertical_inclination: 0.0,
            offset_x: 0.0,
            offset_y: 0.0,
        }
    }

    /// 设置字体名称。
    #[must_use]
    pub fn font_name(mut self, name: impl Into<String>) -> Self {
        self.font_name = Some(name.into());
        self
    }

    /// 设置字号。
    #[must_use]
    pub fn font_size(mut self, size: f64) -> Self {
        self.font_size = size;
        self
    }

    /// 设置字间距。
    #[must_use]
    pub fn letter_spacing(mut self, spacing: f64) -> Self {
        self.letter_spacing = spacing;
        self
    }

    /// 设置加粗。
    #[must_use]
    pub fn bold(mut self, bold: bool) -> Self {
        self.bold = bold;
        self
    }

    /// 设置斜体。
    #[must_use]
    pub fn italic(mut self, italic: bool) -> Self {
        self.italic = italic;
        self
    }

    /// 设置文本内容。
    #[must_use]
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self
    }

    /// 设置颜色。
    #[must_use]
    pub fn color(mut self, r: u8, g: u8, b: u8) -> Self {
        self.color = [r, g, b];
        self
    }

    /// 设置水平缩放。
    #[must_use]
    pub fn horizontal_scaling(mut self, scale: f64) -> Self {
        self.horizontal_scaling = scale;
        self
    }

    /// 设置垂直缩放。
    #[must_use]
    pub fn vertical_scaling(mut self, scale: f64) -> Self {
        self.vertical_scaling = scale;
        self
    }

    /// 设置下划线。
    #[must_use]
    pub fn underline(mut self, underline: bool) -> Self {
        self.underline = underline;
        self
    }

    /// 字符数量。
    #[must_use]
    pub fn length(&self) -> usize {
        self.text.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let aw = ArtWord::new(100.0, 50.0);
        assert!((aw.canvas.width - 100.0).abs() < f64::EPSILON);
        assert!((aw.font_size - 3.0).abs() < f64::EPSILON);
        assert_eq!(aw.color, [0, 0, 0]);
    }

    #[test]
    fn test_builders() {
        let aw = ArtWord::new(100.0, 50.0)
            .font_name("SimSun")
            .font_size(12.0)
            .letter_spacing(1.0)
            .bold(true)
            .italic(true)
            .text("Hello")
            .color(255, 0, 0)
            .horizontal_scaling(1.5)
            .vertical_scaling(0.8)
            .underline(true);
        assert_eq!(aw.font_name.as_deref(), Some("SimSun"));
        assert!((aw.font_size - 12.0).abs() < f64::EPSILON);
        assert!(aw.bold);
        assert!(aw.italic);
        assert_eq!(aw.text, "Hello");
        assert_eq!(aw.color, [255, 0, 0]);
        assert!((aw.horizontal_scaling - 1.5).abs() < f64::EPSILON);
        assert!(aw.underline);
    }

    #[test]
    fn test_length() {
        let aw = ArtWord::new(100.0, 50.0).text("test");
        assert_eq!(aw.length(), 4);
    }
}
