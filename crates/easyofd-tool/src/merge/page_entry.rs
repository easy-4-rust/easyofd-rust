//! 页面项目（PageEntry）。
//!
//! 对应 Java: org.ofdrw.tool.merge.PageEntry
//!
//! 在 OFD 文档合并过程中描述一个待迁移的页面，支持页面混合（tbMixPages）。

/// 页面项目。
///
/// 对应 Java: `org.ofdrw.tool.merge.PageEntry`
///
/// 描述合并过程中一个待迁移的页面。每个 `PageEntry` 关联一个文档上下文索引
/// （对应 [`DocContext`](super::DocContext) 中注册的源文档）和页码（从 1 开始）。
///
/// 支持页面混合：通过 `tb_mix_pages` 字段将其他页面叠加到当前页面上方。
#[derive(Debug, Clone)]
pub struct PageEntry {
    /// 关联文档上下文的源文档索引。
    ///
    /// 对应 Java: `DocContext docCtx`，此处存储源文档在合并上下文中的索引。
    pub doc_ctx_index: usize,

    /// 页面索引号（页码从 1 开始）。
    ///
    /// 对应 Java: `Integer pageIndex`
    pub page_index: usize,

    /// 是否复制模板。
    ///
    /// 对应 Java: `boolean copyTemplate`，默认 `true`。
    pub copy_template: bool,

    /// 是否复制注释。
    ///
    /// 对应 Java: `boolean copyAnnotations`，默认 `true`。
    pub copy_annotations: bool,

    /// 需要混合到指定页面的其他文档页面。
    ///
    /// 对应 Java: `List<PageEntry> tbMixPages`
    ///
    /// 将列表中的页面追加到当前页面的上方（top-to-bottom 混合）。
    pub tb_mix_pages: Vec<PageEntry>,
}

impl PageEntry {
    /// 创建页面项目。
    ///
    /// # 参数
    ///
    /// - `page_index`：页面索引号（从 1 开始）。
    /// - `doc_ctx_index`：关联文档上下文的源文档索引。
    ///
    /// 默认复制模板和注释，无混合页面。
    ///
    /// # 对应 Java
    ///
    /// `PageEntry(Integer pageIndex, DocContext docCtx)`
    #[must_use]
    pub fn new(page_index: usize, doc_ctx_index: usize) -> Self {
        Self {
            doc_ctx_index,
            page_index,
            copy_template: true,
            copy_annotations: true,
            tb_mix_pages: Vec::new(),
        }
    }

    /// 创建带混合页面的页面项目。
    ///
    /// # 参数
    ///
    /// - `page_index`：页面索引号（从 1 开始）。
    /// - `doc_ctx_index`：关联文档上下文的源文档索引。
    /// - `tb_mix_pages`：需要混合到当前页面的其他页面列表。
    ///
    /// # 对应 Java
    ///
    /// `PageEntry(Integer pageIndex, DocContext docCtx, PageEntry... tbMixPages)`
    #[must_use]
    pub fn with_mix_pages(
        page_index: usize,
        doc_ctx_index: usize,
        tb_mix_pages: Vec<PageEntry>,
    ) -> Self {
        Self {
            doc_ctx_index,
            page_index,
            copy_template: true,
            copy_annotations: true,
            tb_mix_pages,
        }
    }

    /// 设置是否复制模板。
    #[must_use]
    pub fn copy_template(mut self, copy: bool) -> Self {
        self.copy_template = copy;
        self
    }

    /// 设置是否复制注释。
    #[must_use]
    pub fn copy_annotations(mut self, copy: bool) -> Self {
        self.copy_annotations = copy;
        self
    }

    /// 获取页面索引号。
    #[must_use]
    pub fn page_index(&self) -> usize {
        self.page_index
    }

    /// 获取关联文档上下文索引。
    #[must_use]
    pub fn doc_ctx_index(&self) -> usize {
        self.doc_ctx_index
    }

    /// 获取是否复制模板。
    #[must_use]
    pub fn should_copy_template(&self) -> bool {
        self.copy_template
    }

    /// 获取是否复制注释。
    #[must_use]
    pub fn should_copy_annotations(&self) -> bool {
        self.copy_annotations
    }

    /// 获取混合页面列表。
    #[must_use]
    pub fn tb_mix_pages(&self) -> &[PageEntry] {
        &self.tb_mix_pages
    }

    /// 是否包含混合页面。
    #[must_use]
    pub fn has_mix_pages(&self) -> bool {
        !self.tb_mix_pages.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_page_entry() {
        let entry = PageEntry::new(1, 0);
        assert_eq!(entry.page_index(), 1);
        assert_eq!(entry.doc_ctx_index(), 0);
        assert!(entry.should_copy_template());
        assert!(entry.should_copy_annotations());
        assert!(!entry.has_mix_pages());
    }

    #[test]
    fn builder_chain() {
        let entry = PageEntry::new(3, 1)
            .copy_template(false)
            .copy_annotations(false);
        assert_eq!(entry.page_index(), 3);
        assert!(!entry.should_copy_template());
        assert!(!entry.should_copy_annotations());
    }

    #[test]
    fn with_mix_pages() {
        let mix1 = PageEntry::new(2, 1);
        let mix2 = PageEntry::new(3, 1);
        let entry = PageEntry::with_mix_pages(1, 0, vec![mix1, mix2]);
        assert!(entry.has_mix_pages());
        assert_eq!(entry.tb_mix_pages().len(), 2);
        assert_eq!(entry.tb_mix_pages()[0].page_index(), 2);
        assert_eq!(entry.tb_mix_pages()[1].page_index(), 3);
    }

    #[test]
    fn empty_mix_pages() {
        let entry = PageEntry::with_mix_pages(1, 0, vec![]);
        assert!(!entry.has_mix_pages());
    }

    #[test]
    fn clone_page_entry() {
        let entry = PageEntry::new(5, 2).copy_template(false);
        let cloned = entry.clone();
        assert_eq!(cloned.page_index(), 5);
        assert_eq!(cloned.doc_ctx_index(), 2);
        assert!(!cloned.should_copy_template());
    }

    #[test]
    fn nested_mix_pages() {
        // 测试嵌套混合页面（tb_mix_pages 中的 PageEntry 自身也有 tb_mix_pages）
        let inner_mix = PageEntry::new(10, 2);
        let outer_mix = PageEntry::with_mix_pages(5, 1, vec![inner_mix]);
        let entry = PageEntry::with_mix_pages(1, 0, vec![outer_mix]);
        assert!(entry.has_mix_pages());
        assert!(entry.tb_mix_pages()[0].has_mix_pages());
        assert_eq!(entry.tb_mix_pages()[0].tb_mix_pages()[0].page_index(), 10);
    }

    #[test]
    fn page_index_from_one() {
        // Java 页码从 1 开始
        let entry = PageEntry::new(1, 0);
        assert_eq!(entry.page_index(), 1);
    }
}
