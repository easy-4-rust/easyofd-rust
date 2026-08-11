//! Div 容器接口。
//!
//! 对应 Java: org.ofdrw.layout.element.DivContainer

use crate::div::Div;

/// Div 容器接口，表示可以容纳子 Div 元素的容器。
///
/// 对应 Java: ofdrw layout DivContainer（interface）。
pub trait DivContainer {
    /// 获取子元素列表。
    fn children(&self) -> &[Div];

    /// 获取子元素可变引用。
    fn children_mut(&mut self) -> &mut Vec<Div>;

    /// 添加子元素。
    fn push(&mut self, child: Div) {
        self.children_mut().push(child);
    }

    /// 子元素数量。
    fn child_count(&self) -> usize {
        self.children().len()
    }

    /// 是否为空容器。
    fn is_empty(&self) -> bool {
        self.children().is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct SimpleContainer {
        items: Vec<Div>,
    }

    impl SimpleContainer {
        fn new() -> Self {
            Self { items: Vec::new() }
        }
    }

    impl DivContainer for SimpleContainer {
        fn children(&self) -> &[Div] {
            &self.items
        }
        fn children_mut(&mut self) -> &mut Vec<Div> {
            &mut self.items
        }
    }

    #[test]
    fn test_empty_container() {
        let c = SimpleContainer::new();
        assert!(c.is_empty());
        assert_eq!(c.child_count(), 0);
    }

    #[test]
    fn test_push_child() {
        let mut c = SimpleContainer::new();
        let div = Div::from_text_object(&easyofd_core::TextObject::new(0.0, 0.0, "test"));
        c.push(div);
        assert_eq!(c.child_count(), 1);
        assert!(!c.is_empty());
    }
}
