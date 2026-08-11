//! 绘制上下文。
//!
//! 对应 Java: org.ofdrw.layout.element.canvas.DrawContext

use easyofd_core::OfdPage;

/// 绘制上下文，封装绘图操作的环境信息。
///
/// 对应 Java: org.ofdrw.layout.element.canvas.DrawContext
///
/// 在绘制过程中，DrawContext 提供当前页面、坐标系变换、
/// 绘制参数等上下文信息，供绘制器使用。
#[derive(Debug)]
pub struct DrawContext<'a> {
    /// 当前页面引用。
    pub page: &'a mut OfdPage,
    /// 当前变换矩阵 [a, b, c, d, e, f]。
    pub transform: [f64; 6],
    /// 当前绘制参数 ID。
    pub draw_param_id: Option<u32>,
}

impl<'a> DrawContext<'a> {
    /// 创建新的绘制上下文。
    #[must_use]
    pub fn new(page: &'a mut OfdPage) -> Self {
        Self {
            page,
            transform: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
            draw_param_id: None,
        }
    }

    /// 设置变换矩阵。
    #[must_use]
    pub fn transform(mut self, transform: [f64; 6]) -> Self {
        self.transform = transform;
        self
    }

    /// 设置绘制参数 ID。
    #[must_use]
    pub fn draw_param_id(mut self, id: u32) -> Self {
        self.draw_param_id = Some(id);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_draw_context_new() {
        let mut page = OfdPage::new(210.0, 297.0);
        let ctx = DrawContext::new(&mut page);
        assert_eq!(ctx.transform, [1.0, 0.0, 0.0, 1.0, 0.0, 0.0]);
        assert!(ctx.draw_param_id.is_none());
    }

    #[test]
    fn test_draw_context_builder() {
        let mut page = OfdPage::new(210.0, 297.0);
        let ctx = DrawContext::new(&mut page)
            .draw_param_id(5);
        assert_eq!(ctx.draw_param_id, Some(5));
    }
}
