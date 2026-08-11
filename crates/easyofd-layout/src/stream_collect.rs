//! 流式收集器。
//!
//! 对应 Java: org.ofdrw.layout.StreamCollect
//!
//! 用于编辑文档时插入文档内容。内容超过一页时，分页并添加新的一页在原页面后面。

use crate::div::Div;
use crate::page_layout::PageLayout;
use crate::segment_engine::SegmentationEngine;
use crate::streaming_layout::{StreamingLayoutAnalyzer, VirtualPage};

/// 流式收集器，收集 Div 元素并通过布局引擎分析为虚拟页面。
///
/// 对应 Java: org.ofdrw.layout.StreamCollect
///
/// 收集器可指定目标页号（从 1 开始），分析后产出的虚拟页面将按此编号递增。
/// 若未指定页号，则由布局引擎默认处理。
#[derive(Debug, Clone)]
pub struct StreamCollect {
    /// 流式页面内容。
    content: Vec<Div>,
    /// 开始页面位置（从 1 开始），`None` 表示不指定。
    page_num: Option<u32>,
}

impl StreamCollect {
    /// 创建流式收集器（不指定目标页号）。
    ///
    /// 对应 Java: `StreamCollect()`
    #[must_use]
    pub fn new() -> Self {
        Self {
            content: Vec::new(),
            page_num: None,
        }
    }

    /// 创建流式收集器，指定目标页号。
    ///
    /// 对应 Java: `StreamCollect(Integer pageNum)`
    ///
    /// # Arguments
    ///
    /// * `page_num` - 页码（从 1 开始）。
    #[must_use]
    pub fn with_page_num(page_num: u32) -> Self {
        Self {
            content: Vec::new(),
            page_num: Some(page_num),
        }
    }

    /// 添加 Div 元素到收集器。
    ///
    /// 对应 Java: `StreamCollect.add(Div element)`
    ///
    /// # Arguments
    ///
    /// * `element` - 要添加的 Div 元素。
    pub fn add(&mut self, element: Div) -> &mut Self {
        self.content.push(element);
        self
    }

    /// 获取内容切片。
    ///
    /// 对应 Java: `StreamCollect.getContent()`
    #[must_use]
    pub fn content(&self) -> &[Div] {
        &self.content
    }

    /// 获取内容切片（别名，对齐 Java `getContent`）。
    ///
    /// 对应 Java: `StreamCollect.getContent()`
    #[must_use]
    pub fn get_content(&self) -> &[Div] {
        &self.content
    }

    /// 获取内容长度。
    #[must_use]
    pub fn len(&self) -> usize {
        self.content.len()
    }

    /// 内容是否为空。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }

    /// 获取目标页号。
    ///
    /// 对应 Java: `StreamCollect.getPageNum()`
    #[must_use]
    pub fn page_num(&self) -> Option<u32> {
        self.page_num
    }

    /// 设置目标页号。
    ///
    /// 对应 Java: `StreamCollect.setPageNum(Integer pageNum)`
    pub fn set_page_num(&mut self, page_num: Option<u32>) -> &mut Self {
        self.page_num = page_num;
        self
    }

    /// 分析流式内容，转换为虚拟页面。
    ///
    /// 对应 Java: org.ofdrw.layout.StreamCollect#analyze
    ///
    /// 内部链路：
    /// 1. `SegmentationEngine::process` —— 将 Div 队列按页面宽度分段。
    /// 2. `StreamingLayoutAnalyzer::analyze` —— 将段队列分配到虚拟页面并设置坐标。
    /// 3. 若指定了 `page_num`，为每个虚拟页面设置递增页码。
    ///
    /// 调用后，收集器的 content 被消费（drain），与 Java 行为一致。
    pub fn analyze(&mut self, page_layout: &PageLayout) -> Vec<VirtualPage> {
        let divs = std::mem::take(&mut self.content);
        if divs.is_empty() {
            return Vec::new();
        }

        let sgm_engine = SegmentationEngine::new(page_layout.width(), page_layout.page_margin_h());
        let analyzer = StreamingLayoutAnalyzer::new(
            page_layout.width(),
            page_layout.height(),
            page_layout.page_margin_v(),
        );

        // 流式布局队列经过分段引擎，获取分段队列
        let segments = sgm_engine.process(divs);
        // 段队列进入布局分析器，构造基于固定布局的虚拟页面
        let mut virtual_pages = analyzer.analyze(segments);

        // 若指定了目标页号，为每个虚拟页面设置递增页码
        if let Some(start) = self.page_num {
            for (i, vpage) in virtual_pages.iter_mut().enumerate() {
                // 页码索引不会超过 u32::MAX（虚拟页面数量远小于此）
                #[allow(clippy::cast_possible_truncation)]
                let page_num = start + i as u32;
                vpage.page_num = Some(page_num);
            }
        }

        virtual_pages
    }
}

impl Default for StreamCollect {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use easyofd_core::TextObject;

    /// 辅助：创建文本 Div。
    fn text_div(text: &str, w: f64) -> Div {
        let mut d = Div::from_text_object(&TextObject::new(0.0, 0.0, text));
        d.width = w;
        d
    }

    // ── 构造 ──────────────────────────────────────────────────────────────

    #[test]
    fn test_new() {
        let sc = StreamCollect::new();
        assert!(sc.is_empty());
        assert_eq!(sc.len(), 0);
        assert!(sc.page_num().is_none());
    }

    #[test]
    fn test_with_page_num() {
        let sc = StreamCollect::with_page_num(3);
        assert!(sc.is_empty());
        assert_eq!(sc.page_num(), Some(3));
    }

    #[test]
    fn test_default_is_new() {
        let sc = StreamCollect::default();
        assert!(sc.is_empty());
        assert!(sc.page_num().is_none());
    }

    // ── add / content ─────────────────────────────────────────────────────

    #[test]
    fn test_add_returns_self_for_chaining() {
        let mut sc = StreamCollect::new();
        sc.add(text_div("a", 50.0))
            .add(text_div("b", 60.0))
            .add(text_div("c", 40.0));
        assert_eq!(sc.len(), 3);
    }

    #[test]
    fn test_content_and_get_content_alias() {
        let mut sc = StreamCollect::new();
        sc.add(text_div("hello", 50.0));
        assert_eq!(sc.content().len(), 1);
        assert_eq!(sc.get_content().len(), 1);
        assert_eq!(sc.content().len(), sc.get_content().len());
    }

    // ── page_num getter / setter ──────────────────────────────────────────

    #[test]
    fn test_set_page_num() {
        let mut sc = StreamCollect::new();
        sc.set_page_num(Some(5));
        assert_eq!(sc.page_num(), Some(5));
        sc.set_page_num(None);
        assert!(sc.page_num().is_none());
    }

    // ── analyze：基本链路 ──────────────────────────────────────────────────

    #[test]
    fn test_analyze_produces_virtual_pages() {
        let mut sc = StreamCollect::new();
        sc.add(text_div("段落一", 50.0));
        sc.add(text_div("段落二", 50.0));

        let pl = PageLayout::a4();
        let vpages = sc.analyze(&pl);
        assert!(!vpages.is_empty(), "应产出至少 1 个虚拟页面");
        // 分析后 content 被消费
        assert!(sc.is_empty(), "analyze 后 content 应为空");
    }

    #[test]
    fn test_analyze_empty_content_returns_empty() {
        let mut sc = StreamCollect::new();
        let pl = PageLayout::a4();
        let vpages = sc.analyze(&pl);
        assert!(vpages.is_empty());
    }

    #[test]
    fn test_analyze_overflow_creates_multiple_pages() {
        let mut sc = StreamCollect::new();
        // 每个 Div 高 130mm，A4 可用高度约 246.2mm
        for _ in 0..3 {
            let mut d = Div::from_text_object(&TextObject::new(0.0, 0.0, "block"));
            d.width = 50.0;
            d.height = 130.0;
            sc.add(d);
        }

        let pl = PageLayout::a4();
        let vpages = sc.analyze(&pl);
        assert_eq!(vpages.len(), 2, "溢出应产生 2 个虚拟页面");
    }

    // ── analyze：page_num 语义 ────────────────────────────────────────────

    #[test]
    fn test_analyze_with_page_num_sets_page_numbers() {
        let mut sc = StreamCollect::with_page_num(10);
        sc.add(text_div("a", 50.0));
        sc.add(text_div("b", 50.0));

        let pl = PageLayout::a4();
        let vpages = sc.analyze(&pl);
        assert_eq!(vpages.len(), 1);
        assert_eq!(vpages[0].page_num, Some(10));
    }

    #[test]
    fn test_analyze_with_page_num_overflow_increments() {
        let mut sc = StreamCollect::with_page_num(5);
        for _ in 0..3 {
            let mut d = Div::from_text_object(&TextObject::new(0.0, 0.0, "block"));
            d.width = 50.0;
            d.height = 130.0;
            sc.add(d);
        }

        let pl = PageLayout::a4();
        let vpages = sc.analyze(&pl);
        assert_eq!(vpages.len(), 2);
        assert_eq!(vpages[0].page_num, Some(5));
        assert_eq!(vpages[1].page_num, Some(6));
    }

    #[test]
    fn test_analyze_without_page_num_leaves_none() {
        let mut sc = StreamCollect::new();
        sc.add(text_div("a", 50.0));

        let pl = PageLayout::a4();
        let vpages = sc.analyze(&pl);
        assert_eq!(vpages.len(), 1);
        assert!(vpages[0].page_num.is_none());
    }

    // ── analyze：坐标设置验证 ─────────────────────────────────────────────

    #[test]
    fn test_analyze_sets_div_coordinates() {
        let mut sc = StreamCollect::new();
        sc.add(text_div("hello", 50.0));

        let pl = PageLayout::a4();
        let vpages = sc.analyze(&pl);
        let div = &vpages[0].divs[0];
        // StreamingLayoutAnalyzer 使用 page_margin 参数同时作为 x 和 y 的偏移基准
        let margin_v = pl.page_margin_v();
        assert!(
            (div.x - margin_v).abs() < f64::EPSILON,
            "x={}, expected={}",
            div.x,
            margin_v
        );
        assert!(
            (div.y - margin_v).abs() < f64::EPSILON,
            "y={}, expected={}",
            div.y,
            margin_v
        );
    }
}
