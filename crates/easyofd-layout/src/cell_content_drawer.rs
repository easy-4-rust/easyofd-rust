//! 单元格内容绘制器配置。
//!
//! 对应 Java: org.ofdrw.layout.element.canvas.CellContentDrawer
//!
//! 提供单元格内文字和图片的绘制配置，包括字体、颜色、对齐方式等。

use crate::paragraph::TextAlign;
use crate::vertical_align::VerticalAlign;

/// 单元格内图片信息。
#[derive(Debug, Clone, PartialEq)]
pub struct CellImage {
    /// 图片路径标识。
    pub path: String,
    /// 图片宽度（mm）。
    pub width: f64,
    /// 图片高度（mm）。
    pub height: f64,
}

impl CellImage {
    /// 创建图片信息。
    #[must_use]
    pub fn new(path: impl Into<String>, width: f64, height: f64) -> Self {
        Self {
            path: path.into(),
            width,
            height,
        }
    }
}

/// 单元格内容绘制器配置。
///
/// 对应 Java: ofdrw layout canvas CellContentDrawer。
///
/// 提供单元格内文字和图片的全部绘制属性。
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, PartialEq)]
pub struct CellContentDrawer {
    /// 单元格文字内容。
    pub value: Option<String>,
    /// 文字水平对齐方式（默认左对齐）。
    pub text_align: TextAlign,
    /// 文字垂直对齐方式（默认居中）。
    pub vertical_align: VerticalAlign,
    /// 文字颜色（默认 `#000000`）。
    pub color: String,
    /// 字体名称（默认宋体）。
    pub font_name: String,
    /// 字号（mm），默认 3。
    pub font_size: f64,
    /// 行间距（mm），默认 0.6。
    pub line_space: f64,
    /// 是否加粗。
    pub bold: bool,
    /// 字体粗细（CSS3 标准），默认 `"normal"`。
    pub font_weight: String,
    /// 是否斜体。
    pub italic: bool,
    /// 字间距（mm），默认 0。
    pub letter_spacing: f64,
    /// 图片（可选）。
    pub img: Option<CellImage>,
    /// 是否有下划线。
    pub underline: bool,
    /// 是否有删除线。
    pub delete_line: bool,
    /// 外部字体路径（可选）。
    pub ext_font_path: Option<String>,
}

impl Default for CellContentDrawer {
    fn default() -> Self {
        Self {
            value: None,
            text_align: TextAlign::Left,
            vertical_align: VerticalAlign::Center,
            color: "#000000".to_owned(),
            font_name: "宋体".to_owned(),
            font_size: 3.0,
            line_space: 0.6,
            bold: false,
            font_weight: "normal".to_owned(),
            italic: false,
            letter_spacing: 0.0,
            img: None,
            underline: false,
            delete_line: false,
            ext_font_path: None,
        }
    }
}

impl CellContentDrawer {
    /// 创建默认单元格内容绘制器。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置文字内容。
    #[must_use]
    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = Some(value.into());
        self
    }

    /// 设置图片内容。
    #[must_use]
    pub fn image(mut self, path: impl Into<String>, width: f64, height: f64) -> Self {
        self.img = Some(CellImage::new(path, width, height));
        self
    }

    /// 设置颜色（对应 Java: CellContentDrawer#setColor）。
    pub fn set_color(&mut self, color: impl Into<String>) -> Result<(), &'static str> {
        let c = color.into();
        if c.is_empty() {
            return Err("颜色(color)不能为空");
        }
        self.color = c;
        Ok(())
    }

    /// 设置字体大小（对应 Java: CellContentDrawer#setFontSize）。
    pub fn set_font_size(&mut self, size: f64) -> Result<(), &'static str> {
        if size <= 0.0 {
            return Err("字号(fontSize)必须大于0");
        }
        self.font_size = size;
        Ok(())
    }

    /// 设置水平对齐。
    #[must_use]
    pub fn text_align(mut self, align: TextAlign) -> Self {
        self.text_align = align;
        self
    }

    /// 设置垂直对齐。
    #[must_use]
    pub fn vertical_align(mut self, align: VerticalAlign) -> Self {
        self.vertical_align = align;
        self
    }

    /// 设置字体名称。
    #[must_use]
    pub fn font_name(mut self, name: impl Into<String>) -> Self {
        self.font_name = name.into();
        self
    }

    /// 设置行间距。
    #[must_use]
    pub fn line_space(mut self, space: f64) -> Self {
        self.line_space = space;
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

    /// 设置字间距。
    #[must_use]
    pub fn letter_spacing(mut self, spacing: f64) -> Self {
        self.letter_spacing = spacing;
        self
    }

    /// 设置下划线。
    #[must_use]
    pub fn underline(mut self, underline: bool) -> Self {
        self.underline = underline;
        self
    }

    /// 设置删除线。
    #[must_use]
    pub fn delete_line(mut self, delete_line: bool) -> Self {
        self.delete_line = delete_line;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let drawer = CellContentDrawer::new();
        assert!(drawer.value.is_none());
        assert_eq!(drawer.text_align, TextAlign::Left);
        assert_eq!(drawer.vertical_align, VerticalAlign::Center);
        assert_eq!(drawer.color, "#000000");
        assert_eq!(drawer.font_name, "宋体");
        assert!((drawer.font_size - 3.0).abs() < f64::EPSILON);
        assert!((drawer.line_space - 0.6).abs() < f64::EPSILON);
    }

    #[test]
    fn test_builders() {
        let drawer = CellContentDrawer::new()
            .value("Hello")
            .text_align(TextAlign::Center)
            .vertical_align(VerticalAlign::Top)
            .font_name("SimHei")
            .line_space(1.0)
            .bold(true)
            .italic(true)
            .underline(true)
            .delete_line(true);
        assert_eq!(drawer.value.as_deref(), Some("Hello"));
        assert_eq!(drawer.text_align, TextAlign::Center);
        assert_eq!(drawer.vertical_align, VerticalAlign::Top);
        assert!(drawer.bold);
        assert!(drawer.italic);
        assert!(drawer.underline);
        assert!(drawer.delete_line);
    }

    #[test]
    fn test_set_color_ok() {
        let mut drawer = CellContentDrawer::new();
        assert!(drawer.set_color("#FF0000").is_ok());
        assert_eq!(drawer.color, "#FF0000");
    }

    #[test]
    fn test_set_color_empty() {
        let mut drawer = CellContentDrawer::new();
        assert!(drawer.set_color("").is_err());
    }

    #[test]
    fn test_set_font_size_ok() {
        let mut drawer = CellContentDrawer::new();
        assert!(drawer.set_font_size(5.0).is_ok());
        assert!((drawer.font_size - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_set_font_size_invalid() {
        let mut drawer = CellContentDrawer::new();
        assert!(drawer.set_font_size(0.0).is_err());
        assert!(drawer.set_font_size(-1.0).is_err());
    }

    #[test]
    fn test_image() {
        let drawer = CellContentDrawer::new().image("/path/to/img.png", 10.0, 10.0);
        assert!(drawer.img.is_some());
        let img = drawer.img.unwrap();
        assert_eq!(img.path, "/path/to/img.png");
        assert!((img.width - 10.0).abs() < f64::EPSILON);
    }
}
