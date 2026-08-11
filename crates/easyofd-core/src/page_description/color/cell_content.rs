//! 单元格内容。
//!
//! 对应 Java: org.ofdrw.core.pageDescription.color.pattern.CellContent

use crate::page_description::ct_color::CT_Color;

/// 单元格内容，用于图案填充中定义单元格内的绘制内容。
///
/// 对应 Java: org.ofdrw.core.pageDescription.color.pattern.CellContent
///
/// 在 CT_Pattern 的 CellContent 中定义每个图案单元格内
/// 的绘制内容（路径、文本、图像等）和颜色信息。
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Default)]
pub struct CellContent {
    /// 单元格宽度（mm）。
    pub width: Option<f64>,
    /// 单元格高度（mm）。
    pub height: Option<f64>,
    /// 填充颜色。
    pub fill_color: Option<CT_Color>,
    /// 描边颜色。
    pub stroke_color: Option<CT_Color>,
}

impl CellContent {
    /// 创建空单元格内容。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置宽度。
    #[must_use]
    pub fn width(mut self, width: f64) -> Self {
        self.width = Some(width);
        self
    }

    /// 设置高度。
    #[must_use]
    pub fn height(mut self, height: f64) -> Self {
        self.height = Some(height);
        self
    }

    /// 设置填充颜色。
    #[must_use]
    pub fn fill_color(mut self, color: CT_Color) -> Self {
        self.fill_color = Some(color);
        self
    }

    /// 设置描边颜色。
    #[must_use]
    pub fn stroke_color(mut self, color: CT_Color) -> Self {
        self.stroke_color = Some(color);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cell_content_new() {
        let cc = CellContent::new();
        assert!(cc.width.is_none());
        assert!(cc.height.is_none());
    }

    #[test]
    fn test_cell_content_builder() {
        let cc = CellContent::new()
            .width(10.0)
            .height(20.0);
        assert_eq!(cc.width, Some(10.0));
        assert_eq!(cc.height, Some(20.0));
    }

    #[test]
    fn test_cell_content_clone_debug() {
        let cc = CellContent::new().width(5.0);
        let cc2 = cc.clone();
        assert_eq!(cc2.width, Some(5.0));
        assert!(format!("{cc:?}").contains("CellContent"));
    }
}
