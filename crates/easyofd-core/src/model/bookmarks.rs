//! OFD 书签集合。
//!
//! 对应 Java: org.ofdrw.core.basicStructure.outline.CT_Outline

use crate::model::bookmark::Bookmark;

/// 书签集合（ofd:Bookmarks / ofd:Outline）。
///
/// 对应 Java: ofdrw CT_Outline。
#[derive(Debug, Clone, PartialEq)]
pub struct Bookmarks {
    /// 书签列表。
    pub items: Vec<Bookmark>,
}

impl Bookmarks {
    /// 创建空书签集合。
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    /// 添加书签。
    pub fn push(&mut self, bookmark: Bookmark) {
        self.items.push(bookmark);
    }

    /// 书签数量。
    #[must_use]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// 是否为空。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

impl Default for Bookmarks {
    fn default() -> Self {
        Self::new()
    }
}
