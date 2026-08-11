//! 画布单元格。
//!
//! 对应 Java: org.ofdrw.layout.element.canvas.Cell

/// 画布单元格，用于表格布局中的单元格定义。
///
/// 对应 Java: org.ofdrw.layout.element.canvas.Cell
///
/// 在表格布局中，Cell 定义了单元格的内容、尺寸和样式属性。
#[derive(Debug, Clone)]
pub struct Cell {
    /// 单元格宽度（mm）。
    pub width: f64,
    /// 单元格高度（mm）。
    pub height: f64,
    /// 单元格内容文本。
    pub text: Option<String>,
    /// 跨列数。
    pub col_span: u32,
    /// 跨行数。
    pub row_span: u32,
}

impl Cell {
    /// 创建新的单元格。
    #[must_use]
    pub fn new(width: f64, height: f64) -> Self {
        Self {
            width,
            height,
            text: None,
            col_span: 1,
            row_span: 1,
        }
    }

    /// 设置单元格文本内容。
    #[must_use]
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self
    }

    /// 设置跨列数。
    #[must_use]
    pub fn col_span(mut self, span: u32) -> Self {
        self.col_span = span;
        self
    }

    /// 设置跨行数。
    #[must_use]
    pub fn row_span(mut self, span: u32) -> Self {
        self.row_span = span;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cell_new() {
        let cell = Cell::new(50.0, 20.0);
        assert_eq!(cell.width, 50.0);
        assert_eq!(cell.height, 20.0);
        assert!(cell.text.is_none());
    }

    #[test]
    fn test_cell_builder() {
        let cell = Cell::new(100.0, 30.0)
            .text("Hello")
            .col_span(2)
            .row_span(3);
        assert_eq!(cell.text.unwrap(), "Hello");
        assert_eq!(cell.col_span, 2);
        assert_eq!(cell.row_span, 3);
    }
}
