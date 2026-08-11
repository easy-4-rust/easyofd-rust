//! 流式布局分析器。
//!
//! 将 Segment 序列分配到虚拟页面，自动执行垂直方向的分页。
//! 每个 Segment 被视为一个"行"，按顺序排列；当累计高度超过页面可用
//! 书写高度时，自动换页。Div 的 `x` / `y` 坐标在此阶段被计算并填充。
//! 对应 Java: ofdrw-layout `StreamingLayoutAnalyzer`。

use crate::div::Div;
use crate::segment_engine::Segment;

/// 虚拟页面：已定位的 Div 集合。
///
/// 对应 Java: ofdrw-layout `VirtualPage`。
#[derive(Debug, Clone)]
pub struct VirtualPage {
    /// 页面上的 Div 列表，`x` / `y` 已被引擎设置。
    pub divs: Vec<Div>,
    /// 页面宽度，单位 mm。
    pub page_width: f64,
    /// 页面高度，单位 mm。
    pub page_height: f64,
}

impl VirtualPage {
    /// 创建空虚拟页面。
    fn new(page_width: f64, page_height: f64) -> Self {
        Self {
            divs: Vec::new(),
            page_width,
            page_height,
        }
    }
}

/// 流式布局分析器。
///
/// 将 Segment 序列（由 [`SegmentationEngine`] 产生）分配到虚拟页面。
/// 每个段按顺序放置，当累计高度超过可用书写高度时换页。
/// 对应 Java: ofdrw-layout `StreamingLayoutAnalyzer`。
#[derive(Debug, Clone)]
pub struct StreamingLayoutAnalyzer {
    /// 页面宽度，单位 mm。
    pub page_width: f64,
    /// 页面高度，单位 mm。
    pub page_height: f64,
    /// 页边距，单位 mm。可用书写高度 = page_height - 2 * page_margin。
    pub page_margin: f64,
}

impl StreamingLayoutAnalyzer {
    /// 创建流式布局分析器。
    ///
    /// # 参数
    /// - `page_width`：页面宽度（mm）。
    /// - `page_height`：页面高度（mm）。
    /// - `page_margin`：页边距（mm）。
    #[must_use]
    pub fn new(page_width: f64, page_height: f64, page_margin: f64) -> Self {
        Self {
            page_width,
            page_height,
            page_margin,
        }
    }

    /// 将 Segment 序列分配到虚拟页面并设置 Div 坐标。
    ///
    /// 算法：
    /// 1. 从页顶开始，逐段放置。
    /// 2. 每段的高度加上行间距（段间额外空白 `0.5mm`）累加到当前 Y 光标。
    /// 3. 如果当前段放入后 Y 超过可用书写高度，将该段推到下一页。
    /// 4. Div 的 `x` 坐标根据段内左对齐（距页边距居中）计算，`y` 坐标由 Y 光标决定。
    #[must_use]
    pub fn analyze(&self, segments: Vec<Segment>) -> Vec<VirtualPage> {
        let usable_height = (self.page_height - 2.0 * self.page_margin).max(0.0);
        let usable_width = (self.page_width - 2.0 * self.page_margin).max(0.0);

        if usable_height <= 0.0 || usable_width <= 0.0 {
            return Vec::new();
        }

        let line_gap = 0.5_f64; // 段间垂直间距（mm）

        let mut pages: Vec<VirtualPage> = Vec::new();
        let mut current_page = VirtualPage::new(self.page_width, self.page_height);
        let mut y_cursor = self.page_margin; // 当前页内的 Y 光标

        for segment in segments {
            let seg_count = segment.divs.len();
            if seg_count == 0 {
                continue;
            }

            let segment_total = if current_page.divs.is_empty() {
                // 页面第一个段，不需要行间距
                segment.height
            } else {
                segment.height + line_gap
            };

            // 如果当前段放入后会超出可用高度，且当前页已有内容，则换页。
            if y_cursor + segment_total > self.page_margin + usable_height
                && !current_page.divs.is_empty()
            {
                pages.push(current_page);
                current_page = VirtualPage::new(self.page_width, self.page_height);
                y_cursor = self.page_margin;
            }

            // 如果不是页面第一个段，加行间距。
            if !current_page.divs.is_empty() {
                y_cursor += line_gap;
            }

            // 水平方向：段在可用宽度内左对齐（从页边距开始）。
            let offset_x = self.page_margin;

            // 将段内 Div 设置坐标后放入当前页。
            let mut x_acc = offset_x;
            for mut div in segment.divs {
                div.x = x_acc;
                div.y = y_cursor;
                x_acc += div.margin_box_width();
                current_page.divs.push(div);
            }

            y_cursor += segment.height;
        }

        if !current_page.divs.is_empty() {
            pages.push(current_page);
        }

        pages
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::segment_engine::Segment;
    use easyofd_core::TextObject;

    /// 辅助：创建指定尺寸的文本 Div。
    fn text_div(text: &str, w: f64, h: f64) -> Div {
        let mut d = Div::from_text_object(&TextObject::new(0.0, 0.0, text));
        d.width = w;
        d.height = h;
        d
    }

    /// 辅助：创建包含给定 Div 的 Segment。
    fn make_segment(divs: Vec<Div>) -> Segment {
        let width: f64 = divs.iter().map(|d| d.margin_box_width()).sum();
        let height: f64 = divs
            .iter()
            .map(|d| d.margin_box_height())
            .fold(0.0_f64, f64::max);
        Segment {
            divs,
            width,
            height,
        }
    }

    // --- 基本分析 ---

    #[test]
    fn single_segment_single_page() {
        let analyzer = StreamingLayoutAnalyzer::new(210.0, 297.0, 15.0);
        let segments = vec![make_segment(vec![text_div("a", 50.0, 10.0)])];
        let pages = analyzer.analyze(segments);
        assert_eq!(pages.len(), 1);
        assert_eq!(pages[0].divs.len(), 1);
    }

    // --- 自动分页 ---

    #[test]
    fn overflow_triggers_new_page() {
        let analyzer = StreamingLayoutAnalyzer::new(210.0, 297.0, 15.0);
        // 可用高度 = 297 - 30 = 267mm
        // 每个段高度 100mm，放 3 个需要 2 页
        let mut segments = Vec::new();
        for i in 0..3 {
            segments.push(make_segment(vec![text_div(&format!("s{i}"), 50.0, 100.0)]));
        }
        let pages = analyzer.analyze(segments);
        assert_eq!(pages.len(), 2);
    }

    // --- Div 坐标设置 ---

    #[test]
    fn div_coordinates_are_set() {
        let analyzer = StreamingLayoutAnalyzer::new(210.0, 297.0, 15.0);
        let segments = vec![make_segment(vec![text_div("hello", 50.0, 10.0)])];
        let pages = analyzer.analyze(segments);
        let div = &pages[0].divs[0];
        // x 应在页边距
        assert!((div.x - 15.0).abs() < f64::EPSILON);
        // y 应在页边距
        assert!((div.y - 15.0).abs() < f64::EPSILON);
    }

    // --- 空输入 ---

    #[test]
    fn empty_segments_returns_empty_pages() {
        let analyzer = StreamingLayoutAnalyzer::new(210.0, 297.0, 15.0);
        let pages = analyzer.analyze(Vec::new());
        assert!(pages.is_empty());
    }

    // --- 多段同页 ---

    #[test]
    fn multiple_segments_on_same_page() {
        let analyzer = StreamingLayoutAnalyzer::new(210.0, 297.0, 15.0);
        let segments = vec![
            make_segment(vec![text_div("a", 50.0, 10.0)]),
            make_segment(vec![text_div("b", 50.0, 10.0)]),
        ];
        let pages = analyzer.analyze(segments);
        assert_eq!(pages.len(), 1);
        assert_eq!(pages[0].divs.len(), 2);
    }

    // --- 页面尺寸正确传递 ---

    #[test]
    fn virtual_page_has_correct_dimensions() {
        let analyzer = StreamingLayoutAnalyzer::new(210.0, 297.0, 15.0);
        let segments = vec![make_segment(vec![text_div("x", 30.0, 5.0)])];
        let pages = analyzer.analyze(segments);
        assert!((pages[0].page_width - 210.0).abs() < f64::EPSILON);
        assert!((pages[0].page_height - 297.0).abs() < f64::EPSILON);
    }

    // --- 超大 margin 导致可用区域为零 ---

    #[test]
    fn excessive_margin_returns_empty() {
        let analyzer = StreamingLayoutAnalyzer::new(100.0, 100.0, 60.0);
        let segments = vec![make_segment(vec![text_div("x", 10.0, 10.0)])];
        let pages = analyzer.analyze(segments);
        assert!(pages.is_empty());
    }
}
