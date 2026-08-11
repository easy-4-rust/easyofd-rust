//! 区域占位区块。
//!
//! 对应 Java: org.ofdrw.layout.element.AreaHolderBlock
//!
//! 用于构造页面中一个用于容纳将来可能出现的页面元素的结构，
//! 该结构不做任何事情仅仅是占位。

use crate::div::{Div, DivContent, TextStyle};

/// 区域占位区块。
///
/// 对应 Java: ofdrw layout AreaHolderBlock（extends Div）。
///
/// 属性与 Div 一致，可绘制边框等内容。用于唯一定位区域的占位块。
#[derive(Debug, Clone)]
pub struct AreaHolderBlock {
    /// 占位区域名称，在文档范围内唯一。
    pub area_name: String,
    /// 内部 Div 盒式模型。
    pub div: Div,
}

impl AreaHolderBlock {
    /// 创建区域占位区块（对应 Java: AreaHolderBlock(areaName, width, height)）。
    #[must_use]
    pub fn new(area_name: impl Into<String>, width: f64, height: f64) -> Self {
        Self {
            area_name: area_name.into(),
            div: Div {
                width,
                height,
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

    /// 创建带位置的区域占位区块（对应 Java: AreaHolderBlock(areaName, x, y, w, h)）。
    #[must_use]
    pub fn with_position(
        area_name: impl Into<String>,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
    ) -> Self {
        Self {
            area_name: area_name.into(),
            div: Div {
                width,
                height,
                x,
                y,
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

    /// 获取内部 Div 引用。
    #[must_use]
    pub fn as_div(&self) -> &Div {
        &self.div
    }

    /// 获取内部 Div 可变引用。
    pub fn as_div_mut(&mut self) -> &mut Div {
        &mut self.div
    }

    /// 元素类型名称（对应 Java: AreaHolderBlock#elementType）。
    #[must_use]
    pub fn element_type(&self) -> &'static str {
        "AreaHolderBlock"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let block = AreaHolderBlock::new("header", 100.0, 20.0);
        assert_eq!(block.area_name, "header");
        assert!((block.div.width - 100.0).abs() < f64::EPSILON);
        assert!((block.div.height - 20.0).abs() < f64::EPSILON);
        assert_eq!(block.element_type(), "AreaHolderBlock");
    }

    #[test]
    fn test_with_position() {
        let block = AreaHolderBlock::with_position("footer", 10.0, 20.0, 200.0, 30.0);
        assert_eq!(block.area_name, "footer");
        assert!((block.div.x - 10.0).abs() < f64::EPSILON);
        assert!((block.div.y - 20.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_as_div() {
        let block = AreaHolderBlock::new("test", 50.0, 50.0);
        let div = block.as_div();
        assert!((div.width - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_as_div_mut() {
        let mut block = AreaHolderBlock::new("test", 50.0, 50.0);
        block.as_div_mut().x = 15.0;
        assert!((block.div.x - 15.0).abs() < f64::EPSILON);
    }
}
