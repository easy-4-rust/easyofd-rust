//! OFD 图形文档。
//!
//! 对应 Java: org.ofdrw.graphics2d.OFDGraphicsDocument

use easyofd_core::OfdPage;

/// OFD 图形文档，用于通过 2D 图形 API 创建 OFD 文档。
///
/// 对应 Java: org.ofdrw.graphics2d.OFDGraphicsDocument
///
/// 提供类似 Java Graphics2D 的 API 来创建 OFD 页面内容，
/// 内部使用 Canvas 录制绘图命令，最终转换为 OFD 页面。
#[derive(Debug)]
pub struct OfdGraphicsDocument {
    /// 文档宽度（mm）。
    pub width: f64,
    /// 文档高度（mm）。
    pub height: f64,
    /// 页面列表。
    pub pages: Vec<OfdPage>,
}

impl OfdGraphicsDocument {
    /// 创建新的图形文档。
    #[must_use]
    pub fn new(width: f64, height: f64) -> Self {
        Self {
            width,
            height,
            pages: Vec::new(),
        }
    }

    /// 添加新页面并返回页面图形上下文。
    #[must_use]
    pub fn add_page(&mut self) -> &mut OfdPage {
        let page = OfdPage::new(self.width, self.height);
        self.pages.push(page);
        self.pages.last_mut().unwrap()
    }

    /// 获取页面数量。
    #[must_use]
    pub fn page_count(&self) -> usize {
        self.pages.len()
    }

    /// 获取所有页面。
    #[must_use]
    pub fn pages(&self) -> &[OfdPage] {
        &self.pages
    }
}

/// 资源管理器。
///
/// 对应 Java: org.ofdrw.graphics2d.ResManager
///
/// 管理 OFD 文档中的资源（字体、图片等）。
#[derive(Debug, Default)]
pub struct ResManager {
    /// 字体资源 ID 计数器。
    next_font_id: u32,
    /// 图片资源 ID 计数器。
    next_image_id: u32,
}

impl ResManager {
    /// 创建新的资源管理器。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 分配下一个字体资源 ID。
    #[must_use]
    pub fn next_font_id(&mut self) -> u32 {
        let id = self.next_font_id;
        self.next_font_id += 1;
        id
    }

    /// 分配下一个图片资源 ID。
    #[must_use]
    pub fn next_image_id(&mut self) -> u32 {
        let id = self.next_image_id;
        self.next_image_id += 1;
        id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ofd_graphics_document_new() {
        let doc = OfdGraphicsDocument::new(210.0, 297.0);
        assert_eq!(doc.page_count(), 0);
    }

    #[test]
    fn test_ofd_graphics_document_add_page() {
        let mut doc = OfdGraphicsDocument::new(210.0, 297.0);
        doc.add_page();
        assert_eq!(doc.page_count(), 1);
    }

    #[test]
    fn test_ofd_graphics_document_pages() {
        let mut doc = OfdGraphicsDocument::new(210.0, 297.0);
        doc.add_page();
        doc.add_page();
        assert_eq!(doc.pages().len(), 2);
    }

    #[test]
    fn test_res_manager_new() {
        let mut rm = ResManager::new();
        assert_eq!(rm.next_font_id(), 0);
        assert_eq!(rm.next_font_id(), 1);
        assert_eq!(rm.next_image_id(), 0);
    }
}
