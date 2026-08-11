//! 文档根节点。
//!
//! 对应 Java: org.ofdrw.core.basicStructure.doc.Document
//! Document.xml 文档根节点，对应 GB/T 33190-2016 图 5

use crate::basic_type::ST_Loc;

/// 文档根节点。
///
/// Document.xml 文档根节点，定义了文档的页面、资源、权限等信息。
///
/// 对应 Java: org.ofdrw.core.basicStructure.doc.Document
#[derive(Debug, Clone, Default)]
pub struct Document {
    /// 文档公共数据引用。
    pub common_data: Option<String>,
    /// 页树引用。
    pub pages: Vec<PageRef>,
    /// 大纲引用。
    pub outlines: Option<String>,
    /// 权限声明。
    pub permissions: Option<String>,
    /// 动作序列。
    pub actions: Option<String>,
    /// 视图首选项。
    pub v_preferences: Option<String>,
    /// 书签集。
    pub bookmarks: Option<String>,
    /// 注释列表路径。
    pub annotations: Option<ST_Loc>,
    /// 自定义标引列表路径。
    pub custom_tags: Option<ST_Loc>,
    /// 附件列表路径。
    pub attachments: Option<ST_Loc>,
    /// 扩展列表路径。
    pub extensions: Option<ST_Loc>,
}

/// 页面引用。
#[derive(Debug, Clone, PartialEq)]
pub struct PageRef {
    /// 页面 ID。
    pub id: u32,
    /// 页面文件路径。
    pub base_loc: ST_Loc,
}

impl PageRef {
    /// 创建页面引用。
    #[must_use]
    pub fn new(id: u32, base_loc: ST_Loc) -> Self {
        Self { id, base_loc }
    }
}

impl Document {
    /// 创建新的文档根节点。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 添加页面引用。
    pub fn add_page(&mut self, page: PageRef) {
        self.pages.push(page);
    }

    /// 获取页面数量。
    #[must_use]
    pub fn page_count(&self) -> usize {
        self.pages.len()
    }

    /// 设置注释路径。
    #[must_use]
    pub fn annotations(mut self, path: ST_Loc) -> Self {
        self.annotations = Some(path);
        self
    }

    /// 设置附件路径。
    #[must_use]
    pub fn attachments(mut self, path: ST_Loc) -> Self {
        self.attachments = Some(path);
        self
    }

    /// 设置扩展路径。
    #[must_use]
    pub fn extensions(mut self, path: ST_Loc) -> Self {
        self.extensions = Some(path);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn document_new() {
        let doc = Document::new();
        assert!(doc.pages.is_empty());
        assert!(doc.annotations.is_none());
    }

    #[test]
    fn document_add_page() {
        let mut doc = Document::new();
        doc.add_page(PageRef::new(1, ST_Loc::new("Pages/Page_0.xml")));
        doc.add_page(PageRef::new(2, ST_Loc::new("Pages/Page_1.xml")));
        assert_eq!(doc.page_count(), 2);
    }

    #[test]
    fn page_ref_new() {
        let pr = PageRef::new(1, ST_Loc::new("Pages/Page_0.xml"));
        assert_eq!(pr.id, 1);
        assert_eq!(pr.base_loc.loc(), "Pages/Page_0.xml");
    }

    #[test]
    fn document_builder() {
        let doc = Document::new()
            .annotations(ST_Loc::new("Annotations.xml"))
            .attachments(ST_Loc::new("Attachments.xml"));
        assert!(doc.annotations.is_some());
        assert!(doc.attachments.is_some());
    }

    #[test]
    fn document_clone_debug() {
        let doc = Document::new();
        let doc2 = doc.clone();
        assert_eq!(doc2.page_count(), 0);
        assert!(format!("{doc:?}").contains("Document"));
    }
}
