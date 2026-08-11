//! 轴向渐变着色器类型。
//!
//! 对应 Java: org.ofdrw.core.pageDescription.color.color.CT_AxialShd

use crate::basic_type::ST_Pos;

/// 轴向渐变着色器。
///
/// 对应 Java: org.ofdrw.core.pageDescription.color.color.CT_AxialShd
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq)]
pub struct CT_AxialShd {
    /// 映射类型
    map_type: Option<MapType>,
    /// 映射单元
    map_unit: Option<f64>,
    /// 延展方式
    extend: Option<Extend>,
    /// 起始点
    start_point: Option<ST_Pos>,
    /// 结束点
    end_point: Option<ST_Pos>,
    /// 渐变段列表
    segments: Vec<Segment>,
}

/// 映射类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MapType {
    /// 直接映射
    Direct,
    /// 扇形映射
    Fan,
}

/// 延展方式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Extend {
    /// 不延展
    None,
    /// 起始延展
    Start,
    /// 结束延展
    End,
    /// 两端延展
    Both,
}

/// 渐变段
#[derive(Debug, Clone, PartialEq)]
pub struct Segment {
    /// 起始位置 (0.0-1.0)
    pub start: f64,
    /// 结束位置 (0.0-1.0)
    pub end: f64,
    /// 起始颜色值 (RGB)
    pub start_color: [u8; 3],
    /// 结束颜色值 (RGB)
    pub end_color: [u8; 3],
}

impl CT_AxialShd {
    /// 创建空轴向渐变。
    pub fn new() -> Self {
        Self {
            map_type: None,
            map_unit: None,
            extend: None,
            start_point: None,
            end_point: None,
            segments: Vec::new(),
        }
    }

    /// 设置映射类型。
    pub fn set_map_type(&mut self, map_type: MapType) -> &mut Self {
        self.map_type = Some(map_type);
        self
    }

    /// 获取映射类型。
    pub fn map_type(&self) -> Option<MapType> {
        self.map_type
    }

    /// 设置映射单元。
    pub fn set_map_unit(&mut self, map_unit: f64) -> &mut Self {
        self.map_unit = Some(map_unit);
        self
    }

    /// 获取映射单元。
    pub fn map_unit(&self) -> Option<f64> {
        self.map_unit
    }

    /// 设置延展方式。
    pub fn set_extend(&mut self, extend: Extend) -> &mut Self {
        self.extend = Some(extend);
        self
    }

    /// 获取延展方式。
    pub fn extend(&self) -> Option<Extend> {
        self.extend
    }

    /// 设置起始点。
    pub fn set_start_point(&mut self, point: ST_Pos) -> &mut Self {
        self.start_point = Some(point);
        self
    }

    /// 获取起始点。
    pub fn start_point(&self) -> Option<&ST_Pos> {
        self.start_point.as_ref()
    }

    /// 设置结束点。
    pub fn set_end_point(&mut self, point: ST_Pos) -> &mut Self {
        self.end_point = Some(point);
        self
    }

    /// 获取结束点。
    pub fn end_point(&self) -> Option<&ST_Pos> {
        self.end_point.as_ref()
    }

    /// 添加渐变段。
    pub fn add_segment(&mut self, segment: Segment) -> &mut Self {
        self.segments.push(segment);
        self
    }

    /// 获取渐变段列表。
    pub fn segments(&self) -> &[Segment] {
        &self.segments
    }

    /// 序列化为 OFD XML 字符串表示。
    pub fn to_xml_string(&self) -> String {
        let mut parts = Vec::new();
        if let Some(sp) = &self.start_point {
            parts.push(format!("StartPoint=\"{}\"", sp.to_xml_string()));
        }
        if let Some(ep) = &self.end_point {
            parts.push(format!("EndPoint=\"{}\"", ep.to_xml_string()));
        }
        format!("<AxialShd {} />", parts.join(" "))
    }

    /// 从字符串解析 CT_AxialShd（简化格式：x1 y1 x2 y2）。
    pub fn from_str(s: &str) -> Result<Self, String> {
        let parts: Vec<&str> = s.split_whitespace().collect();
        if parts.len() < 4 {
            return Err("CT_AxialShd 需要至少 4 个值 (x1 y1 x2 y2)".to_string());
        }
        let x1: f64 = parts[0].parse().map_err(|e| format!("解析 x1 失败: {e}"))?;
        let y1: f64 = parts[1].parse().map_err(|e| format!("解析 y1 失败: {e}"))?;
        let x2: f64 = parts[2].parse().map_err(|e| format!("解析 x2 失败: {e}"))?;
        let y2: f64 = parts[3].parse().map_err(|e| format!("解析 y2 失败: {e}"))?;
        let mut shd = Self::new();
        shd.set_start_point(ST_Pos::new(x1, y1));
        shd.set_end_point(ST_Pos::new(x2, y2));
        Ok(shd)
    }
}

impl Default for CT_AxialShd {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_creation() {
        let mut shd = CT_AxialShd::new();
        shd.set_start_point(ST_Pos::new(0.0, 0.0))
            .set_end_point(ST_Pos::new(100.0, 100.0));
        assert_eq!(shd.start_point(), Some(&ST_Pos::new(0.0, 0.0)));
        assert_eq!(shd.end_point(), Some(&ST_Pos::new(100.0, 100.0)));
    }

    #[test]
    fn test_segments() {
        let mut shd = CT_AxialShd::new();
        shd.add_segment(Segment {
            start: 0.0,
            end: 1.0,
            start_color: [255, 0, 0],
            end_color: [0, 0, 255],
        });
        assert_eq!(shd.segments().len(), 1);
    }

    #[test]
    fn test_to_xml_string() {
        let mut shd = CT_AxialShd::new();
        shd.set_start_point(ST_Pos::new(0.0, 0.0))
            .set_end_point(ST_Pos::new(100.0, 100.0));
        let xml = shd.to_xml_string();
        assert!(xml.contains("AxialShd"));
        assert!(xml.contains("StartPoint"));
        assert!(xml.contains("EndPoint"));
    }

    #[test]
    fn test_from_str() {
        let shd = CT_AxialShd::from_str("0 0 100 100").unwrap();
        assert_eq!(shd.start_point(), Some(&ST_Pos::new(0.0, 0.0)));
        assert_eq!(shd.end_point(), Some(&ST_Pos::new(100.0, 100.0)));
    }

    #[test]
    fn test_from_str_invalid() {
        assert!(CT_AxialShd::from_str("0 0").is_err());
    }
}
