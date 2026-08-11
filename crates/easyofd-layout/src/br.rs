//! 换行符元素。
//!
//! 对应 Java: org.ofdrw.layout.element.BR
//!
//! 解析器在解析到该元素时会自动换行，结束当前 segment。
//! 仅在流式布局中有效。

use crate::Div;

/// 换行符元素，继承 Div 盒式模型。
///
/// 对应 Java: ofdrw layout BR，宽度和高度均为 0。
#[derive(Debug, Clone)]
pub struct BR {
    /// 内部 Div（宽高为 0）。
    inner: Div,
}

impl Default for BR {
    fn default() -> Self {
        Self::new()
    }
}

impl BR {
    /// 创建换行符（对应 Java: BR()）。
    #[must_use]
    pub fn new() -> Self {
        let inner = Div {
            width: 0.0,
            height: 0.0,
            x: 0.0,
            y: 0.0,
            padding: 0.0,
            border: 0.0,
            margin: 0.0,
            background: None,
            content: crate::DivContent::Text(
                String::new(),
                crate::div::TextStyle {
                    font: String::new(),
                    size: 0.0,
                    weight: 400,
                    italic: false,
                    color: 0,
                },
            ),
        };
        Self { inner }
    }

    /// 获取内部 Div 引用。
    #[must_use]
    pub fn as_div(&self) -> &Div {
        &self.inner
    }

    /// 获取内部 Div 可变引用。
    pub fn as_div_mut(&mut self) -> &mut Div {
        &mut self.inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_br_default() {
        let br = BR::new();
        assert!((br.as_div().width - 0.0).abs() < f64::EPSILON);
        assert!((br.as_div().height - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_br_as_div() {
        let br = BR::new();
        let div = br.as_div();
        assert!((div.x - 0.0).abs() < f64::EPSILON);
        assert!((div.y - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_br_as_div_mut() {
        let mut br = BR::new();
        br.as_div_mut().x = 10.0;
        assert!((br.as_div().x - 10.0).abs() < f64::EPSILON);
    }
}
