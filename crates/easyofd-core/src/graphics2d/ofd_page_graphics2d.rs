//! OFD 页面 2D 图形上下文。
//!
//! 对应 Java: org.ofdrw.graphics2d.OFDPageGraphics2D
//!
//! Java 版继承 `java.awt.Graphics2D`，提供 2D 绘图 API。
//! Rust 版提供简化结构，保留页面级绘图上下文状态。

use super::OfdGraphics2DDrawParam;

/// OFD 页面 2D 图形上下文。
///
/// 对应 Java: org.ofdrw.graphics2d.OFDPageGraphics2D
///
/// 绑定到单个 OFD 页面的绘图上下文，持有当前绘制参数和页面尺寸。
/// Java 版提供完整的 `Graphics2D` API（drawLine / fillRect / drawString 等）；
/// Rust 版保留上下文状态，具体绘制由上层引擎实现。
#[derive(Debug, Clone)]
pub struct OfdPageGraphics2D {
    /// 页面宽度（mm）。
    pub page_width: f64,
    /// 页面高度（mm）。
    pub page_height: f64,
    /// 当前绘制参数。
    pub draw_param: OfdGraphics2DDrawParam,
    /// 已绘制的对象计数。
    pub object_count: u32,
}

impl OfdPageGraphics2D {
    /// 创建新的页面图形上下文。
    #[must_use]
    pub fn new(page_width: f64, page_height: f64) -> Self {
        Self {
            page_width,
            page_height,
            draw_param: OfdGraphics2DDrawParam::new(),
            object_count: 0,
        }
    }

    /// 设置绘制参数。
    #[must_use]
    pub fn draw_param(mut self, param: OfdGraphics2DDrawParam) -> Self {
        self.draw_param = param;
        self
    }

    /// 获取页面宽度。
    #[must_use]
    pub fn page_width(&self) -> f64 {
        self.page_width
    }

    /// 获取页面高度。
    #[must_use]
    pub fn page_height(&self) -> f64 {
        self.page_height
    }

    /// 获取已绘制对象数。
    #[must_use]
    pub fn object_count(&self) -> u32 {
        self.object_count
    }

    /// 增加对象计数（模拟绘制操作）。
    pub fn increment_object_count(&mut self) {
        self.object_count += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let g = OfdPageGraphics2D::new(210.0, 297.0);
        assert!((g.page_width() - 210.0).abs() < f64::EPSILON);
        assert!((g.page_height() - 297.0).abs() < f64::EPSILON);
        assert_eq!(g.object_count(), 0);
    }

    #[test]
    fn test_draw_param() {
        let param = OfdGraphics2DDrawParam::new().line_width(3.0);
        let g = OfdPageGraphics2D::new(100.0, 100.0).draw_param(param);
        assert!((g.draw_param.line_width - 3.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_increment_object_count() {
        let mut g = OfdPageGraphics2D::new(100.0, 100.0);
        g.increment_object_count();
        g.increment_object_count();
        assert_eq!(g.object_count(), 2);
    }

    #[test]
    fn test_clone_debug() {
        let g = OfdPageGraphics2D::new(100.0, 200.0);
        let g2 = g.clone();
        assert!((g2.page_width - 100.0).abs() < f64::EPSILON);
        assert!(format!("{g:?}").contains("OfdPageGraphics2D"));
    }
}
