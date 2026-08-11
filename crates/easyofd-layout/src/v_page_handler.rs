//! 虚拟页面处理回调接口。
//!
//! 对应 Java: org.ofdrw.layout.handler.VPageHandler

use crate::rectangle::Rectangle;

/// 虚拟页面处理回调接口。
///
/// 对应 Java: ofdrw layout handler VPageHandler（interface）。
pub trait VPageHandler {
    /// 页面创建时的回调。
    ///
    /// # Arguments
    ///
    /// * `page_index` - 页面索引（从 0 开始）。
    /// * `page_area` - 页面可布局区域。
    fn on_page_created(&self, page_index: u32, page_area: &Rectangle);

    /// 页面渲染完成时的回调。
    ///
    /// # Arguments
    ///
    /// * `page_index` - 页面索引（从 0 开始）。
    fn on_page_finished(&self, page_index: u32);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    struct TestPageHandler {
        events: RefCell<Vec<String>>,
    }

    impl TestPageHandler {
        fn new() -> Self {
            Self {
                events: RefCell::new(Vec::new()),
            }
        }
    }

    impl VPageHandler for TestPageHandler {
        fn on_page_created(&self, page_index: u32, _page_area: &Rectangle) {
            self.events
                .borrow_mut()
                .push(format!("created:{page_index}"));
        }

        fn on_page_finished(&self, page_index: u32) {
            self.events
                .borrow_mut()
                .push(format!("finished:{page_index}"));
        }
    }

    #[test]
    fn test_page_handler() {
        let handler = TestPageHandler::new();
        let area = Rectangle::from_size(210.0, 297.0);
        handler.on_page_created(0, &area);
        handler.on_page_finished(0);
        handler.on_page_created(1, &area);
        handler.on_page_finished(1);
        assert_eq!(
            handler.events.borrow().clone(),
            vec!["created:0", "finished:0", "created:1", "finished:1"]
        );
    }
}
