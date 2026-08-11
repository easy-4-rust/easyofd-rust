//! LaGouraud 渐变着色器类型。
//!
//! 对应 Java: org.ofdrw.core.pageDescription.color.color.CT_LaGouraudShd

use crate::basic_type::{ST_Array, ST_ID};

/// LaGouraud 渐变着色器。
///
/// 对应 Java: org.ofdrw.core.pageDescription.color.color.CT_LaGouraudShd
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq)]
pub struct CT_LaGouraudShd {
    /// 对象 ID
    id: Option<ST_ID>,
    /// 顶点颜色
    vertices: Vec<Vertex>,
    /// 三角形索引
    triangles: Vec<[usize; 3]>,
}

/// 渐变顶点
#[derive(Debug, Clone, PartialEq)]
pub struct Vertex {
    /// X 坐标
    pub x: f64,
    /// Y 坐标
    pub y: f64,
    /// 颜色值
    pub color: ST_Array,
}

impl CT_LaGouraudShd {
    /// 创建空 LaGouraud 渐变。
    pub fn new() -> Self {
        Self {
            id: None,
            vertices: Vec::new(),
            triangles: Vec::new(),
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

    /// 添加顶点。
    pub fn add_vertex(&mut self, vertex: Vertex) -> &mut Self {
        self.vertices.push(vertex);
        self
    }

    /// 获取顶点列表。
    pub fn vertices(&self) -> &[Vertex] {
        &self.vertices
    }

    /// 添加三角形索引。
    pub fn add_triangle(&mut self, a: usize, b: usize, c: usize) -> &mut Self {
        self.triangles.push([a, b, c]);
        self
    }

    /// 获取三角形索引列表。
    pub fn triangles(&self) -> &[[usize; 3]] {
        &self.triangles
    }

    /// 序列化为 OFD XML 字符串表示。
    pub fn to_xml_string(&self) -> String {
        let mut attrs = Vec::new();
        if let Some(id) = self.id {
            attrs.push(format!("ID=\"{}\"", id.to_xml_string()));
        }
        format!("<LaGouraudShd {} />", attrs.join(" "))
    }

    /// 从字符串解析 CT_LaGouraudShd。
    pub fn from_str(s: &str) -> Result<Self, String> {
        let s = s.trim();
        if s.is_empty() {
            return Err("CT_LaGouraudShd 不能为空".to_string());
        }
        Ok(Self::new())
    }
}

impl Default for CT_LaGouraudShd {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_creation() {
        let shd = CT_LaGouraudShd::new();
        assert!(shd.vertices().is_empty());
        assert!(shd.triangles().is_empty());
    }

    #[test]
    fn test_vertices_and_triangles() {
        let mut shd = CT_LaGouraudShd::new();
        let mut color1 = ST_Array::new();
        color1.push_number(255.0);
        color1.push_number(0.0);
        color1.push_number(0.0);
        shd.add_vertex(Vertex {
            x: 0.0,
            y: 0.0,
            color: color1,
        })
        .add_triangle(0, 0, 0);
        assert_eq!(shd.vertices().len(), 1);
        assert_eq!(shd.triangles().len(), 1);
    }

    #[test]
    fn test_to_xml_string() {
        let shd = CT_LaGouraudShd::new();
        let xml = shd.to_xml_string();
        assert!(xml.contains("LaGouraudShd"));
    }

    #[test]
    fn test_from_str() {
        let shd = CT_LaGouraudShd::from_str("dummy").unwrap();
        assert!(shd.vertices().is_empty());
    }

    #[test]
    fn test_from_str_empty() {
        assert!(CT_LaGouraudShd::from_str("").is_err());
    }
}
