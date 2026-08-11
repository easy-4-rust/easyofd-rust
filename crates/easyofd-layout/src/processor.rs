//! 渲染处理器接口。
//!
//! 对应 Java: org.ofdrw.layout.engine.render.Processor

use crate::div::Div;
use crate::rectangle::Rectangle;

/// 渲染处理器接口，负责将 Div 渲染到指定区域。
///
/// 对应 Java: ofdrw layout engine render Processor（interface）。
pub trait Processor {
    /// 处理器名称（用于日志和调试）。
    fn name(&self) -> &str;

    /// 判断是否能处理指定的 Div。
    fn can_process(&self, div: &Div) -> bool;

    /// 处理 Div，将其渲染到指定区域。
    ///
    /// # Errors
    ///
    /// 渲染失败时返回错误描述。
    fn process(&self, div: &Div, area: &Rectangle) -> Result<(), String>;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TextProcessor;

    impl Processor for TextProcessor {
        fn name(&self) -> &'static str {
            "TextProcessor"
        }

        fn can_process(&self, div: &Div) -> bool {
            div.text_content().is_some()
        }

        fn process(&self, _div: &Div, _area: &Rectangle) -> Result<(), String> {
            Ok(())
        }
    }

    #[test]
    fn test_processor_name() {
        let p = TextProcessor;
        assert_eq!(p.name(), "TextProcessor");
    }

    #[test]
    fn test_processor_can_process() {
        let p = TextProcessor;
        let text_div = Div::from_text_object(&easyofd_core::TextObject::new(0.0, 0.0, "hello"));
        assert!(p.can_process(&text_div));
    }

    #[test]
    fn test_processor_process() {
        let p = TextProcessor;
        let div = Div::from_text_object(&easyofd_core::TextObject::new(0.0, 0.0, "test"));
        let area = Rectangle::from_size(100.0, 50.0);
        assert!(p.process(&div, &area).is_ok());
    }
}
