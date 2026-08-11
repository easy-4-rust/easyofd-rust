//! 多媒体资源列表。
//!
//! 对应 Java: org.ofdrw.core.basicStructure.res.resources.MultiMedias

use super::CT_MultiMedia;

/// 多媒体资源列表。
///
/// 对应 Java: org.ofdrw.core.basicStructure.res.resources.MultiMedias
#[derive(Debug, Clone, Default)]
pub struct MultiMedias {
    /// 多媒体资源列表。
    pub items: Vec<CT_MultiMedia>,
}

impl MultiMedias {
    /// 创建空多媒体列表。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 添加多媒体资源。
    pub fn add(&mut self, item: CT_MultiMedia) {
        self.items.push(item);
    }

    /// 获取数量。
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::page_description::res::MediaType;

    #[test]
    fn multi_medias_new() {
        let mm = MultiMedias::new();
        assert!(mm.is_empty());
    }

    #[test]
    fn multi_medias_add() {
        let mut mm = MultiMedias::new();
        mm.add(CT_MultiMedia::new(1, MediaType::Image));
        assert_eq!(mm.len(), 1);
    }
}
