//! OFD 布局文档。
//!
//! 对应 Java: org.ofdrw.layout.OFDDoc

use easyofd_core::OfdPage;

/// OFD 布局文档，用于通过布局引擎创建 OFD 文档。
///
/// 对应 Java: org.ofdrw.layout.OFDDoc
///
/// 提供高级布局 API，支持自动分页、流式布局等功能。
#[derive(Debug)]
pub struct OfdLayoutDoc {
    /// 文档宽度（mm）。
    pub width: f64,
    /// 文档高度（mm）。
    pub height: f64,
    /// 页面列表。
    pub pages: Vec<OfdPage>,
    /// 是否已关闭（不可再添加内容）。
    pub closed: bool,
}

impl OfdLayoutDoc {
    /// 创建新的布局文档。
    #[must_use]
    pub fn new(width: f64, height: f64) -> Self {
        Self {
            width,
            height,
            pages: Vec::new(),
            closed: false,
        }
    }

    /// 添加页面。
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

    /// 关闭文档（不可再添加内容）。
    pub fn close(&mut self) {
        self.closed = true;
    }

    /// 文档是否已关闭。
    #[must_use]
    pub fn is_closed(&self) -> bool {
        self.closed
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
        let entry = AreaHolderBlockEntry::new(5, "10 20 80 30")
            .name("footer");
        assert_eq!(entry.id, 5);
        assert_eq!(entry.boundary, "10 20 80 30");
        assert_eq!(entry.name.unwrap(), "footer");
    }
}
