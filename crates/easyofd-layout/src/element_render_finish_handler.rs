//! 元素渲染完成回调接口。
//!
//! 对应 Java: org.ofdrw.layout.handler.ElementRenderFinishHandler

use crate::div::Div;

/// 元素渲染完成回调接口。
///
/// 对应 Java: ofdrw layout handler ElementRenderFinishHandler（interface）。
pub trait ElementRenderFinishHandler: Send + Sync {
    /// 元素渲染完成时的回调。
    ///
    /// # Arguments
    ///
    /// * `page_index` - 元素所在页面索引。
    /// * `div` - 被渲染的 Div 元素。
    fn on_element_render_finish(&self, page_index: u32, div: &Div);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    struct TestHandler {
        count: AtomicU32,
    }

    impl TestHandler {
        fn new() -> Self {
            Self {
                count: AtomicU32::new(0),
            }
        }
    }

    impl ElementRenderFinishHandler for TestHandler {
        fn on_element_render_finish(&self, _page_index: u32, _div: &Div) {
            self.count.fetch_add(1, Ordering::Relaxed);
        }
    }

    #[test]
    fn test_element_render_finish_handler() {
        let handler = TestHandler::new();
        let div = Div::from_text_object(&easyofd_core::TextObject::new(0.0, 0.0, "test"));
        handler.on_element_render_finish(0, &div);
        handler.on_element_render_finish(1, &div);
        assert_eq!(handler.count.load(Ordering::Relaxed), 2);
    }
}
