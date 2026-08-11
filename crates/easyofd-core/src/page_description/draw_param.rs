//! 绘制参数类型。
//!
//! 对应 Java: org.ofdrw.core.pageDescription.drawParam.CT_DrawParam

use crate::basic_type::{ST_Array, ST_ID};
use crate::page_description::color::CT_Color;
use crate::xml_element::{XmlElement, XmlElementError, XmlNode};

/// 绘制参数。
///
/// 对应 Java: org.ofdrw.core.pageDescription.drawParam.CT_DrawParam
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq)]
pub struct CT_DrawParam {
    /// 对象 ID
    id: Option<ST_ID>,
    /// 线宽
    line_width: Option<f64>,
    /// 线端帽
    line_cap: Option<LineCap>,
    /// 线连接样式
    line_join: Option<LineJoin>,
    /// 虚线偏移
    dash_offset: Option<f64>,
    /// 虚线模式
    dash_pattern: Option<ST_Array>,
    /// 斜接限制
    miter_limit: Option<f64>,
    /// 填充颜色
    fill_color: Option<CT_Color>,
    /// 描边颜色
    stroke_color: Option<CT_Color>,
    /// 变换矩阵
    transform: Option<ST_Array>,
}

/// 线端帽样式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineCap {
    /// 平头
    Butt,
    /// 圆头
    Round,
    /// 方头
    Square,
}

/// 线连接样式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineJoin {
    /// 尖角
    Miter,
    /// 圆角
    Round,
    /// 平角
    Bevel,
}

impl CT_DrawParam {
    /// 创建空绘制参数。
    pub fn new() -> Self {
        Self {
            id: None,
            line_width: None,
            line_cap: None,
            line_join: None,
            dash_offset: None,
            dash_pattern: None,
            miter_limit: None,
            fill_color: None,
            stroke_color: None,
            transform: None,
        }
    }

    /// 设置对象 ID。
    pub fn set_id(&mut self, id: ST_ID) -> &mut Self {
        self.id = Some(id);
        self
    }

    /// 获取对象 ID。
    pub fn id(&self) -> Option<ST_ID> {
        self.id
    }

    /// 设置线宽。
    pub fn set_line_width(&mut self, width: f64) -> &mut Self {
        self.line_width = Some(width);
        self
    }

    /// 获取线宽。
    pub fn line_width(&self) -> Option<f64> {
        self.line_width
    }

    /// 设置线端帽样式。
    pub fn set_line_cap(&mut self, line_cap: LineCap) -> &mut Self {
        self.line_cap = Some(line_cap);
        self
    }

    /// 获取线端帽样式。
    pub fn line_cap(&self) -> Option<LineCap> {
        self.line_cap
    }

    /// 设置线连接样式。
    pub fn set_line_join(&mut self, line_join: LineJoin) -> &mut Self {
        self.line_join = Some(line_join);
        self
    }

    /// 获取线连接样式。
    pub fn line_join(&self) -> Option<LineJoin> {
        self.line_join
    }

    /// 设置虚线偏移。
    pub fn set_dash_offset(&mut self, offset: f64) -> &mut Self {
        self.dash_offset = Some(offset);
        self
    }

    /// 获取虚线偏移。
    pub fn dash_offset(&self) -> Option<f64> {
        self.dash_offset
    }

    /// 设置虚线模式。
    pub fn set_dash_pattern(&mut self, pattern: ST_Array) -> &mut Self {
        self.dash_pattern = Some(pattern);
        self
    }

    /// 获取虚线模式。
    pub fn dash_pattern(&self) -> Option<&ST_Array> {
        self.dash_pattern.as_ref()
    }

    /// 设置斜接限制。
    pub fn set_miter_limit(&mut self, limit: f64) -> &mut Self {
        self.miter_limit = Some(limit);
        self
    }

    /// 获取斜接限制。
    pub fn miter_limit(&self) -> Option<f64> {
        self.miter_limit
    }

    /// 设置填充颜色。
    pub fn set_fill_color(&mut self, color: CT_Color) -> &mut Self {
        self.fill_color = Some(color);
        self
    }

    /// 获取填充颜色。
    pub fn fill_color(&self) -> Option<&CT_Color> {
        self.fill_color.as_ref()
    }

    /// 设置描边颜色。
    pub fn set_stroke_color(&mut self, color: CT_Color) -> &mut Self {
        self.stroke_color = Some(color);
        self
    }

    /// 获取描边颜色。
    pub fn stroke_color(&self) -> Option<&CT_Color> {
        self.stroke_color.as_ref()
    }

    /// 设置变换矩阵。
    pub fn set_transform(&mut self, transform: ST_Array) -> &mut Self {
        self.transform = Some(transform);
        self
    }

    /// 获取变换矩阵。
    pub fn transform(&self) -> Option<&ST_Array> {
        self.transform.as_ref()
    }

    /// 序列化为 OFD XML 字符串表示。
    pub fn to_xml_string(&self) -> String {
        let mut attrs = Vec::new();
        if let Some(id) = self.id {
            attrs.push(format!("ID=\"{}\"", id.to_xml_string()));
        }
        if let Some(lw) = self.line_width {
            attrs.push(format!("LineWidth=\"{lw}\""));
        }
        if let Some(ml) = self.miter_limit {
            attrs.push(format!("MiterLimit=\"{ml}\""));
        }
        format!("<DrawParam {} />", attrs.join(" "))
    }

    /// 从字符串解析 CT_DrawParam（简化格式：line_width）。
    pub fn from_str(s: &str) -> Result<Self, String> {
        let s = s.trim();
        if s.is_empty() {
            return Err("CT_DrawParam 不能为空".to_string());
        }
        let width: f64 = s
            .parse()
            .map_err(|e| format!("解析 line_width 失败: {e}"))?;
        let mut dp = Self::new();
        dp.set_line_width(width);
        Ok(dp)
    }
}

impl Default for CT_DrawParam {
    fn default() -> Self {
        Self::new()
    }
}

impl XmlElement for CT_DrawParam {
    /// 对应 Java: CT_DrawParam 元素名 "DrawParam"。
    fn element_name(&self) -> &'static str {
        "DrawParam"
    }

    fn attributes(&self) -> Vec<(String, String)> {
        let mut attrs = Vec::new();
        if let Some(id) = self.id {
            attrs.push(("ID".to_string(), id.to_xml_string()));
        }
        if let Some(lw) = self.line_width {
            attrs.push(("LineWidth".to_string(), lw.to_string()));
        }
        if let Some(lc) = self.line_cap {
            attrs.push((
                "LineCap".to_string(),
                match lc {
                    LineCap::Butt => "Butt",
                    LineCap::Round => "Round",
                    LineCap::Square => "Square",
                }
                .to_string(),
            ));
        }
        if let Some(lj) = self.line_join {
            attrs.push((
                "LineJoin".to_string(),
                match lj {
                    LineJoin::Miter => "Miter",
                    LineJoin::Round => "Round",
                    LineJoin::Bevel => "Bevel",
                }
                .to_string(),
            ));
        }
        if let Some(doff) = self.dash_offset {
            attrs.push(("DashOffset".to_string(), doff.to_string()));
        }
        if let Some(ref dp) = self.dash_pattern {
            attrs.push(("DashPattern".to_string(), dp.to_xml_string()));
        }
        if let Some(ml) = self.miter_limit {
            attrs.push(("MiterLimit".to_string(), ml.to_string()));
        }
        if let Some(ref tf) = self.transform {
            attrs.push(("Transform".to_string(), tf.to_xml_string()));
        }
        attrs
    }

    fn child_nodes(&self) -> Vec<XmlNode> {
        let mut children = Vec::new();
        if let Some(ref fc) = self.fill_color {
            let mut node = XmlNode::element("FillColor");
            for (k, v) in fc.attributes() {
                node.attrs.push((k, v));
            }
            children.push(node);
        }
        if let Some(ref sc) = self.stroke_color {
            let mut node = XmlNode::element("StrokeColor");
            for (k, v) in sc.attributes() {
                node.attrs.push((k, v));
            }
            children.push(node);
        }
        children
    }

    fn from_xml(node: &XmlNode) -> Result<Self, XmlElementError> {
        let id = node
            .get_attr("ID")
            .map(|s| {
                ST_ID::from_str(s)
                    .map_err(|e| XmlElementError(format!("解析 DrawParam.ID 失败: {e}")))
            })
            .transpose()?;
        let line_width = node
            .get_attr("LineWidth")
            .map(|s| {
                s.parse::<f64>()
                    .map_err(|e| XmlElementError(format!("解析 DrawParam.LineWidth 失败: {e}")))
            })
            .transpose()?;
        let line_cap = node
            .get_attr("LineCap")
            .map(|s| match s {
                "Butt" => Ok(LineCap::Butt),
                "Round" => Ok(LineCap::Round),
                "Square" => Ok(LineCap::Square),
                other => Err(XmlElementError(format!("未知 LineCap 值: {other}"))),
            })
            .transpose()?;
        let line_join = node
            .get_attr("LineJoin")
            .map(|s| match s {
                "Miter" => Ok(LineJoin::Miter),
                "Round" => Ok(LineJoin::Round),
                "Bevel" => Ok(LineJoin::Bevel),
                other => Err(XmlElementError(format!("未知 LineJoin 值: {other}"))),
            })
            .transpose()?;
        let dash_offset = node
            .get_attr("DashOffset")
            .map(|s| {
                s.parse::<f64>()
                    .map_err(|e| XmlElementError(format!("解析 DrawParam.DashOffset 失败: {e}")))
            })
            .transpose()?;
        let dash_pattern = node
            .get_attr("DashPattern")
            .map(|s| {
                ST_Array::from_str(s)
                    .map_err(|e| XmlElementError(format!("解析 DrawParam.DashPattern 失败: {e}")))
            })
            .transpose()?;
        let miter_limit = node
            .get_attr("MiterLimit")
            .map(|s| {
                s.parse::<f64>()
                    .map_err(|e| XmlElementError(format!("解析 DrawParam.MiterLimit 失败: {e}")))
            })
            .transpose()?;
        let transform = node
            .get_attr("Transform")
            .map(|s| {
                ST_Array::from_str(s)
                    .map_err(|e| XmlElementError(format!("解析 DrawParam.Transform 失败: {e}")))
            })
            .transpose()?;
        let fill_color = node
            .child("FillColor")
            .map(CT_Color::from_xml)
            .transpose()?;
        let stroke_color = node
            .child("StrokeColor")
            .map(CT_Color::from_xml)
            .transpose()?;
        Ok(Self {
            id,
            line_width,
            line_cap,
            line_join,
            dash_offset,
            dash_pattern,
            miter_limit,
            fill_color,
            stroke_color,
            transform,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::xml_parse::parse_xml_to_nodes;

    #[test]
    fn test_basic_creation() {
        let mut dp = CT_DrawParam::new();
        dp.set_line_width(2.0);
        assert_eq!(dp.line_width(), Some(2.0));
    }

    #[test]
    fn test_line_cap_and_join() {
        let mut dp = CT_DrawParam::new();
        dp.set_line_cap(LineCap::Round)
            .set_line_join(LineJoin::Bevel);
        assert_eq!(dp.line_cap(), Some(LineCap::Round));
        assert_eq!(dp.line_join(), Some(LineJoin::Bevel));
    }

    #[test]
    fn test_fill_and_stroke_color() {
        let mut dp = CT_DrawParam::new();
        dp.set_fill_color(CT_Color::rgb(255, 0, 0))
            .set_stroke_color(CT_Color::rgb(0, 0, 0));
        assert!(dp.fill_color().is_some());
        assert!(dp.stroke_color().is_some());
    }

    #[test]
    fn test_to_xml_string() {
        let mut dp = CT_DrawParam::new();
        dp.set_line_width(2.0);
        let xml = dp.to_xml_string();
        assert!(xml.contains("DrawParam"));
        assert!(xml.contains("LineWidth"));
    }

    #[test]
    fn test_from_str() {
        let dp = CT_DrawParam::from_str("2.0").unwrap();
        assert_eq!(dp.line_width(), Some(2.0));
    }

    #[test]
    fn test_from_str_empty() {
        assert!(CT_DrawParam::from_str("").is_err());
    }

    #[test]
    fn test_xml_element_name() {
        let dp = CT_DrawParam::new();
        assert_eq!(dp.element_name(), "DrawParam");
    }

    #[test]
    fn test_xml_element_to_xml_attrs() {
        let mut dp = CT_DrawParam::new();
        dp.set_id(ST_ID::new(10).unwrap());
        dp.set_line_width(2.5);
        dp.set_line_cap(LineCap::Round);
        dp.set_line_join(LineJoin::Bevel);
        dp.set_miter_limit(4.0);
        let xml = dp.to_xml();
        assert!(xml.contains("ID=\"10\""));
        assert!(xml.contains("LineWidth=\"2.5\""));
        assert!(xml.contains("LineCap=\"Round\""));
        assert!(xml.contains("LineJoin=\"Bevel\""));
        assert!(xml.contains("MiterLimit=\"4\""));
    }

    #[test]
    fn test_xml_element_roundtrip_basic() {
        let mut dp = CT_DrawParam::new();
        dp.set_id(ST_ID::new(5).unwrap());
        dp.set_line_width(3.0);
        dp.set_line_cap(LineCap::Square);
        dp.set_line_join(LineJoin::Miter);
        dp.set_miter_limit(10.0);
        let xml = dp.to_xml();
        let node = parse_xml_to_nodes(&xml).unwrap();
        let dp2 = CT_DrawParam::from_xml(&node).unwrap();
        assert_eq!(dp.id(), dp2.id());
        assert_eq!(dp.line_width(), dp2.line_width());
        assert_eq!(dp.line_cap(), dp2.line_cap());
        assert_eq!(dp.line_join(), dp2.line_join());
        assert_eq!(dp.miter_limit(), dp2.miter_limit());
    }

    #[test]
    fn test_xml_element_roundtrip_with_colors() {
        let mut dp = CT_DrawParam::new();
        dp.set_line_width(1.5);
        dp.set_fill_color(CT_Color::rgb(255, 0, 0));
        dp.set_stroke_color(CT_Color::rgb(0, 0, 255));
        let xml = dp.to_xml();
        assert!(xml.contains("FillColor"));
        assert!(xml.contains("StrokeColor"));
        let node = parse_xml_to_nodes(&xml).unwrap();
        let dp2 = CT_DrawParam::from_xml(&node).unwrap();
        assert_eq!(dp.line_width(), dp2.line_width());
        assert!(dp2.fill_color().is_some());
        assert!(dp2.stroke_color().is_some());
    }

    #[test]
    fn test_xml_element_roundtrip_empty() {
        let dp = CT_DrawParam::new();
        let xml = dp.to_xml();
        assert_eq!(xml, "<DrawParam/>");
        let node = parse_xml_to_nodes(&xml).unwrap();
        let dp2 = CT_DrawParam::from_xml(&node).unwrap();
        assert_eq!(dp.line_width(), dp2.line_width());
    }
}
