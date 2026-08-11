//! 段落。
//!
//! 对应 Java: org.ofdrw.layout.element.Paragraph

use crate::span::Span;

/// 文本对齐方式（对应 Java: ofdrw TextAlign）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextAlign {
    /// 左对齐。
    #[default]
    Left,
    /// 居中对齐。
    Center,
    /// 右对齐。
    Right,
    /// 两端对齐。
    Justify,
}

/// 段落（ofdrw layout Paragraph，Div 的文本变体）。
///
/// 对应 Java: ofdrw Paragraph。包含首行缩进、行距与文本块序列。
#[derive(Debug, Clone, PartialEq)]
pub struct Paragraph {
    /// 宽度（mm）。
    pub width: Option<f64>,
    /// 高度（mm）。
    pub height: Option<f64>,
    /// 首行缩进（字符数，可选）。
    pub first_line_indent: Option<u32>,
    /// 首行缩进宽度（mm，可选）。
    pub first_line_indent_width: Option<f64>,
    /// 行距（默认 2.0）。
    pub line_space: f64,
    /// 文本块序列。
    pub contents: Vec<Span>,
    /// 文本对齐方式。
    pub text_align: TextAlign,
}

impl Default for Paragraph {
    fn default() -> Self {
        Self {
            width: None,
            height: None,
            first_line_indent: None,
            first_line_indent_width: None,
            line_space: 2.0,
            contents: Vec::new(),
            text_align: TextAlign::Left,
        }
    }
}

impl Paragraph {
    /// 创建空段落（对应 Java: Paragraph()）。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 创建指定尺寸的段落（对应 Java: Paragraph(width, height)）。
    #[must_use]
    pub fn with_size(width: f64, height: f64) -> Self {
        Self {
            width: Some(width),
            height: Some(height),
            ..Self::default()
        }
    }

    /// 设置首行缩进（字符数，对应 Java: Paragraph#setFirstLineIndent）。
    #[must_use]
    pub fn first_line_indent(mut self, indent: u32) -> Self {
        self.first_line_indent = Some(indent);
        self
    }

    /// 设置行距（对应 Java: Paragraph#setLineSpace）。
    #[must_use]
    pub fn line_space(mut self, space: f64) -> Self {
        self.line_space = space;
        self
    }

    /// 设置文本对齐（对应 Java: Paragraph#setTextAlign）。
    #[must_use]
    pub fn text_align(mut self, align: TextAlign) -> Self {
        self.text_align = align;
        self
    }

    /// 追加文本块（对应 Java: Paragraph#add）。
    #[must_use]
    #[allow(clippy::should_implement_trait)]
    pub fn add(mut self, span: Span) -> Self {
        self.contents.push(span);
        self
    }

    /// 追加多个文本块。
    pub fn add_all(&mut self, spans: Vec<Span>) {
        self.contents.extend(spans);
    }

    /// 段落文本内容拼接。
    #[must_use]
    pub fn text(&self) -> String {
        self.contents
            .iter()
            .map(|s| s.text.as_str())
            .collect::<Vec<_>>()
            .join("")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paragraph_default() {
        let p = Paragraph::new();
        assert!((p.line_space - 2.0).abs() < f64::EPSILON);
        assert!(p.contents.is_empty());
        assert_eq!(p.text_align, TextAlign::Left);
    }

    #[test]
    fn test_paragraph_build() {
        let p = Paragraph::new()
            .first_line_indent(2)
            .line_space(1.5)
            .text_align(TextAlign::Center)
            .add(Span::new("你好"))
            .add(Span::new("世界"));
        assert_eq!(p.first_line_indent, Some(2));
        assert_eq!(p.text_align, TextAlign::Center);
        assert_eq!(p.contents.len(), 2);
        assert_eq!(p.text(), "你好世界");
    }

    #[test]
    fn test_with_size_and_add_all() {
        let mut p = Paragraph::with_size(100.0, 50.0);
        assert_eq!(p.width, Some(100.0));
        p.add_all(vec![Span::new("a"), Span::new("b")]);
        assert_eq!(p.contents.len(), 2);
        assert_eq!(p.text(), "ab");
    }
}
