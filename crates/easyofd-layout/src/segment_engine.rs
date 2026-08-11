//! 流式版面分段引擎。
//!
//! 将 Div 队列按页面可书写宽度进行水平分段，同一段内的 Div 可以排在同一行。
//! 对应 Java: ofdrw-layout `SegmentationEngine`。

use crate::Div;

/// 一个段：同一行内可容纳的 Div 集合。
///
/// 对应 Java: ofdrw-layout `Segment`。
#[derive(Debug, Clone)]
pub struct Segment {
    /// 段内包含的 Div 列表。
    pub divs: Vec<Div>,
    /// 段总宽度（含各 Div 外边距），单位 mm。
    pub width: f64,
    /// 段高度，取所有 Div 中最高的外边距盒高度，单位 mm。
    pub height: f64,
}

impl Segment {
    /// 创建空段。
    fn new() -> Self {
        Self {
            divs: Vec::new(),
            width: 0.0,
            height: 0.0,
        }
    }

    /// 判断 Div 是否能放入本段（不溢出可用宽度）。
    fn can_fit(&self, div: &Div, available_width: f64) -> bool {
        if self.divs.is_empty() {
            return true; // 空段总是能容纳（超宽 Div 独占一段）
        }
        self.width + div.margin_box_width() <= available_width
    }

    /// 将 Div 追加到本段（调用前应确保 [`can_fit`] 返回 `true`）。
    fn push(&mut self, div: Div) {
        let div_w = div.margin_box_width();
        if self.divs.is_empty() {
            self.width = div_w;
        } else {
            self.width += div_w;
        }
        self.height = self.height.max(div.margin_box_height());
        self.divs.push(div);
    }
}

/// 分段引擎。
///
/// 将 Div 队列按页面宽度分段。每段代表水平方向上可以排在同一行的 Div 集合。
/// 对应 Java: ofdrw-layout `SegmentationEngine`。
#[derive(Debug, Clone)]
pub struct SegmentationEngine {
    /// 页面宽度，单位 mm。
    pub page_width: f64,
    /// 页边距，单位 mm。可用书写宽度 = page_width - 2 * page_margin。
    pub page_margin: f64,
}

impl SegmentationEngine {
    /// 创建分段引擎。
    ///
    /// # 参数
    /// - `page_width`：页面宽度（mm）。
    /// - `page_margin`：页边距（mm）。
    #[must_use]
    pub fn new(page_width: f64, page_margin: f64) -> Self {
        Self {
            page_width,
            page_margin,
        }
    }

    /// 对 Div 列表进行分段。
    ///
    /// 每个 Div 按其 `margin_box_width()` 计算占用宽度。如果单个 Div 超过可用
    /// 书写宽度，它将独占一个段（不被截断）。
    #[must_use]
    pub fn process(&self, divs: Vec<Div>) -> Vec<Segment> {
        let available = (self.page_width - 2.0 * self.page_margin).max(0.0);
        if available <= 0.0 || divs.is_empty() {
            return Vec::new();
        }

        let mut segments: Vec<Segment> = Vec::new();
        let mut current = Segment::new();

        for div in divs {
            if current.can_fit(&div, available) {
                current.push(div);
            } else {
                // 当前段已满，先保存，再用新段接收。
                segments.push(current);
                current = Segment::new();
                current.push(div);
            }
        }
        if !current.divs.is_empty() {
            segments.push(current);
        }

        segments
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use easyofd_core::TextObject;

    /// 辅助：创建指定宽度（mm）的文本 Div。
    fn text_div(text: &str, w: f64) -> Div {
        let mut d = Div::from_text_object(&TextObject::new(0.0, 0.0, text));
        d.width = w;
        d
    }

    // --- 基本分段 ---

    #[test]
    fn single_div_fits_in_one_segment() {
        let engine = SegmentationEngine::new(210.0, 15.0);
        let divs = vec![text_div("hello", 50.0)];
        let segments = engine.process(divs);
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].divs.len(), 1);
    }

    // --- 多 Div 同段 ---

    #[test]
    fn multiple_divs_fit_in_single_segment() {
        let engine = SegmentationEngine::new(210.0, 15.0);
        // 可用宽度 = 210 - 30 = 180mm
        let divs = vec![
            text_div("a", 50.0),
            text_div("b", 60.0),
            text_div("c", 40.0),
        ];
        let segments = engine.process(divs);
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].divs.len(), 3);
    }

    // --- 溢出分段 ---

    #[test]
    fn overflow_creates_new_segment() {
        let engine = SegmentationEngine::new(210.0, 15.0);
        // 可用宽度 180mm
        let divs = vec![
            text_div("a", 100.0),
            text_div("b", 100.0), // 100+100 > 180 => 分段
            text_div("c", 50.0),
        ];
        let segments = engine.process(divs);
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].divs.len(), 1);
        assert_eq!(segments[1].divs.len(), 2);
    }

    // --- 超宽 Div 独占一段 ---

    #[test]
    fn oversized_div_gets_own_segment() {
        let engine = SegmentationEngine::new(210.0, 15.0);
        // 可用宽度 180mm，Div 宽 200mm
        let divs = vec![text_div("wide", 200.0)];
        let segments = engine.process(divs);
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].divs.len(), 1);
    }

    // --- 空输入 ---

    #[test]
    fn empty_input_returns_empty() {
        let engine = SegmentationEngine::new(210.0, 15.0);
        let segments = engine.process(Vec::new());
        assert!(segments.is_empty());
    }

    // --- margin 为零 ---

    #[test]
    fn zero_margin_uses_full_width() {
        let engine = SegmentationEngine::new(100.0, 0.0);
        let divs = vec![text_div("a", 90.0), text_div("b", 5.0)];
        let segments = engine.process(divs);
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].divs.len(), 2);
    }

    // --- 段高度计算 ---

    #[test]
    fn segment_height_is_max_of_divs() {
        let engine = SegmentationEngine::new(210.0, 15.0);
        let mut d1 = text_div("a", 50.0);
        d1.height = 10.0;
        let mut d2 = text_div("b", 50.0);
        d2.height = 25.0;
        let segments = engine.process(vec![d1, d2]);
        assert_eq!(segments.len(), 1);
        assert!((segments[0].height - 25.0).abs() < f64::EPSILON);
    }

    // --- 超大 margin 导致可用宽度为零 ---

    #[test]
    fn excessive_margin_returns_empty() {
        let engine = SegmentationEngine::new(100.0, 60.0);
        // 可用宽度 = 100 - 120 = -20 => 0
        let divs = vec![text_div("x", 10.0)];
        let segments = engine.process(divs);
        assert!(segments.is_empty());
    }
}
