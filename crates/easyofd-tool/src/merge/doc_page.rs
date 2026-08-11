//! 文档页面描述。
//!
//! 对应 Java: org.ofdrw.tool.merge.DocPage
//!
//! 描述待合并文档中的一个页面。

/// 文档页面描述。
///
/// 对应 Java: `org.ofdrw.tool.merge.DocPage`
///
/// 描述源 OFD 文档中的一个页面，包含页面尺寸和源文档/页面索引。
#[derive(Debug, Clone)]
pub struct DocPage {
    /// 源文档索引（在合并上下文中的编号）。
    pub source_index: usize,
    /// 源文档中的页面索引（从 0 开始）。
    pub page_index: usize,
    /// 页面宽度（mm）。
    pub width: f64,
    /// 页面高度（mm）。
    pub height: f64,
}

impl DocPage {
    /// 创建文档页面描述。
    #[must_use]
    pub fn new(source_index: usize, page_index: usize, width: f64, height: f64) -> Self {
        Self {
            source_index,
            page_index,
            width,
            height,
        }
    }

    /// 获取源文档索引。
    #[must_use]
    pub fn source_index(&self) -> usize {
        self.source_index
    }

    /// 获取页面索引。
    #[must_use]
    pub fn page_index(&self) -> usize {
        self.page_index
    }

    /// 获取页面宽度。
    #[must_use]
    pub fn width(&self) -> f64 {
        self.width
    }

    /// 获取页面高度。
    #[must_use]
    pub fn height(&self) -> f64 {
        self.height
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_page() {
        let page = DocPage::new(0, 2, 210.0, 297.0);
        assert_eq!(page.source_index(), 0);
        assert_eq!(page.page_index(), 2);
        assert!((page.width() - 210.0).abs() < f64::EPSILON);
        assert!((page.height() - 297.0).abs() < f64::EPSILON);
    }

    #[test]
    fn clone_page() {
        let page = DocPage::new(1, 0, 100.0, 200.0);
        let cloned = page.clone();
        assert_eq!(cloned.source_index, 1);
        assert_eq!(cloned.page_index, 0);
    }
}
