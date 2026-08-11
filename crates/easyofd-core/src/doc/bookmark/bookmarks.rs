//! 书签集。

use super::Bookmark;

/// 对应 Java: org.ofdrw.core.pageDescription.Bookmarks
///
/// 书签集合，包含文档中所有书签定义。
#[derive(Debug, Clone)]
pub struct Bookmarks {
    /// 书签列表。
    pub bookmarks: Vec<Bookmark>,
}

impl Bookmarks {
    /// 创建空的书签集。
    #[must_use]
    pub fn new() -> Self {
        Self {
            bookmarks: Vec::new(),
        }
    }

    /// 添加一个书签。
    pub fn push(&mut self, bookmark: Bookmark) {
        self.bookmarks.push(bookmark);
    }

    /// 序列化为 XML 字符串。
    #[must_use]
    pub fn to_xml_string(&self) -> String {
        let inner: String = self.bookmarks.iter().map(Bookmark::to_xml_string).collect();
        format!("<Bookmarks>{inner}</Bookmarks>")
    }
}

impl Default for Bookmarks {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bookmarks_new() {
        let b = Bookmarks::new();
        assert!(b.bookmarks.is_empty());
        let b2 = Bookmarks::default();
        assert!(b2.bookmarks.is_empty());
    }

    #[test]
    fn test_bookmarks_push_and_xml() {
        let mut b = Bookmarks::new();
        b.push(Bookmark::new("Chapter 1", 1));
        b.push(Bookmark::new("Chapter 2", 5));
        assert_eq!(b.bookmarks.len(), 2);
        let xml = b.to_xml_string();
        assert!(xml.contains("<Bookmarks>"));
        assert!(xml.contains("</Bookmarks>"));
        assert!(xml.contains("Chapter 1"));
        assert!(xml.contains("Chapter 2"));
    }
}
