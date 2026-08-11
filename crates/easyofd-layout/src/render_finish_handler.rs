//! 渲染完成回调接口。
//!
//! 对应 Java: org.ofdrw.layout.handler.RenderFinishHandler

/// 渲染完成回调接口。
///
/// 对应 Java: ofdrw layout handler RenderFinishHandler（interface）。
pub trait RenderFinishHandler {
    /// 渲染完成时的回调。
    ///
    /// # Arguments
    ///
    /// * `page_index` - 页面索引。
    /// * `page_count` - 总页面数。
    fn on_render_finish(&self, page_index: u32, page_count: u32);
}

/// 闭包形式的渲染完成回调。
pub struct FnRenderFinishHandler<F>(pub F)
where
    F: Fn(u32, u32);

impl<F> RenderFinishHandler for FnRenderFinishHandler<F>
where
    F: Fn(u32, u32),
{
    fn on_render_finish(&self, page_index: u32, page_count: u32) {
        (self.0)(page_index, page_count);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    struct TestHandler {
        last_page: Cell<(u32, u32)>,
    }

    impl TestHandler {
        fn new() -> Self {
            Self {
                last_page: Cell::new((0, 0)),
            }
        }
    }

    impl RenderFinishHandler for TestHandler {
        fn on_render_finish(&self, page_index: u32, page_count: u32) {
            self.last_page.set((page_index, page_count));
        }
    }

    #[test]
    fn test_handler() {
        let handler = TestHandler::new();
        handler.on_render_finish(2, 10);
        assert_eq!(handler.last_page.get(), (2, 10));
    }

    #[test]
    fn test_fn_handler() {
        use std::cell::RefCell;
        let captured = RefCell::new(Vec::new());
        let handler = FnRenderFinishHandler(|idx, total| {
            captured.borrow_mut().push((idx, total));
        });
        handler.on_render_finish(0, 3);
        handler.on_render_finish(1, 3);
        assert_eq!(*captured.borrow(), vec![(0, 3), (1, 3)]);
    }
}
