//! 附加虚拟页面。
//!
//! 对应 Java: org.ofdrw.layout.edit.AdditionVPage

use crate::rectangle::Rectangle;

/// 附加虚拟页面，表示在文档编辑阶段追加的页面。
///
/// 对应 Java: ofdrw layout edit AdditionVPage。
#[derive(Debug, Clone, PartialEq)]
pub struct AdditionVPage {
    /// 页面索引（从 0 开始）。
    pub page_index: u32,
    /// 页面可布局区域。
    pub area: Rectangle,
    /// 页面内容描述（可选）。
    pub content_description: Option<String>,
}

impl AdditionVPage {
    /// 创建附加虚拟页面。
    #[must_use]
    pub fn new(page_index: u32, area: Rectangle) -> Self {
        Self {
            page_index,
            area,
            content_description: None,
        }
    }

    /// 设置内容描述。
    #[must_use]
    pub fn content_description(mut self, desc: impl Into<String>) -> Self {
        self.content_description = Some(desc.into());
        self
    }

    /// 页面面积（mm^2）。
    #[must_use]
    pub fn area_size(&self) -> f64 {
        self.area.width * self.area.height
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let page = AdditionVPage::new(0, Rectangle::new(0.0, 0.0, 210.0, 297.0));
        assert_eq!(page.page_index, 0);
        assert!((page.area.width - 210.0).abs() < f64::EPSILON);
        assert!(page.content_description.is_none());
    }

    #[test]
    fn test_content_description() {
        let page =
            AdditionVPage::new(1, Rectangle::from_size(100.0, 100.0)).content_description("附加页");
        assert_eq!(page.content_description.as_deref(), Some("附加页"));
    }

    #[test]
    fn test_area_size() {
        let page = AdditionVPage::new(0, Rectangle::from_size(100.0, 50.0));
        assert!((page.area_size() - 5000.0).abs() < f64::EPSILON);
    }
}
