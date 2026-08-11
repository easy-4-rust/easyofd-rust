//! 页面剩余空间填充元素。
//!
//! 对应 Java: org.ofdrw.layout.element.PageAreaFiller
//!
//! 该元素不会被布局分析器解析，只是作为一个命令标志，
//! 告诉分析器使剩余的空间为 0，也就是构造一个特殊的段。

use crate::a_float::AFloat;
use crate::clear::Clear;
use crate::div::{Div, DivContent, TextStyle};

/// 页面剩余空间填充元素。
///
/// 对应 Java: ofdrw layout PageAreaFiller（extends Div）。
#[derive(Debug, Clone)]
pub struct PageAreaFiller {
    /// 内部 Div。
    pub div: Div,
}

impl Default for PageAreaFiller {
    fn default() -> Self {
        Self::new()
    }
}

impl PageAreaFiller {
    /// 创建页面剩余空间填充元素（对应 Java: PageAreaFiller()）。
    ///
    /// 默认设置为占位符、清除两侧浮动、居中浮动。
    #[must_use]
    pub fn new() -> Self {
        Self {
            div: Div {
                width: 0.0,
                height: 0.0,
                x: 0.0,
                y: 0.0,
                padding: 0.0,
                border: 0.0,
                margin: 0.0,
                background: None,
                content: DivContent::Text(
                    String::new(),
                    TextStyle {
                        font: String::new(),
                        size: 0.0,
                        weight: 400,
                        italic: false,
                        color: 0,
                    },
                ),
            },
        }
    }

    /// 获取清除方式。
    #[must_use]
    pub fn clear(&self) -> Clear {
        Clear::Both
    }

    /// 获取浮动方式。
    #[must_use]
    pub fn float_mode(&self) -> AFloat {
        AFloat::Center
    }

    /// 是否为占位符。
    #[must_use]
    pub fn is_placeholder(&self) -> bool {
        true
    }

    /// 获取内部 Div 引用。
    #[must_use]
    pub fn as_div(&self) -> &Div {
        &self.div
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let filler = PageAreaFiller::new();
        assert!(filler.is_placeholder());
        assert_eq!(filler.clear(), Clear::Both);
        assert_eq!(filler.float_mode(), AFloat::Center);
    }

    #[test]
    fn test_as_div() {
        let filler = PageAreaFiller::new();
        let div = filler.as_div();
        assert!((div.width - 0.0).abs() < f64::EPSILON);
        assert!((div.height - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_default() {
        let filler = PageAreaFiller::default();
        assert!(filler.is_placeholder());
    }
}
