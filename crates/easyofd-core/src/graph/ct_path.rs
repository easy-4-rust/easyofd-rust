//! CT_Path 路径对象。

use super::AbbreviatedData;
use crate::xml_element::{XmlElement, XmlElementError, XmlNode};

/// 填充规则。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FillRule {
    /// 非零绕组规则（默认）。
    NonZero,
    /// 奇偶规则。
    EvenOdd,
}

impl FillRule {
    /// 转为 OFD XML 属性值。
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::NonZero => "NonZero",
            Self::EvenOdd => "EvenOdd",
        }
    }
}

impl std::fmt::Display for FillRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// 对应 Java: org.ofdrw.core.graph.pathObj.CT_Path
///
/// 路径对象，扩展自图形单元，包含填充规则、线宽、线帽、
/// 填充/描边颜色和缩略路径数据等属性。
/// 对应 GB/T 33190-2016 第 9.1 节图 46 表 35。
#[allow(non_camel_case_types)]
#[derive(Debug, Clone)]
pub struct CT_Path {
    /// 对象 ID。
    pub id: u32,
    /// 边界框 "x y width height"（单位 mm）。
    pub boundary: String,
    /// 是否描边。
    pub stroke: bool,
    /// 是否填充。
    pub fill: bool,
    /// 填充规则。
    pub rule: FillRule,
    /// 线宽（mm）。
    pub line_width: Option<f64>,
    /// 描边颜色 RGB hex。
    pub stroke_color: Option<u32>,
    /// 填充颜色 RGB hex。
    pub fill_color: Option<u32>,
    /// 路径缩写数据。
    pub abbreviated_data: Option<AbbreviatedData>,
}

impl CT_Path {
    /// 创建新的路径对象。
    #[must_use]
    pub fn new(id: u32, boundary: impl Into<String>) -> Self {
        Self {
            id,
            boundary: boundary.into(),
            stroke: true,
            fill: false,
            rule: FillRule::NonZero,
            line_width: None,
            stroke_color: None,
            fill_color: None,
            abbreviated_data: None,
        }
    }

    /// 设置描边。
    #[must_use]
    pub fn stroke(mut self, stroke: bool) -> Self {
        self.stroke = stroke;
        self
    }

    /// 设置填充。
    #[must_use]
    pub fn fill(mut self, fill: bool) -> Self {
        self.fill = fill;
        self
    }

    /// 设置填充规则。
    #[must_use]
    pub fn rule(mut self, rule: FillRule) -> Self {
        self.rule = rule;
        self
    }

    /// 设置线宽。
    #[must_use]
    pub fn line_width(mut self, width: f64) -> Self {
        self.line_width = Some(width);
        self
    }

    /// 设置描边颜色。
    #[must_use]
    pub fn stroke_color(mut self, color: u32) -> Self {
        self.stroke_color = Some(color);
        self
    }

    /// 设置填充颜色。
    #[must_use]
    pub fn fill_color(mut self, color: u32) -> Self {
        self.fill_color = Some(color);
        self
    }

    /// 设置路径数据。
    #[must_use]
    pub fn abbreviated_data(mut self, data: AbbreviatedData) -> Self {
        self.abbreviated_data = Some(data);
        self
    }

    /// 获取路径数据字符串。
    #[must_use]
    pub fn get_abbreviated_data(&self) -> Option<String> {
        self.abbreviated_data.as_ref().map(|d| d.to_data_string())
    }

    /// 获取描边颜色。
    #[must_use]
    pub fn get_stroke_color(&self) -> Option<u32> {
        self.stroke_color
    }

    /// 获取填充颜色。
    #[must_use]
    pub fn get_fill_color(&self) -> Option<u32> {
        self.fill_color
    }

    /// 序列化为 OFD XML 字符串。
    #[must_use]
    pub fn to_xml_string(&self) -> String {
        use std::fmt::Write;
        let mut xml = format!(
            "<ofd:PathObject ID=\"{}\" Boundary=\"{}\"",
            self.id, self.boundary
        );
        if !self.stroke {
            xml.push_str(" Stroke=\"false\"");
        }
        if self.fill {
            xml.push_str(" Fill=\"true\"");
        }
        if self.rule != FillRule::NonZero {
            write!(xml, " Rule=\"{}\"", self.rule.as_str()).expect("写入内存缓冲区不会失败");
        }
        if let Some(lw) = self.line_width {
            write!(xml, " LineWidth=\"{lw}\"").expect("写入内存缓冲区不会失败");
        }
        if let Some(sc) = self.stroke_color {
            write!(xml, " StrokeColor=\"{sc}\"").expect("写入内存缓冲区不会失败");
        }
        if let Some(fc) = self.fill_color {
            write!(xml, " FillColor=\"{fc}\"").expect("写入内存缓冲区不会失败");
        }
        if let Some(ref ad) = self.abbreviated_data {
            xml.push_str(">\n  ");
            xml.push_str(&ad.to_xml_string());
            xml.push_str("\n</ofd:PathObject>\n");
        } else {
            xml.push_str(" />\n");
        }
        xml
    }
}

impl XmlElement for CT_Path {
    /// 对应 Java: CT_Path 元素名 "Path"。
    fn element_name(&self) -> &'static str {
        "Path"
    }

    fn attributes(&self) -> Vec<(String, String)> {
        let mut attrs = Vec::new();
        attrs.push(("ID".to_string(), self.id.to_string()));
        attrs.push(("Boundary".to_string(), self.boundary.clone()));
        if !self.stroke {
            attrs.push(("Stroke".to_string(), "false".to_string()));
        }
        if self.fill {
            attrs.push(("Fill".to_string(), "true".to_string()));
        }
        if self.rule != FillRule::NonZero {
            attrs.push(("Rule".to_string(), self.rule.as_str().to_string()));
        }
        if let Some(lw) = self.line_width {
            attrs.push(("LineWidth".to_string(), lw.to_string()));
        }
        if let Some(sc) = self.stroke_color {
            attrs.push(("StrokeColor".to_string(), sc.to_string()));
        }
        if let Some(fc) = self.fill_color {
            attrs.push(("FillColor".to_string(), fc.to_string()));
        }
        attrs
    }

    fn child_nodes(&self) -> Vec<XmlNode> {
        let mut children = Vec::new();
        if let Some(ref ad) = self.abbreviated_data {
            let mut node = XmlNode::element("AbbreviatedData");
            node.push_child(XmlNode::text_node(ad.to_data_string()));
            children.push(node);
        }
        children
    }

    fn from_xml(node: &XmlNode) -> Result<Self, XmlElementError> {
        let id = node
            .get_attr("ID")
            .ok_or_else(|| XmlElementError("Path 缺少 ID 属性".to_string()))?
            .parse::<u32>()
            .map_err(|e| XmlElementError(format!("解析 Path.ID 失败: {e}")))?;
        let boundary = node
            .get_attr("Boundary")
            .ok_or_else(|| XmlElementError("Path 缺少 Boundary 属性".to_string()))?
            .to_string();
        let stroke = node.get_attr("Stroke").map_or(true, |s| s != "false");
        let fill = node.get_attr("Fill").is_some_and(|s| s == "true");
        let rule = match node.get_attr("Rule") {
            Some("EvenOdd") => FillRule::EvenOdd,
            _ => FillRule::NonZero,
        };
        let line_width = node
            .get_attr("LineWidth")
            .map(|s| {
                s.parse::<f64>()
                    .map_err(|e| XmlElementError(format!("解析 Path.LineWidth 失败: {e}")))
            })
            .transpose()?;
        let stroke_color = node
            .get_attr("StrokeColor")
            .map(|s| {
                s.parse::<u32>()
                    .map_err(|e| XmlElementError(format!("解析 Path.StrokeColor 失败: {e}")))
            })
            .transpose()?;
        let fill_color = node
            .get_attr("FillColor")
            .map(|s| {
                s.parse::<u32>()
                    .map_err(|e| XmlElementError(format!("解析 Path.FillColor 失败: {e}")))
            })
            .transpose()?;
        let abbreviated_data = node.child("AbbreviatedData").map(|child| {
            let text = child.text.as_deref().unwrap_or("");
            AbbreviatedData::parse(text)
        });
        Ok(Self {
            id,
            boundary,
            stroke,
            fill,
            rule,
            line_width,
            stroke_color,
            fill_color,
            abbreviated_data,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::xml_parse::parse_xml_to_nodes;

    #[test]
    fn test_ct_path_new() {
        let p = CT_Path::new(1, "0 0 100 50");
        assert_eq!(p.id, 1);
        assert!(p.stroke);
        assert!(!p.fill);
        assert_eq!(p.rule, FillRule::NonZero);
        assert!(p.abbreviated_data.is_none());
    }

    #[test]
    fn test_ct_path_builder() {
        let p = CT_Path::new(2, "10 20 50 50")
            .stroke(false)
            .fill(true)
            .rule(FillRule::EvenOdd)
            .line_width(2.0)
            .stroke_color(0xFF_0000)
            .fill_color(0x00_FF00);
        assert!(!p.stroke);
        assert!(p.fill);
        assert_eq!(p.rule, FillRule::EvenOdd);
        assert!((p.line_width.unwrap() - 2.0).abs() < f64::EPSILON);
        assert_eq!(p.stroke_color, Some(0xFF_0000));
        assert_eq!(p.fill_color, Some(0x00_FF00));
    }

    #[test]
    fn test_ct_path_with_data() {
        let data = AbbreviatedData::new()
            .move_to(0.0, 0.0)
            .line_to(100.0, 100.0)
            .close();
        let p = CT_Path::new(3, "0 0 100 100").abbreviated_data(data);
        assert!(p.abbreviated_data.is_some());
        let data_str = p.get_abbreviated_data().unwrap();
        assert!(data_str.contains("M 0 0"));
        assert!(data_str.contains("L 100 100"));
    }

    #[test]
    fn test_fill_rule_display() {
        assert_eq!(FillRule::NonZero.to_string(), "NonZero");
        assert_eq!(FillRule::EvenOdd.to_string(), "EvenOdd");
    }

    #[test]
    fn test_ct_path_to_xml_basic() {
        let p = CT_Path::new(1, "0 0 50 50");
        let xml = p.to_xml_string();
        assert!(xml.contains("ID=\"1\""));
        assert!(xml.contains("Boundary=\"0 0 50 50\""));
        assert!(xml.contains("<ofd:PathObject"));
        assert!(xml.ends_with(" />\n"));
    }

    #[test]
    fn test_ct_path_to_xml_with_data() {
        let data = AbbreviatedData::new().move_to(0.0, 0.0).line_to(10.0, 10.0);
        let p = CT_Path::new(5, "0 0 10 10")
            .fill(true)
            .stroke(false)
            .fill_color(0x00_00FF)
            .abbreviated_data(data);
        let xml = p.to_xml_string();
        assert!(xml.contains("Fill=\"true\""));
        assert!(xml.contains("Stroke=\"false\""));
        assert!(xml.contains("FillColor=\"255\""));
        assert!(xml.contains("ofd:AbbreviatedData"));
        assert!(xml.contains("M 0 0"));
        assert!(xml.contains("L 10 10"));
    }

    #[test]
    fn test_ct_path_to_xml_evenodd() {
        let p = CT_Path::new(6, "0 0 10 10").rule(FillRule::EvenOdd);
        let xml = p.to_xml_string();
        assert!(xml.contains("Rule=\"EvenOdd\""));
    }

    #[test]
    fn test_ct_path_clone_debug() {
        let p = CT_Path::new(1, "0 0 1 1");
        let p2 = p.clone();
        assert_eq!(p2.id, 1);
        assert!(format!("{p:?}").contains("CT_Path"));
    }

    #[test]
    fn test_xml_element_name() {
        let p = CT_Path::new(1, "0 0 1 1");
        assert_eq!(p.element_name(), "Path");
    }

    #[test]
    fn test_xml_element_to_xml_contains_attrs() {
        let p = CT_Path::new(3, "10 20 50 50")
            .stroke(false)
            .fill(true)
            .rule(FillRule::EvenOdd)
            .line_width(2.5)
            .stroke_color(0xFF_0000)
            .fill_color(0x00_FF00);
        let xml = p.to_xml();
        assert!(xml.contains("ID=\"3\""));
        assert!(xml.contains("Boundary=\"10 20 50 50\""));
        assert!(xml.contains("Stroke=\"false\""));
        assert!(xml.contains("Fill=\"true\""));
        assert!(xml.contains("Rule=\"EvenOdd\""));
        assert!(xml.contains("LineWidth=\"2.5\""));
        assert!(xml.contains("StrokeColor=\"16711680\""));
        assert!(xml.contains("FillColor=\"65280\""));
    }

    #[test]
    fn test_xml_element_roundtrip_full() {
        let data = AbbreviatedData::new()
            .move_to(0.0, 0.0)
            .line_to(10.0, 10.0)
            .close();
        let p = CT_Path::new(5, "0 0 100 100")
            .stroke(false)
            .fill(true)
            .rule(FillRule::EvenOdd)
            .line_width(2.0)
            .stroke_color(0xFF_0000)
            .fill_color(0x00_FF00)
            .abbreviated_data(data);
        let xml = p.to_xml();
        let node = parse_xml_to_nodes(&xml).unwrap();
        let p2 = CT_Path::from_xml(&node).unwrap();
        assert_eq!(p.id, p2.id);
        assert_eq!(p.boundary, p2.boundary);
        assert_eq!(p.stroke, p2.stroke);
        assert_eq!(p.fill, p2.fill);
        assert_eq!(p.rule, p2.rule);
        assert_eq!(p.line_width, p2.line_width);
        assert_eq!(p.stroke_color, p2.stroke_color);
        assert_eq!(p.fill_color, p2.fill_color);
        assert_eq!(p.get_abbreviated_data(), p2.get_abbreviated_data());
    }

    #[test]
    fn test_xml_element_roundtrip_minimal() {
        let p = CT_Path::new(1, "0 0 10 10");
        let xml = p.to_xml();
        let node = parse_xml_to_nodes(&xml).unwrap();
        let p2 = CT_Path::from_xml(&node).unwrap();
        assert_eq!(p.id, p2.id);
        assert_eq!(p.boundary, p2.boundary);
        assert!(p2.stroke);
        assert!(!p2.fill);
    }
}
