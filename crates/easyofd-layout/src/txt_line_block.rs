//! 行内文字块。
//!
//! 对应 Java: org.ofdrw.layout.element.TxtLineBlock

use crate::paragraph::TextAlign;
use crate::span::Span;
use crate::txt_glyph::TxtGlyph;

/// 行内文字块，管理一行内所有 Span 的布局。
///
/// 对应 Java: ofdrw layout TxtLineBlock。
#[derive(Debug, Clone)]
pub struct TxtLineBlock {
    /// 行内字体浮动方向（默认左浮动）。
    pub text_align: TextAlign,
    /// 行内文字单元列表。
    pub inline_spans: Vec<Span>,
    /// 最高的文字高度（mm）。
    pub max_span_height: f64,
    /// 行宽度（mm）。
    pub width: f64,
    /// 行间距（mm）。
    pub line_space: f64,
    /// 行内可用最大宽度（mm）。
    pub line_max_available_width: f64,
}

impl TxtLineBlock {
    /// 创建行块（对应 Java: TxtLineBlock(lineMaxAvailableWidth, lineSpace)）。
    #[must_use]
    pub fn new(line_max_available_width: f64, line_space: f64) -> Self {
        Self::with_text_align(line_max_available_width, line_space, TextAlign::Left)
    }

    /// 创建带对齐方式的行块（对应 Java: TxtLineBlock(lineMaxAvailableWidth, lineSpace, textAlign)）。
    #[must_use]
    pub fn with_text_align(
        line_max_available_width: f64,
        line_space: f64,
        text_align: TextAlign,
    ) -> Self {
        Self {
            text_align,
            inline_spans: Vec::new(),
            max_span_height: 0.0,
            width: 0.0,
            line_space,
            line_max_available_width,
        }
    }

    /// 尝试向行中加入文字单元（对应 Java: TxtLineBlock#tryAdd）。
    ///
    /// 返回 `true` 表示空间足够已加入，`false` 表示空间不足无法加入。
    pub fn try_add(&mut self, span_width: f64, span_height: f64, span: Span) -> bool {
        if span_width + self.width > self.line_max_available_width {
            return false;
        }
        self.width += span_width;
        if span_height > self.max_span_height {
            self.max_span_height = span_height;
        }
        self.inline_spans.push(span);
        true
    }

    /// 是否为空行。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.inline_spans.is_empty()
    }

    /// 获取行所占据的高度（文字高度 + 行间距）。
    #[must_use]
    pub fn height(&self) -> f64 {
        self.max_span_height + self.line_space
    }

    /// 剩余可用宽度（mm）。
    #[must_use]
    pub fn remaining_width(&self) -> f64 {
        (self.line_max_available_width - self.width).max(0.0)
    }

    /// 尝试通过切分文字单元的方式加入行内。
    ///
    /// 返回切分后的剩余文字（glyph 列表），若无法切分则返回 `None`。
    #[must_use]
    pub fn try_split_add(&mut self, glyphs: &[TxtGlyph], span: Span) -> Option<Vec<TxtGlyph>> {
        let remain_width = self.remaining_width();
        let mut split_index = 0;
        let mut consumed_width = 0.0;

        for (i, glyph) in glyphs.iter().enumerate() {
            let glyph_w = glyph.w();
            if consumed_width + glyph_w <= remain_width {
                consumed_width += glyph_w;
                continue;
            }
            if i == 0 {
                // 一个字符都无法加入
                return None;
            }
            split_index = i;
            break;
        }

        // 前半段加入行中
        let span_width: f64 = glyphs[..split_index].iter().map(|g| g.w()).sum();
        let span_height: f64 = glyphs[..split_index]
            .iter()
            .map(|g| g.h())
            .fold(0.0_f64, f64::max);
        self.try_add(span_width, span_height, span);

        // 后半段返回
        Some(glyphs[split_index..].to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let block = TxtLineBlock::new(100.0, 1.0);
        assert!((block.line_max_available_width - 100.0).abs() < f64::EPSILON);
        assert!((block.line_space - 1.0).abs() < f64::EPSILON);
        assert!(block.is_empty());
    }

    #[test]
    fn test_try_add_success() {
        let mut block = TxtLineBlock::new(100.0, 1.0);
        let span = Span::new("hello");
        let added = block.try_add(50.0, 10.0, span);
        assert!(added);
        assert!(!block.is_empty());
        assert!((block.width - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_try_add_overflow() {
        let mut block = TxtLineBlock::new(100.0, 1.0);
        let span1 = Span::new("hello");
        assert!(block.try_add(60.0, 10.0, span1));
        let span2 = Span::new("world");
        assert!(!block.try_add(50.0, 10.0, span2)); // 60 + 50 > 100
    }

    #[test]
    fn test_height() {
        let mut block = TxtLineBlock::new(100.0, 1.5);
        let span = Span::new("test");
        block.try_add(30.0, 12.0, span);
        // height = max_span_height + line_space = 12.0 + 1.5 = 13.5
        assert!((block.height() - 13.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_remaining_width() {
        let mut block = TxtLineBlock::new(100.0, 1.0);
        block.try_add(40.0, 10.0, Span::new("a"));
        assert!((block.remaining_width() - 60.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_with_text_align() {
        let block = TxtLineBlock::with_text_align(100.0, 1.0, TextAlign::Center);
        assert_eq!(block.text_align, TextAlign::Center);
    }
}
