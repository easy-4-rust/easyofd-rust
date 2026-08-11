//! 元素区域（ofd:Area）。
//!
//! 对应 Java: org.ofdrw.core.basicStructure.pageObj.layer.area.Area

use crate::basic_type::ST_Array;
use crate::xml_element::{XmlElement, XmlElementError, XmlNode};

/// 元素区域（ofd:Area），定义绘制参数、变换矩阵与裁剪。
///
/// 对应 Java: ofdrw Area。
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Area {
    /// 绘制参数引用（ST_RefID，可选）。
    pub draw_param: Option<u32>,
    /// 变换矩阵（ST_Array，6 元素 a b c d e f，可选）。
    pub ctm: Option<ST_Array>,
    /// 裁剪区域（原始 XML 内容，可选）。
    pub clip: Option<String>,
}

impl Area {
    /// 创建空区域（对应 Java: Area()）。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置绘制参数引用（对应 Java: Area#setDrawParam）。
    #[must_use]
    pub fn draw_param(mut self, id: u32) -> Self {
        self.draw_param = Some(id);
        self
    }

    /// 设置变换矩阵（对应 Java: Area#setCTM）。
    #[must_use]
    pub fn ctm(mut self, ctm: ST_Array) -> Self {
        self.ctm = Some(ctm);
        self
    }

    /// 设置裁剪区域（对应 Java: Area#setClipObj）。
    #[must_use]
    pub fn clip(mut self, clip: impl Into<String>) -> Self {
        self.clip = Some(clip.into());
        self
    }
}

impl XmlElement for Area {
    /// 对应 Java: Area 元素名 "Area"。
    fn element_name(&self) -> &'static str {
        "Area"
    }

    fn attributes(&self) -> Vec<(String, String)> {
        let mut attrs = Vec::new();
        if let Some(dp) = self.draw_param {
            attrs.push(("DrawParam".to_string(), dp.to_string()));
        }
        if let Some(ref ctm) = self.ctm {
            attrs.push(("CTM".to_string(), ctm.to_xml_string()));
        }
        attrs
    }

    fn child_nodes(&self) -> Vec<XmlNode> {
        let mut children = Vec::new();
        if let Some(ref clip) = self.clip {
            let mut clip_node = XmlNode::element("Clip");
            clip_node.push_child(XmlNode::text_node(clip.clone()));
            children.push(clip_node);
        }
        children
    }

    fn from_xml(node: &XmlNode) -> Result<Self, XmlElementError> {
        let draw_param = node
            .get_attr("DrawParam")
            .map(|s| {
                s.parse::<u32>()
                    .map_err(|e| XmlElementError(format!("解析 Area.DrawParam 失败: {e}")))
            })
            .transpose()?;
        let ctm = node
            .get_attr("CTM")
            .map(|s| {
                ST_Array::from_str(s)
                    .map_err(|e| XmlElementError(format!("解析 Area.CTM 失败: {e}")))
            })
            .transpose()?;
        let clip = node.child("Clip").and_then(|c| c.text.clone());
        Ok(Self {
            draw_param,
            ctm,
            clip,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::xml_parse::parse_xml_to_nodes;

    #[test]
    fn test_area_default() {
        let a = Area::new();
        assert!(a.draw_param.is_none());
        assert!(a.ctm.is_none());
    }

    #[test]
    fn test_area_builders() {
        let a = Area::new()
            .draw_param(5)
            .ctm(ST_Array::transform(1.0, 0.0, 0.0, 1.0, 0.0, 0.0))
            .clip("M0 0 L10 0 L10 10 Z");
        assert_eq!(a.draw_param, Some(5));
        assert!(a.ctm.is_some());
        assert_eq!(a.clip.as_deref(), Some("M0 0 L10 0 L10 10 Z"));
    }

    #[test]
    fn test_xml_element_name() {
        let a = Area::new();
        assert_eq!(a.element_name(), "Area");
    }

    #[test]
    fn test_xml_element_to_xml_contains_attrs() {
        let a = Area::new()
            .draw_param(3)
            .ctm(ST_Array::transform(1.0, 0.0, 0.0, 1.0, 10.0, 20.0));
        let xml = a.to_xml();
        assert!(xml.contains("DrawParam=\"3\""));
        assert!(xml.contains("CTM=\""));
    }

    #[test]
    fn test_xml_element_roundtrip_full() {
        let a = Area::new()
            .draw_param(5)
            .ctm(ST_Array::transform(1.0, 0.0, 0.0, 1.0, 10.0, 20.0))
            .clip("M0 0 L10 0 L10 10 Z");
        let xml = a.to_xml();
        let node = parse_xml_to_nodes(&xml).unwrap();
        let a2 = Area::from_xml(&node).unwrap();
        assert_eq!(a, a2);
    }

    #[test]
    fn test_xml_element_roundtrip_empty() {
        let a = Area::new();
        let xml = a.to_xml();
        assert_eq!(xml, "<Area/>");
        let node = parse_xml_to_nodes(&xml).unwrap();
        let a2 = Area::from_xml(&node).unwrap();
        assert_eq!(a, a2);
    }
}
