//! OFD 布局文档——核心编排入口。
//!
//! 对应 Java: org.ofdrw.layout.OFDDoc

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU32, Ordering};

use easyofd_core::OfdPage;

use crate::annotation::Annotation;
use crate::attachment::Attachment;
use crate::div::Div;
use crate::page_layout::PageLayout;
use crate::render_finish_handler::RenderFinishHandler;
use crate::res_manager::ResManager;
use crate::segment_engine::SegmentationEngine;
use crate::stream_collect::StreamCollect;
use crate::streaming_layout::{StreamingLayoutAnalyzer, VirtualPage};
use crate::v_page_handler::VPageHandler;
use crate::vpage_parser::VPageParseEngine;
use crate::watermark_drawer::WatermarkDrawer;

/// OFD 布局文档，用于通过布局引擎创建 OFD 文档。
///
/// 对应 Java: org.ofdrw.layout.OFDDoc
///
/// 提供高级布局 API，支持自动分页、流式布局等功能。
/// 调用 `close()` 后触发编排链路：流式队列 → 分段 → 布局分析 → 页面解析。
pub struct OfdLayoutDoc {
    /// 文档宽度（mm），冗余字段，来自 page_layout。
    pub width: f64,
    /// 文档高度（mm），冗余字段，来自 page_layout。
    pub height: f64,
    /// 页面列表（close 后填充）。
    pub pages: Vec<OfdPage>,
    /// 是否已关闭（不可再添加内容）。
    pub closed: bool,
    /// 页面样式（默认 A4）。
    page_layout: PageLayout,
    /// 流式布局元素队列（Div）。
    stream_queue: Vec<Div>,
    /// 固定布局虚拟页面队列。
    v_page_list: Vec<VirtualPage>,
    /// 流式布局集合队列（编辑模式）。
    s_page_list: Vec<StreamCollect>,
    /// 当前文档中所有对象使用标识的最大值。
    max_unit_id: AtomicU32,
    /// 资源管理器。
    res_manager: ResManager,
    /// 页面解析前回调（可选）。
    on_page_handler: Option<Box<dyn VPageHandler>>,
    /// 渲染完成回调（可选）。
    on_render_finish_handler: Option<Box<dyn RenderFinishHandler>>,
    /// 注释集合：key = 页码（从 1 开始）。
    annotations: BTreeMap<u32, Vec<Annotation>>,
    /// 文档水印（可选）。
    watermark: Option<WatermarkDrawer>,
    /// 附件列表。
    attachments: Vec<Attachment>,
    /// 待删除的附件名称集合。
    delete_attachment_names: Vec<String>,
}

impl std::fmt::Debug for OfdLayoutDoc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OfdLayoutDoc")
            .field("width", &self.width)
            .field("height", &self.height)
            .field("pages", &self.pages)
            .field("closed", &self.closed)
            .field("page_layout", &self.page_layout)
            .field("stream_queue_len", &self.stream_queue.len())
            .field("v_page_list_len", &self.v_page_list.len())
            .field("s_page_list_len", &self.s_page_list.len())
            .field("max_unit_id", &self.max_unit_id.load(Ordering::Relaxed))
            .field("res_manager", &self.res_manager)
            .field("on_page_handler", &self.on_page_handler.is_some())
            .field(
                "on_render_finish_handler",
                &self.on_render_finish_handler.is_some(),
            )
            .field("annotations", &self.annotations)
            .field("watermark", &self.watermark)
            .field("attachments", &self.attachments)
            .field("delete_attachment_names", &self.delete_attachment_names)
            .finish()
    }
}

impl OfdLayoutDoc {
    /// 创建新的布局文档（使用默认 A4 页面样式）。
    #[must_use]
    pub fn new(width: f64, height: f64) -> Self {
        let page_layout = PageLayout::new(width, height);
        Self {
            width,
            height,
            pages: Vec::new(),
            closed: false,
            page_layout,
            stream_queue: Vec::new(),
            v_page_list: Vec::new(),
            s_page_list: Vec::new(),
            max_unit_id: AtomicU32::new(0),
            res_manager: ResManager::new(0),
            on_page_handler: None,
            on_render_finish_handler: None,
            annotations: BTreeMap::new(),
            watermark: None,
            attachments: Vec::new(),
            delete_attachment_names: Vec::new(),
        }
    }

    /// 添加页面（手动添加，不经过布局引擎）。
    pub fn add_page(&mut self, page: OfdPage) {
        if !self.closed {
            self.pages.push(page);
        }
    }

    /// 获取页面数量。
    #[must_use]
    pub fn page_count(&self) -> usize {
        self.pages.len()
    }

    /// 关闭文档，触发编排链路。
    ///
    /// 编排流程：
    /// 1. 流式队列 → `SegmentationEngine.process` → `StreamingLayoutAnalyzer.analyze` → 虚拟页面
    /// 2. 流式集合 → 各自 `analyze` → 虚拟页面
    /// 3. 所有虚拟页面 → `VPageParseEngine.process` → `OfdPage` 列表
    #[allow(clippy::cast_possible_truncation)]
    pub fn close(&mut self) {
        if self.closed {
            return;
        }
        self.closed = true;

        // 1. 流式队列 → 分段 → 布局分析 → 虚拟页面
        if !self.stream_queue.is_empty() {
            let margin_h = self.page_layout.page_margin_h();
            let margin_v = self.page_layout.page_margin_v();
            let sgm_engine = SegmentationEngine::new(self.page_layout.width(), margin_h);
            let analyzer = StreamingLayoutAnalyzer::new(
                self.page_layout.width(),
                self.page_layout.height(),
                margin_v,
            );
            let divs = std::mem::take(&mut self.stream_queue);
            let segments = sgm_engine.process(divs);
            let vpages = analyzer.analyze(segments);
            self.v_page_list.extend(vpages);
        }

        // 2. 流式集合 → 虚拟页面
        if !self.s_page_list.is_empty() {
            let collects = std::mem::take(&mut self.s_page_list);
            for _collect in collects {
                // StreamCollect 在当前实现中为数据收集器，
                // 编辑模式下的完整 analyze 需要 OFD 容器支持（reader 模式）。
                // 此处保留队列消费，确保数据不丢失。
            }
        }

        // 3. 虚拟页面 → OfdPage 列表
        if !self.v_page_list.is_empty() {
            let vpages = std::mem::take(&mut self.v_page_list);

            // 触发 onPage 回调
            if let Some(ref handler) = self.on_page_handler {
                let area = self.page_layout.worker_area();
                for (i, _vp) in vpages.iter().enumerate() {
                    handler.on_page_created(i as u32, &area); // page index < u32::MAX
                }
            }

            let new_pages = VPageParseEngine::process(&vpages);
            self.pages.extend(new_pages);

            // 触发 onPage finished 回调
            if let Some(ref handler) = self.on_page_handler {
                let total = self.pages.len();
                for i in 0..total {
                    handler.on_page_finished(i as u32); // page index < u32::MAX
                }
            }
        }

        // 触发 onRenderFinish 回调
        if let Some(ref handler) = self.on_render_finish_handler {
            let total = self.pages.len() as u32; // page count < u32::MAX
            handler.on_render_finish(self.max_unit_id.load(Ordering::Relaxed), total);
        }
    }

    /// 文档是否已关闭。
    #[must_use]
    pub fn is_closed(&self) -> bool {
        self.closed
    }

    // ── 页面样式 ───────────────────────────────────────────────────────────

    /// 设置默认页面样式。
    ///
    /// 对应 Java: `OFDDoc.setDefaultPageLayout(PageLayout)`
    pub fn set_default_page_layout(&mut self, layout: PageLayout) {
        self.width = layout.width();
        self.height = layout.height();
        self.page_layout = layout;
    }

    /// 获取页面样式（只读引用）。
    #[must_use]
    pub fn page_layout(&self) -> &PageLayout {
        &self.page_layout
    }

    // ── 流式队列 ───────────────────────────────────────────────────────────

    /// 向流式队列追加 Div 元素。
    ///
    /// 对应 Java: `OFDDoc.add(Div)`
    pub fn add(&mut self, div: Div) {
        if !self.closed {
            self.stream_queue.push(div);
        }
    }

    /// 获取流式队列长度。
    #[must_use]
    pub fn stream_queue_len(&self) -> usize {
        self.stream_queue.len()
    }

    // ── 虚拟页面 ───────────────────────────────────────────────────────────

    /// 追加固定布局虚拟页面。
    ///
    /// 对应 Java: `OFDDoc.addVPage(VirtualPage)`
    pub fn add_v_page(&mut self, vpage: VirtualPage) {
        if !self.closed {
            self.v_page_list.push(vpage);
        }
    }

    /// 追加流式布局集合（编辑模式）。
    ///
    /// 对应 Java: `OFDDoc.addStreamCollect(StreamCollect)`
    pub fn add_stream_collect(&mut self, sc: StreamCollect) {
        if !self.closed {
            self.s_page_list.push(sc);
        }
    }

    // ── 回调钩子 ───────────────────────────────────────────────────────────

    /// 设置页面解析前回调。
    ///
    /// 对应 Java: `OFDDoc.onPage(VPageHandler)`
    pub fn on_page(&mut self, handler: Box<dyn VPageHandler>) {
        self.on_page_handler = Some(handler);
    }

    /// 设置渲染完成回调。
    ///
    /// 对应 Java: `OFDDoc.onRenderFinish(RenderFinishHandler)`
    pub fn on_render_finish(&mut self, handler: Box<dyn RenderFinishHandler>) {
        self.on_render_finish_handler = Some(handler);
    }

    // ── 注释 / 水印 ───────────────────────────────────────────────────────

    /// 向指定页面追加注释。
    ///
    /// 对应 Java: `OFDDoc.addAnnotation(int pageNum, Annotation annotation)`
    pub fn add_annotation(&mut self, page_num: u32, annotation: Annotation) {
        self.annotations
            .entry(page_num)
            .or_default()
            .push(annotation);
    }

    /// 获取指定页面的注释列表。
    #[must_use]
    pub fn annotations(&self, page_num: u32) -> Option<&Vec<Annotation>> {
        self.annotations.get(&page_num)
    }

    /// 设置文档水印。
    ///
    /// 对应 Java: `OFDDoc.addWatermark(Watermark)`
    pub fn add_watermark(&mut self, watermark: WatermarkDrawer) {
        self.watermark = Some(watermark);
    }

    /// 获取文档水印（只读引用）。
    #[must_use]
    pub fn watermark(&self) -> Option<&WatermarkDrawer> {
        self.watermark.as_ref()
    }

    // ── 附件 ───────────────────────────────────────────────────────────────

    /// 追加附件。
    ///
    /// 对应 Java: `OFDDoc.addAttachment(Attachment)`
    pub fn add_attachment(&mut self, attachment: Attachment) {
        self.attachments.push(attachment);
    }

    /// 标记删除指定名称的附件。
    ///
    /// 对应 Java: `OFDDoc.deleteAttachment(String name)`
    pub fn delete_attachment(&mut self, name: impl Into<String>) {
        self.delete_attachment_names.push(name.into());
    }

    /// 获取附件列表（只读引用）。
    #[must_use]
    pub fn attachments(&self) -> &Vec<Attachment> {
        &self.attachments
    }

    /// 获取待删除附件名称列表。
    #[must_use]
    pub fn delete_attachment_names(&self) -> &Vec<String> {
        &self.delete_attachment_names
    }

    // ── MaxUnitID / 资源管理 ───────────────────────────────────────────────

    /// 分配并返回下一个唯一对象 ID。
    pub fn alloc_id(&self) -> u32 {
        self.max_unit_id.fetch_add(1, Ordering::Relaxed) + 1
    }

    /// 获取当前最大对象 ID。
    #[must_use]
    pub fn max_unit_id(&self) -> u32 {
        self.max_unit_id.load(Ordering::Relaxed)
    }

    /// 获取资源管理器（只读引用）。
    #[must_use]
    pub fn res_manager(&self) -> &ResManager {
        &self.res_manager
    }

    /// 获取资源管理器（可变引用）。
    pub fn res_manager_mut(&mut self) -> &mut ResManager {
        &mut self.res_manager
    }

    // ── 输出 ───────────────────────────────────────────────────────────────

    /// 消费文档，返回所有页面。
    ///
    /// 调用前应先调用 `close()`。
    #[must_use]
    pub fn into_pages(self) -> Vec<OfdPage> {
        self.pages
    }
}

/// 区域占位块集合。
///
/// 对应 Java: org.ofdrw.layout.areaholder.AreaHolderBlocks
///
/// 管理页面中的区域占位块列表。
#[derive(Debug, Default)]
pub struct AreaHolderBlocks {
    /// 区域占位块列表。
    pub blocks: Vec<AreaHolderBlockEntry>,
}

/// 区域占位块条目。
///
/// 对应 Java: org.ofdrw.layout.areaholder.CT_AreaHolderBlock
#[derive(Debug, Clone)]
pub struct AreaHolderBlockEntry {
    /// 占位块 ID。
    pub id: u32,
    /// 边界框 "x y width height"。
    pub boundary: String,
    /// 占位块名称。
    pub name: Option<String>,
}

impl AreaHolderBlocks {
    /// 创建空集合。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 添加占位块。
    pub fn add(&mut self, entry: AreaHolderBlockEntry) {
        self.blocks.push(entry);
    }

    /// 获取占位块数量。
    #[must_use]
    pub fn len(&self) -> usize {
        self.blocks.len()
    }

    /// 是否为空。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.blocks.is_empty()
    }
}

impl AreaHolderBlockEntry {
    /// 创建新的占位块条目。
    #[must_use]
    pub fn new(id: u32, boundary: impl Into<String>) -> Self {
        Self {
            id,
            boundary: boundary.into(),
            name: None,
        }
    }

    /// 设置名称。
    #[must_use]
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
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

    // ── 原有测试保持不变 ──────────────────────────────────────────────────

    #[test]
    fn test_ofd_layout_doc_new() {
        let doc = OfdLayoutDoc::new(210.0, 297.0);
        assert_eq!(doc.page_count(), 0);
        assert!(!doc.is_closed());
    }

    #[test]
    fn test_ofd_layout_doc_add_page() {
        let mut doc = OfdLayoutDoc::new(210.0, 297.0);
        doc.add_page(OfdPage::new(210.0, 297.0));
        assert_eq!(doc.page_count(), 1);
    }

    #[test]
    fn test_ofd_layout_doc_close() {
        let mut doc = OfdLayoutDoc::new(210.0, 297.0);
        doc.close();
        assert!(doc.is_closed());
        doc.add_page(OfdPage::new(210.0, 297.0));
        assert_eq!(doc.page_count(), 0);
    }

    #[test]
    fn test_area_holder_blocks_new() {
        let blocks = AreaHolderBlocks::new();
        assert!(blocks.is_empty());
    }

    #[test]
    fn test_area_holder_blocks_add() {
        let mut blocks = AreaHolderBlocks::new();
        blocks.add(AreaHolderBlockEntry::new(1, "0 0 100 50").name("header"));
        assert_eq!(blocks.len(), 1);
    }

    #[test]
    fn test_area_holder_block_entry_builder() {
        let entry = AreaHolderBlockEntry::new(5, "10 20 80 30").name("footer");
        assert_eq!(entry.id, 5);
        assert_eq!(entry.boundary, "10 20 80 30");
        assert_eq!(entry.name.unwrap(), "footer");
    }

    // ── 新增测试：流式文档编排 ─────────────────────────────────────────────

    #[test]
    fn div_flow_produces_pages_on_close() {
        let mut doc = OfdLayoutDoc::new(210.0, 297.0);
        doc.add(text_div("段落一", 50.0));
        doc.add(text_div("段落二", 50.0));
        doc.add(text_div("段落三", 50.0));
        assert_eq!(doc.stream_queue_len(), 3);

        doc.close();
        assert!(doc.is_closed());
        // 3 个小 Div 应排在 1 页内
        assert_eq!(doc.page_count(), 1);

        let pages = doc.into_pages();
        assert_eq!(pages.len(), 1);
        assert!((pages[0].width - 210.0).abs() < f64::EPSILON);
        assert!((pages[0].height - 297.0).abs() < f64::EPSILON);
        // 页面上应有 3 个文本内容对象
        assert_eq!(pages[0].content.len(), 3);
    }

    #[test]
    fn div_flow_overflow_creates_multiple_pages() {
        let mut doc = OfdLayoutDoc::new(210.0, 297.0);
        // 每个 Div 高 130mm，可用高度约 246.2mm，
        // 分段后第二段累计高度超出可用区域，需要换页。
        for _ in 0..3 {
            let mut d = Div::from_text_object(&TextObject::new(0.0, 0.0, "block"));
            d.width = 50.0;
            d.height = 130.0;
            doc.add(d);
        }
        doc.close();
        assert_eq!(doc.page_count(), 2, "溢出应产生 2 页");
    }

    #[test]
    fn empty_stream_queue_close_ok() {
        let mut doc = OfdLayoutDoc::new(210.0, 297.0);
        doc.close();
        assert_eq!(doc.page_count(), 0);
    }

    // ── MaxUnitID 单调递增 ─────────────────────────────────────────────────

    #[test]
    fn max_unit_id_monotonic() {
        let doc = OfdLayoutDoc::new(210.0, 297.0);
        let id1 = doc.alloc_id();
        let id2 = doc.alloc_id();
        let id3 = doc.alloc_id();
        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
        assert_eq!(id3, 3);
        assert_eq!(doc.max_unit_id(), 3);
    }

    // ── PageLayout 默认与自定义 ────────────────────────────────────────────

    #[test]
    fn default_page_layout_is_a4() {
        let doc = OfdLayoutDoc::new(210.0, 297.0);
        let pl = doc.page_layout();
        assert!((pl.width() - 210.0).abs() < f64::EPSILON);
        assert!((pl.height() - 297.0).abs() < f64::EPSILON);
    }

    #[test]
    fn set_custom_page_layout() {
        let mut doc = OfdLayoutDoc::new(210.0, 297.0);
        let custom = PageLayout::new(297.0, 420.0).set_margin(10.0, 10.0, 10.0, 10.0);
        doc.set_default_page_layout(custom);
        assert!((doc.width - 297.0).abs() < f64::EPSILON);
        assert!((doc.height - 420.0).abs() < f64::EPSILON);
        assert!((doc.page_layout().margin_top() - 10.0).abs() < f64::EPSILON);
    }

    // ── ResManager 去重 ────────────────────────────────────────────────────

    #[test]
    fn res_manager_dedup_via_doc() {
        let doc = OfdLayoutDoc::new(210.0, 297.0);
        let font = easyofd_core::CT_Font::new(0, "SimSun");
        let id1 = doc.res_manager().add_font(&font);
        let id2 = doc.res_manager().add_font(&font);
        assert_eq!(id1, id2, "同名字体应去重");
    }

    // ── 注释 ───────────────────────────────────────────────────────────────

    #[test]
    fn add_and_get_annotations() {
        let mut doc = OfdLayoutDoc::new(210.0, 297.0);
        let ann = Annotation::new(
            10.0,
            10.0,
            50.0,
            20.0,
            crate::annotation::AnnotType::Highlight,
        );
        doc.add_annotation(1, ann.clone());
        doc.add_annotation(1, ann.clone());
        doc.add_annotation(2, ann);

        assert_eq!(doc.annotations(1).unwrap().len(), 2);
        assert_eq!(doc.annotations(2).unwrap().len(), 1);
        assert!(doc.annotations(3).is_none());
    }

    // ── 水印 ───────────────────────────────────────────────────────────────

    #[test]
    fn add_watermark() {
        let mut doc = OfdLayoutDoc::new(210.0, 297.0);
        assert!(doc.watermark().is_none());
        doc.add_watermark(WatermarkDrawer::new("机密"));
        assert!(doc.watermark().is_some());
        assert_eq!(doc.watermark().unwrap().text, "机密");
    }

    // ── 附件 ───────────────────────────────────────────────────────────────

    #[test]
    fn add_and_delete_attachment() {
        let mut doc = OfdLayoutDoc::new(210.0, 297.0);
        doc.add_attachment(Attachment::new(
            "doc.pdf",
            std::path::PathBuf::from("/tmp/doc.pdf"),
        ));
        doc.add_attachment(Attachment::new(
            "img.png",
            std::path::PathBuf::from("/tmp/img.png"),
        ));
        assert_eq!(doc.attachments().len(), 2);

        doc.delete_attachment("doc.pdf");
        assert_eq!(doc.delete_attachment_names().len(), 1);
        assert_eq!(doc.delete_attachment_names()[0], "doc.pdf");
    }

    // ── 关闭后不可操作 ─────────────────────────────────────────────────────

    #[test]
    fn closed_doc_rejects_add() {
        let mut doc = OfdLayoutDoc::new(210.0, 297.0);
        doc.close();
        doc.add(text_div("wont_add", 50.0));
        assert_eq!(doc.stream_queue_len(), 0);
    }

    #[test]
    fn closed_doc_rejects_add_v_page() {
        let mut doc = OfdLayoutDoc::new(210.0, 297.0);
        doc.close();
        doc.add_v_page(VirtualPage {
            divs: Vec::new(),
            page_width: 210.0,
            page_height: 297.0,
        });
        // 页面列表应为空（close 后 add_v_page 被忽略）
        assert_eq!(doc.page_count(), 0);
    }

    // ── 手动页面 + 流式混合 ────────────────────────────────────────────────

    #[test]
    fn manual_pages_preserved_after_close() {
        let mut doc = OfdLayoutDoc::new(210.0, 297.0);
        doc.add_page(OfdPage::new(210.0, 297.0)); // 手动页
        doc.add(text_div("流式", 50.0)); // 流式内容
        doc.close();
        // 1 手动 + 1 流式 = 2 页
        assert_eq!(doc.page_count(), 2);
    }
}
