//! OFD 图形文档。
//!
//! 对应 Java: org.ofdrw.graphics2d.OFDGraphicsDocument
//!
//! Java 版中继承 `java.awt.Graphics2D` 体系，依赖 AWT 渲染管线。
//! Rust 版提供简化结构，保留核心元数据字段，不包含 AWT 渲染逻辑。

/// OFD 图形文档。
///
/// 对应 Java: org.ofdrw.graphics2d.OFDGraphicsDocument
///
/// 表示一个可通过图形 API 绘制的 OFD 文档容器。
/// Java 版依赖 AWT；Rust 版保留文档级元数据。
#[derive(Debug, Clone)]
pub struct OfdGraphicsDocument {
    /// 文档宽度（mm）。
    pub width: f64,
    /// 文档高度（mm）。
    pub height: f64,
    /// 文档标题。
    pub title: Option<String>,
    /// 作者。
    pub author: Option<String>,
    /// 已创建的页面数。
    pub page_count: u32,
}

impl OfdGraphicsDocument {
    /// 创建新的图形文档（A4 默认尺寸）。
    #[must_use]
    pub fn new() -> Self {
        Self {
            width: 210.0,
            height: 297.0,
            title: None,
            author: None,
            page_count: 0,
        }
    }

    /// 创建指定尺寸的图形文档。
    #[must_use]
    pub fn with_size(width: f64, height: f64) -> Self {
        Self {
            width,
            height,
            title: None,
            author: None,
            page_count: 0,
        }
    }

    /// 设置文档标题。
    #[must_use]
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// 设置作者。
    #[must_use]
    pub fn author(mut self, author: impl Into<String>) -> Self {
        self.author = Some(author.into());
        self
    }

    /// 获取文档宽度（mm）。
    #[must_use]
    pub fn width(&self) -> f64 {
        self.width
    }

    /// 获取文档高度（mm）。
    #[must_use]
    pub fn height(&self) -> f64 {
        self.height
    }

    /// 获取已创建的页面数。
    #[must_use]
    pub fn page_count(&self) -> u32 {
        self.page_count
    }

    /// 新增一页并返回页码。
    pub fn new_page(&mut self) -> u32 {
        self.page_count += 1;
        self.page_count
    }
}

impl Default for OfdGraphicsDocument {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_default_a4() {
        let doc = OfdGraphicsDocument::new();
        assert!((doc.width - 210.0).abs() < f64::EPSILON);
        assert!((doc.height - 297.0).abs() < f64::EPSILON);
        assert!(doc.title.is_none());
        assert_eq!(doc.page_count(), 0);
    }

    #[test]
    fn test_with_size() {
        let doc = OfdGraphicsDocument::with_size(100.0, 200.0);
        assert!((doc.width() - 100.0).abs() < f64::EPSILON);
        assert!((doc.height() - 200.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_builder() {
        let doc = OfdGraphicsDocument::new()
            .title("测试文档")
            .author("easyofd");
        assert_eq!(doc.title.as_deref(), Some("测试文档"));
        assert_eq!(doc.author.as_deref(), Some("easyofd"));
    }

    #[test]
    fn test_new_page() {
        let mut doc = OfdGraphicsDocument::new();
        assert_eq!(doc.new_page(), 1);
        assert_eq!(doc.new_page(), 2);
        assert_eq!(doc.page_count(), 2);
    }

    #[test]
    fn test_default() {
        let doc = OfdGraphicsDocument::default();
        assert_eq!(doc.page_count(), 0);
    }

    #[test]
    fn test_clone_debug() {
        let doc = OfdGraphicsDocument::new().title("x");
        let doc2 = doc.clone();
        assert_eq!(doc2.title.as_deref(), Some("x"));
        assert!(format!("{doc:?}").contains("OfdGraphicsDocument"));
    }
}
