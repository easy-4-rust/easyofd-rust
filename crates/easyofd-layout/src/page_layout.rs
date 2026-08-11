//! 页面样式（页面尺寸、边距、背景）。
//!
//! 对应 Java: org.ofdrw.layout.PageLayout

use easyofd_core::CT_PageArea;

use crate::rectangle::Rectangle;

/// 页面样式，描述页面物理尺寸、页边距和可选背景色。
///
/// 对应 Java: org.ofdrw.layout.PageLayout
///
/// 默认为 A4 纸张尺寸（210mm x 297mm），页边距上下 25.4mm、左右 31.7mm。
#[derive(Debug, Clone, PartialEq)]
pub struct PageLayout {
    /// 页面宽度（mm）。
    width: f64,
    /// 页面高度（mm）。
    height: f64,
    /// 上边距（mm），默认 25.4。
    margin_top: f64,
    /// 右边距（mm），默认 31.7。
    margin_right: f64,
    /// 下边距（mm），默认 25.4。
    margin_bottom: f64,
    /// 左边距（mm），默认 31.7。
    margin_left: f64,
    /// 背景色（RGB），`None` 表示无背景。
    background: Option<u32>,
}

impl PageLayout {
    /// A0 纸张（841mm x 1189mm）。
    #[must_use]
    pub fn a0() -> Self {
        Self::new(841.0, 1189.0)
    }

    /// A1 纸张（594mm x 841mm）。
    #[must_use]
    pub fn a1() -> Self {
        Self::new(594.0, 841.0)
    }

    /// A2 纸张（420mm x 594mm）。
    #[must_use]
    pub fn a2() -> Self {
        Self::new(420.0, 594.0)
    }

    /// A3 纸张（297mm x 420mm）。
    #[must_use]
    pub fn a3() -> Self {
        Self::new(297.0, 420.0)
    }

    /// A4 纸张（210mm x 297mm），带默认页边距。
    #[must_use]
    pub fn a4() -> Self {
        Self::new(210.0, 297.0)
    }

    /// A5 纸张（148mm x 210mm）。
    #[must_use]
    pub fn a5() -> Self {
        Self::new(148.0, 210.0)
    }

    /// A6 纸张（105mm x 148mm）。
    #[must_use]
    pub fn a6() -> Self {
        Self::new(105.0, 148.0)
    }

    /// A7 纸张（74mm x 105mm）。
    #[must_use]
    pub fn a7() -> Self {
        Self::new(74.0, 105.0)
    }

    /// A8 纸张（52mm x 74mm）。
    #[must_use]
    pub fn a8() -> Self {
        Self::new(52.0, 74.0)
    }

    /// A9 纸张（37mm x 52mm）。
    #[must_use]
    pub fn a9() -> Self {
        Self::new(37.0, 52.0)
    }

    /// A10 纸张（26mm x 37mm）。
    #[must_use]
    pub fn a10() -> Self {
        Self::new(26.0, 37.0)
    }

    /// 创建页面样式，使用默认 A4 页边距（上下 25.4mm，左右 31.7mm）。
    #[must_use]
    pub fn new(width: f64, height: f64) -> Self {
        Self {
            width,
            height,
            margin_top: 25.4,
            margin_right: 31.7,
            margin_bottom: 25.4,
            margin_left: 31.7,
            background: None,
        }
    }

    /// 获取页面宽度（mm）。
    #[must_use]
    pub fn width(&self) -> f64 {
        self.width
    }

    /// 获取页面高度（mm）。
    #[must_use]
    pub fn height(&self) -> f64 {
        self.height
    }

    /// 获取上边距（mm）。
    #[must_use]
    pub fn margin_top(&self) -> f64 {
        self.margin_top
    }

    /// 获取右边距（mm）。
    #[must_use]
    pub fn margin_right(&self) -> f64 {
        self.margin_right
    }

    /// 获取下边距（mm）。
    #[must_use]
    pub fn margin_bottom(&self) -> f64 {
        self.margin_bottom
    }

    /// 获取左边距（mm）。
    #[must_use]
    pub fn margin_left(&self) -> f64 {
        self.margin_left
    }

    /// 获取背景色（RGB），`None` 表示无背景。
    #[must_use]
    pub fn background(&self) -> Option<u32> {
        self.background
    }

    /// 设置页面宽度（mm）。
    #[must_use]
    pub fn set_width(mut self, width: f64) -> Self {
        self.width = width;
        self
    }

    /// 设置页面高度（mm）。
    #[must_use]
    pub fn set_height(mut self, height: f64) -> Self {
        self.height = height;
        self
    }

    /// 设置上边距（mm）。
    #[must_use]
    pub fn set_margin_top(mut self, top: f64) -> Self {
        self.margin_top = top;
        self
    }

    /// 设置右边距（mm）。
    #[must_use]
    pub fn set_margin_right(mut self, right: f64) -> Self {
        self.margin_right = right;
        self
    }

    /// 设置下边距（mm）。
    #[must_use]
    pub fn set_margin_bottom(mut self, bottom: f64) -> Self {
        self.margin_bottom = bottom;
        self
    }

    /// 设置左边距（mm）。
    #[must_use]
    pub fn set_margin_left(mut self, left: f64) -> Self {
        self.margin_left = left;
        self
    }

    /// 设置四周边距（mm）。
    #[must_use]
    pub fn set_margin(mut self, top: f64, right: f64, bottom: f64, left: f64) -> Self {
        self.margin_top = top;
        self.margin_right = right;
        self.margin_bottom = bottom;
        self.margin_left = left;
        self
    }

    /// 设置背景色（RGB）。
    #[must_use]
    pub fn set_background(mut self, color: u32) -> Self {
        self.background = Some(color);
        self
    }

    /// 清除背景色。
    #[must_use]
    pub fn clear_background(mut self) -> Self {
        self.background = None;
        self
    }

    /// 内容区域宽度（mm）= 页面宽度 - 左边距 - 右边距。
    #[must_use]
    pub fn content_width(&self) -> f64 {
        (self.width - self.margin_left - self.margin_right).max(0.0)
    }

    /// 内容区域高度（mm）= 页面高度 - 上边距 - 下边距。
    #[must_use]
    pub fn content_height(&self) -> f64 {
        (self.height - self.margin_top - self.margin_bottom).max(0.0)
    }

    /// 绘制区域原点 X = 左边距。
    #[must_use]
    pub fn start_x(&self) -> f64 {
        self.margin_left
    }

    /// 绘制区域原点 Y = 上边距。
    #[must_use]
    pub fn start_y(&self) -> f64 {
        self.margin_top
    }

    /// 页面正文工作区域。
    #[must_use]
    pub fn worker_area(&self) -> Rectangle {
        Rectangle::new(
            self.start_x(),
            self.start_y(),
            self.content_width(),
            self.content_height(),
        )
    }

    /// 页边距（对称值，用于 SegmentationEngine / StreamingLayoutAnalyzer）。
    ///
    /// 取左右边距的较大值作为水平边距，上下边距的较大值作为垂直边距。
    #[must_use]
    pub fn page_margin_h(&self) -> f64 {
        self.margin_left.max(self.margin_right)
    }

    /// 页边距（垂直方向）。
    #[must_use]
    pub fn page_margin_v(&self) -> f64 {
        self.margin_top.max(self.margin_bottom)
    }

    /// 转为 OFD 页面区域（`CT_PageArea`）。
    ///
    /// 对应 Java: `PageLayout.getPageArea()`
    #[must_use]
    pub fn page_area(&self) -> CT_PageArea {
        CT_PageArea::new()
            .physical_box(0.0, 0.0, self.width, self.height)
            .application_box(0.0, 0.0, self.width, self.height)
    }
}

impl Default for PageLayout {
    /// 默认 A4 页面样式。
    fn default() -> Self {
        Self::a4()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- A4 默认值 ---

    #[test]
    fn a4_default_dimensions() {
        let pl = PageLayout::a4();
        assert!((pl.width() - 210.0).abs() < f64::EPSILON);
        assert!((pl.height() - 297.0).abs() < f64::EPSILON);
    }

    #[test]
    fn a4_default_margins() {
        let pl = PageLayout::a4();
        assert!((pl.margin_top() - 25.4).abs() < f64::EPSILON);
        assert!((pl.margin_right() - 31.7).abs() < f64::EPSILON);
        assert!((pl.margin_bottom() - 25.4).abs() < f64::EPSILON);
        assert!((pl.margin_left() - 31.7).abs() < f64::EPSILON);
    }

    #[test]
    fn default_trait_is_a4() {
        let pl = PageLayout::default();
        assert!((pl.width() - 210.0).abs() < f64::EPSILON);
        assert!((pl.height() - 297.0).abs() < f64::EPSILON);
    }

    // --- 内容区域 ---

    #[test]
    fn content_dimensions() {
        let pl = PageLayout::a4();
        let expected_w = 210.0 - 31.7 - 31.7;
        let expected_h = 297.0 - 25.4 - 25.4;
        assert!((pl.content_width() - expected_w).abs() < f64::EPSILON);
        assert!((pl.content_height() - expected_h).abs() < f64::EPSILON);
    }

    #[test]
    fn start_coordinates() {
        let pl = PageLayout::a4();
        assert!((pl.start_x() - 31.7).abs() < f64::EPSILON);
        assert!((pl.start_y() - 25.4).abs() < f64::EPSILON);
    }

    // --- Builder ---

    #[test]
    fn custom_builder() {
        let pl = PageLayout::new(297.0, 420.0)
            .set_margin_top(10.0)
            .set_margin_right(20.0)
            .set_margin_bottom(10.0)
            .set_margin_left(20.0)
            .set_background(0xFF_FF00);
        assert!((pl.width() - 297.0).abs() < f64::EPSILON);
        assert!((pl.margin_top() - 10.0).abs() < f64::EPSILON);
        assert_eq!(pl.background(), Some(0xFF_FF00));
    }

    #[test]
    fn set_margin_symmetric() {
        let pl = PageLayout::a4().set_margin(10.0, 20.0, 10.0, 20.0);
        assert!((pl.margin_top() - 10.0).abs() < f64::EPSILON);
        assert!((pl.margin_right() - 20.0).abs() < f64::EPSILON);
        assert!((pl.margin_bottom() - 10.0).abs() < f64::EPSILON);
        assert!((pl.margin_left() - 20.0).abs() < f64::EPSILON);
    }

    #[test]
    fn clear_background() {
        let pl = PageLayout::a4().set_background(0xFF).clear_background();
        assert!(pl.background().is_none());
    }

    // --- 纸张尺寸系列 ---

    #[test]
    fn all_paper_sizes() {
        let sizes = [
            (PageLayout::a0(), 841.0, 1189.0),
            (PageLayout::a1(), 594.0, 841.0),
            (PageLayout::a2(), 420.0, 594.0),
            (PageLayout::a3(), 297.0, 420.0),
            (PageLayout::a4(), 210.0, 297.0),
            (PageLayout::a5(), 148.0, 210.0),
            (PageLayout::a6(), 105.0, 148.0),
            (PageLayout::a7(), 74.0, 105.0),
            (PageLayout::a8(), 52.0, 74.0),
            (PageLayout::a9(), 37.0, 52.0),
            (PageLayout::a10(), 26.0, 37.0),
        ];
        for (pl, w, h) in sizes {
            assert!(
                (pl.width() - w).abs() < f64::EPSILON,
                "width mismatch for {w}x{h}"
            );
            assert!(
                (pl.height() - h).abs() < f64::EPSILON,
                "height mismatch for {w}x{h}"
            );
        }
    }

    // --- Worker area ---

    #[test]
    fn worker_area_matches_content() {
        let pl = PageLayout::a4();
        let wa = pl.worker_area();
        assert!((wa.x - pl.start_x()).abs() < f64::EPSILON);
        assert!((wa.y - pl.start_y()).abs() < f64::EPSILON);
        assert!((wa.width - pl.content_width()).abs() < f64::EPSILON);
        assert!((wa.height - pl.content_height()).abs() < f64::EPSILON);
    }

    // --- Page area ---

    #[test]
    fn page_area_has_physical_and_application_box() {
        let pl = PageLayout::a4();
        let pa = pl.page_area();
        assert!(pa.physical_box.is_some());
        assert!(pa.application_box.is_some());
        let pb = pa.physical_box.as_ref().unwrap();
        assert!(pb.contains("210"));
        assert!(pb.contains("297"));
    }

    // --- Margin helpers ---

    #[test]
    fn margin_helpers() {
        let pl = PageLayout::new(100.0, 100.0).set_margin(10.0, 20.0, 30.0, 40.0);
        assert!((pl.page_margin_h() - 40.0).abs() < f64::EPSILON);
        assert!((pl.page_margin_v() - 30.0).abs() < f64::EPSILON);
    }

    // --- 大边距导致内容区域为零 ---

    #[test]
    fn excessive_margins_yield_zero_content() {
        let pl = PageLayout::new(100.0, 100.0).set_margin(60.0, 60.0, 60.0, 60.0);
        assert!((pl.content_width()).abs() < f64::EPSILON);
        assert!((pl.content_height()).abs() < f64::EPSILON);
    }

    // --- Clone + PartialEq ---

    #[test]
    fn clone_eq() {
        let pl = PageLayout::a4().set_background(0x00AB_CDEF);
        let pl2 = pl.clone();
        assert_eq!(pl, pl2);
    }
}
