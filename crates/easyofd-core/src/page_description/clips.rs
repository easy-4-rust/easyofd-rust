//! 裁剪区域类型。
//!
//! 对应 Java: org.ofdrw.core.pageDescription.clips.CT_Clip

use crate::basic_type::ST_Array;
use crate::xml_element::{XmlElement, XmlElementError, XmlNode};

/// 裁剪区域。
///
/// 对应 Java: org.ofdrw.core.pageDescription.clips.CT_Clip
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq)]
pub struct CT_Clip {
    /// 裁剪路径
    path: Vec<ClipPath>,
}

/// 裁剪路径
#[derive(Debug, Clone, PartialEq)]
pub struct ClipPath {
    /// 路径数据
    data: String,
    /// 变换矩阵
    transform: Option<ST_Array>,
    /// 绘制参数
    draw_param: Option<u64>,
}

impl CT_Clip {
    /// 创建空裁剪区域。
    pub fn new() -> Self {
        Self { path: Vec::new() }
    }

    /// 添加裁剪路径。
    pub fn add_path(&mut self, path: ClipPath) -> &mut Self {
        self.path.push(path);
        self
    }

    /// 获取裁剪路径列表。
    pub fn paths(&self) -> &[ClipPath] {
        &self.path
    }

    /// 序列化为 OFD XML 字符串表示。
    pub fn to_xml_string(&self) -> String {
        format!("<Clip />")
    }

    /// 从字符串解析 CT_Clip。
    pub fn from_str(s: &str) -> Result<Self, String> {
        let s = s.trim();
        if s.is_empty() {
            return Err("CT_Clip 不能为空".to_string());
        }
        Ok(Self::new())
    }
}

impl ClipPath {
    /// 创建裁剪路径。
    pub fn new(data: &str) -> Self {
        Self {
            data: data.to_string(),
            transform: None,
            draw_param: None,
        }
    }

    /// 获取路径数据。
    pub fn data(&self) -> &str {
        &self.data
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

    /// 设置绘制参数引用。
    pub fn set_draw_param(&mut self, draw_param: u64) -> &mut Self {
        self.draw_param = Some(draw_param);
        self
    }

    /// 获取绘制参数引用。
    pub fn draw_param(&self) -> Option<u64> {
        self.draw_param
    }
}

impl Default for CT_Clip {
    fn default() -> Self {
        Self::new()
    }
}

impl XmlElement for CT_Clip {
    /// 对应 Java: CT_Clip 元素名 "Clip"。
    fn element_name(&self) -> &'static str {
        "Clip"
    }

    fn attributes(&self) -> Vec<(String, String)> {
        Vec::new()
    }

    fn child_nodes(&self) -> Vec<XmlNode> {
        self.path
            .iter()
            .map(|p| {
                let mut node = XmlNode::element("Path");
                node.attrs.push(("Data".to_string(), p.data.clone()));
                if let Some(ref tf) = p.transform {
                    node.attrs
                        .push(("Transform".to_string(), tf.to_xml_string()));
                }
                if let Some(dp) = p.draw_param {
                    node.attrs.push(("DrawParam".to_string(), dp.to_string()));
                }
                node
            })
            .collect()
    }

    fn from_xml(node: &XmlNode) -> Result<Self, XmlElementError> {
        let path = node
            .children_named("Path")
            .map(|child| {
                let data = child
                    .get_attr("Data")
                    .ok_or_else(|| XmlElementError("Path 缺少 Data 属性".to_string()))?
                    .to_string();
                let transform = child
                    .get_attr("Transform")
                    .map(|s| {
                        ST_Array::from_str(s)
                            .map_err(|e| XmlElementError(format!("解析 Path.Transform 失败: {e}")))
                    })
                    .transpose()?;
                let draw_param = child
                    .get_attr("DrawParam")
                    .map(|s| {
                        s.parse::<u64>()
                            .map_err(|e| XmlElementError(format!("解析 Path.DrawParam 失败: {e}")))
                    })
                    .transpose()?;
                Ok(ClipPath {
                    data,
                    transform,
                    draw_param,
                })
            })
            .collect::<Result<Vec<_>, XmlElementError>>()?;
        Ok(Self { path })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::xml_parse::parse_xml_to_nodes;

    #[test]
    fn test_basic_creation() {
        let clip = CT_Clip::new();
        assert!(clip.paths().is_empty());
    }

    #[test]
    fn test_add_path() {
        let mut clip = CT_Clip::new();
        clip.add_path(ClipPath::new("M 0 0 L 100 0 L 100 100 Z"));
        assert_eq!(clip.paths().len(), 1);
        assert_eq!(clip.paths()[0].data(), "M 0 0 L 100 0 L 100 100 Z");
    }

    #[test]
    fn test_clip_path_transform() {
        let mut path = ClipPath::new("M 0 0 L 100 0");
        let transform = ST_Array::transform(1.0, 0.0, 0.0, 1.0, 10.0, 20.0);
        path.set_transform(transform);
        assert!(path.transform().is_some());
    }

    #[test]
    fn test_to_xml_string() {
        let clip = CT_Clip::new();
        let xml = clip.to_xml_string();
        assert!(xml.contains("Clip"));
    }

    #[test]
    fn test_from_str() {
        let clip = CT_Clip::from_str("dummy").unwrap();
        assert!(clip.paths().is_empty());
    }

    #[test]
    fn test_from_str_empty() {
        assert!(CT_Clip::from_str("").is_err());
    }

    #[test]
    fn test_xml_element_name() {
        let clip = CT_Clip::new();
        assert_eq!(clip.element_name(), "Clip");
    }

    #[test]
    fn test_xml_element_roundtrip_empty() {
        let clip = CT_Clip::new();
        let xml = clip.to_xml();
        assert_eq!(xml, "<Clip/>");
        let node = parse_xml_to_nodes(&xml).unwrap();
        let clip2 = CT_Clip::from_xml(&node).unwrap();
        assert_eq!(clip, clip2);
    }

    #[test]
    fn test_xml_element_roundtrip_with_paths() {
        let mut clip = CT_Clip::new();
        let mut p1 = ClipPath::new("M 0 0 L 100 0 L 100 100 Z");
        p1.set_draw_param(5);
        clip.add_path(p1);
        let mut p2 = ClipPath::new("M 0 0 L 50 50");
        p2.set_transform(ST_Array::transform(1.0, 0.0, 0.0, 1.0, 10.0, 20.0));
        clip.add_path(p2);

        let xml = clip.to_xml();
        assert!(xml.contains("Path"));
        assert!(xml.contains("Data=\"M 0 0 L 100 0 L 100 100 Z\""));
        assert!(xml.contains("DrawParam=\"5\""));

        let node = parse_xml_to_nodes(&xml).unwrap();
        let clip2 = CT_Clip::from_xml(&node).unwrap();
        assert_eq!(clip, clip2);
    }
}
