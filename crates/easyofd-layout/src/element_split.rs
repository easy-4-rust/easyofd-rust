//! 元素分割接口。
//!
//! 对应 Java: org.ofdrw.layout.engine.ElementSplit

use crate::div::Div;
use crate::rectangle::Rectangle;

/// 元素分割接口，用于将一个 Div 分割为多个可布局的部分。
///
/// 对应 Java: ofdrw layout engine ElementSplit（interface）。
pub trait ElementSplit {
    /// 判断元素是否可以在指定区域内分割。
    fn can_split(&self, div: &Div, area: &Rectangle) -> bool;

    /// 将元素按指定区域进行分割。
    ///
    /// 返回 `(当前页部分, 剩余部分)` 元组。若不可分割则剩余部分为 `None`。
    ///
    /// # Errors
    ///
    /// 分割失败时返回错误描述。
    fn split(&self, div: &Div, area: &Rectangle) -> Result<(Div, Option<Div>), String>;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct SimpleSplitter;

    impl ElementSplit for SimpleSplitter {
        fn can_split(&self, _div: &Div, _area: &Rectangle) -> bool {
            true
        }

        fn split(&self, div: &Div, _area: &Rectangle) -> Result<(Div, Option<Div>), String> {
            Ok((div.clone(), None))
        }
    }

    #[test]
    fn test_element_split() {
        let splitter = SimpleSplitter;
        let div = Div::from_text_object(&easyofd_core::TextObject::new(0.0, 0.0, "hello world"));
        let area = Rectangle::from_size(100.0, 50.0);
        assert!(splitter.can_split(&div, &area));
        let (current, rest) = splitter.split(&div, &area).unwrap();
        assert!(current.text_content().is_some());
        assert!(rest.is_none());
    }
}
