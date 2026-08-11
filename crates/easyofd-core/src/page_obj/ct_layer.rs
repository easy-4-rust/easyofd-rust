//! CT_Layer 图层。

use super::CT_PageBlock;

/// 对应 Java: org.ofdrw.core.basicStructure.pageObj.layer.CT_Layer
///
/// 图层类型，用于描述页面中的不同层（正文层、前景层、背景层）。
/// 继承自 CT_PageBlock，增加了图层类型和绘制参数引用。
/// 对应 GB/T 33190-2016 第 7.7 节。
#[allow(non_camel_case_types)]
#[derive(Debug, Clone)]
pub struct CT_Layer {
    /// 图层类型。
    pub layer_type: LayerType,
    /// 绘制参数引用 ID（可选）。
    pub draw_param: Option<u32>,
    /// 页面块内容。
    pub block: CT_PageBlock,
}

/// 图层类型枚举。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayerType {
    /// 正文层（默认）。
    Body,
    /// 前景层。
    Foreground,
    /// 背景层。
    Background,
}

impl LayerType {
    /// 转为 OFD XML 属性值。
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Body => "Body",
            Self::Foreground => "Foreground",
            Self::Background => "Background",
        }
    }
}

impl std::fmt::Display for LayerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl CT_Layer {
    /// 使用指定类型创建图层。
    #[must_use]
    pub fn new(layer_type: LayerType) -> Self {
        Self {
            layer_type,
            draw_param: None,
            block: CT_PageBlock::new(),
        }
    }

    /// 创建正文层。
    #[must_use]
    pub fn body() -> Self {
        Self::new(LayerType::Body)
    }

    /// 创建前景层。
    #[must_use]
    pub fn foreground() -> Self {
        Self::new(LayerType::Foreground)
    }

    /// 创建背景层。
    #[must_use]
    pub fn background() -> Self {
        Self::new(LayerType::Background)
    }

    /// 设置绘制参数引用。
    #[must_use]
    pub fn draw_param(mut self, id: u32) -> Self {
        self.draw_param = Some(id);
        self
    }

    /// 设置图层类型。
    #[must_use]
    pub fn layer_type(mut self, layer_type: LayerType) -> Self {
        self.layer_type = layer_type;
        self
    }

    /// 获取图层类型。
    #[must_use]
    pub fn get_type(&self) -> LayerType {
        self.layer_type
    }

    /// 获取绘制参数引用。
    #[must_use]
    pub fn get_draw_param(&self) -> Option<u32> {
        self.draw_param
    }

    /// 添加嵌套页面块。
    pub fn add_page_block(&mut self, page_block: CT_PageBlock) {
        self.block.add_page_block(page_block);
    }

    /// 序列化为 OFD XML 字符串。
    #[must_use]
    pub fn to_xml_string(&self) -> String {
        use std::fmt::Write;
        let mut xml = format!("<ofd:Layer Type=\"{}\"", self.layer_type.as_str());
        if let Some(dp) = self.draw_param {
            write!(xml, " DrawParam=\"{dp}\"").unwrap();
        }
        xml.push_str(">\n");
        // Inline the block content (skip outer PageBlock tags for layer).
        for text_obj in &self.block.text_objects {
            let _ = writeln!(
                xml,
                "  <ofd:TextObject ID=\"{}\" Boundary=\"{}\">{}</ofd:TextObject>",
                text_obj.id, text_obj.boundary, text_obj.content
            );
        }
        for path_obj in &self.block.path_objects {
            let _ = writeln!(
                xml,
                "  <ofd:PathObject ID=\"{}\" Boundary=\"{}\">\
                 <ofd:AbbreviatedData>{}</ofd:AbbreviatedData>\
                 </ofd:PathObject>",
                path_obj.id, path_obj.boundary, path_obj.abbreviated_data
            );
        }
        for img_obj in &self.block.image_objects {
            let _ = writeln!(
                xml,
                "  <ofd:ImageObject ID=\"{}\" Boundary=\"{}\" ResourceID=\"{}\" />",
                img_obj.id, img_obj.boundary, img_obj.resource_id
            );
        }
        for nested in &self.block.page_blocks {
            xml.push_str(&nested.to_xml_string());
        }
        xml.push_str("</ofd:Layer>\n");
        xml
    }
}

#[cfg(test)]
mod tests {
    use super::super::ct_page_block::{PageBlockImageObject, PageBlockTextObject};
    use super::*;

    #[test]
    fn test_ct_layer_body() {
        let layer = CT_Layer::body();
        assert_eq!(layer.layer_type, LayerType::Body);
        assert!(layer.draw_param.is_none());
    }

    #[test]
    fn test_ct_layer_foreground() {
        let layer = CT_Layer::foreground();
        assert_eq!(layer.layer_type, LayerType::Foreground);
    }

    #[test]
    fn test_ct_layer_background() {
        let layer = CT_Layer::background();
        assert_eq!(layer.layer_type, LayerType::Background);
    }

    #[test]
    fn test_ct_layer_builder() {
        let layer = CT_Layer::new(LayerType::Body).draw_param(42);
        assert_eq!(layer.get_draw_param(), Some(42));
    }

    #[test]
    fn test_layer_type_display() {
        assert_eq!(LayerType::Body.to_string(), "Body");
        assert_eq!(LayerType::Foreground.to_string(), "Foreground");
        assert_eq!(LayerType::Background.to_string(), "Background");
    }

    #[test]
    fn test_layer_type_as_str() {
        assert_eq!(LayerType::Body.as_str(), "Body");
        assert_eq!(LayerType::Foreground.as_str(), "Foreground");
        assert_eq!(LayerType::Background.as_str(), "Background");
    }

    #[test]
    fn test_ct_layer_to_xml_basic() {
        let layer = CT_Layer::body();
        let xml = layer.to_xml_string();
        assert!(xml.contains("<ofd:Layer"));
        assert!(xml.contains("Type=\"Body\""));
        assert!(xml.contains("</ofd:Layer>"));
    }

    #[test]
    fn test_ct_layer_to_xml_with_draw_param() {
        let layer = CT_Layer::foreground().draw_param(7);
        let xml = layer.to_xml_string();
        assert!(xml.contains("DrawParam=\"7\""));
        assert!(xml.contains("Type=\"Foreground\""));
    }

    #[test]
    fn test_ct_layer_to_xml_with_content() {
        let mut layer = CT_Layer::body();
        layer
            .block
            .add_text_object(PageBlockTextObject::new(1, "0 0 50 20", "hi"));
        layer
            .block
            .add_image_object(PageBlockImageObject::new(2, "0 0 100 100", 3));
        let xml = layer.to_xml_string();
        assert!(xml.contains("ofd:TextObject"));
        assert!(xml.contains("hi"));
        assert!(xml.contains("ofd:ImageObject"));
    }

    #[test]
    fn test_ct_layer_clone_debug() {
        let layer = CT_Layer::body();
        let layer2 = layer.clone();
        assert_eq!(layer2.layer_type, LayerType::Body);
        assert!(format!("{layer:?}").contains("CT_Layer"));
    }
}
