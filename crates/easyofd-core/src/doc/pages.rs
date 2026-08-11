//! 页树。
//!
//! 对应 Java: org.ofdrw.core.basicStructure.pageTree.Pages
//! 图 12 页树结构

/// 页树。
///
/// 包含一个或多个页面叶节点，页顺序根据页树前序遍历确定。
///
/// 对应 Java: org.ofdrw.core.basicStructure.pageTree.Pages
#[derive(Debug, Clone, Default)]
pub struct Pages {
    /// 页面列表（ID, BaseLoc）。
    pub pages: Vec<PageEntry>,
}

/// 页面条目。
#[derive(Debug, Clone, PartialEq)]
pub struct PageEntry {
    /// 页面 ID。
    pub id: u32,
    /// 页面文件路径。
    pub base_loc: String,
}

impl PageEntry {
    /// 创建页面条目。
    #[must_use]
    pub fn new(id: u32, base_loc: impl Into<String>) -> Self {
        Self {
            id,
            base_loc: base_loc.into(),
        }
    }
}

impl Pages {
    /// 创建空页树。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 添加页面。
    pub fn add_page(&mut self, entry: PageEntry) {
        self.pages.push(entry);
    }

    /// 获取页面数量。
    #[must_use]
    pub fn len(&self) -> usize {
        self.pages.len()
    }

    /// 是否为空。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.pages.is_empty()
    }

    /// 获取指定索引的页面。
    #[must_use]
    pub fn get(&self, index: usize) -> Option<&PageEntry> {
        self.pages.get(index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pages_new() {
        let p = Pages::new();
        assert!(p.is_empty());
        assert_eq!(p.len(), 0);
    }

    #[test]
    fn pages_add_page() {
        let mut p = Pages::new();
        p.add_page(PageEntry::new(1, "Pages/Page_0.xml"));
        p.add_page(PageEntry::new(2, "Pages/Page_1.xml"));
        assert_eq!(p.len(), 2);
        assert!(!p.is_empty());
    }

    #[test]
    fn pages_get() {
        let mut p = Pages::new();
        p.add_page(PageEntry::new(1, "Pages/Page_0.xml"));
        let entry = p.get(0).unwrap();
        assert_eq!(entry.id, 1);
        assert_eq!(entry.base_loc, "Pages/Page_0.xml");
        assert!(p.get(1).is_none());
    }

    #[test]
    fn page_entry_new() {
        let pe = PageEntry::new(5, "Pages/Page_4.xml");
        assert_eq!(pe.id, 5);
        assert_eq!(pe.base_loc, "Pages/Page_4.xml");
    }

    #[test]
    fn pages_clone_debug() {
        let p = Pages::new();
        let p2 = p.clone();
        assert!(p2.is_empty());
        assert!(format!("{p:?}").contains("Pages"));
    }
}
