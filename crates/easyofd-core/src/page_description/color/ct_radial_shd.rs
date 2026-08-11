//! 径向渐变着色器类型。
//!
//! 对应 Java: org.ofdrw.core.pageDescription.color.color.CT_RadialShd

use crate::basic_type::ST_Pos;

/// 径向渐变着色器。
///
/// 对应 Java: org.ofdrw.core.pageDescription.color.color.CT_RadialShd
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq)]
pub struct CT_RadialShd {
    /// 映射类型
    map_type: Option<MapType>,
    /// 延展方式
    extend: Option<Extend>,
    /// 圆心点
    center: Option<ST_Pos>,
    /// 圆心半径
    radius: Option<f64>,
    /// 焦点
    focus: Option<ST_Pos>,
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

impl CT_RadialShd {
    /// 创建空径向渐变。
    pub fn new() -> Self {
        Self {
            map_type: None,
            extend: None,
            center: None,
            radius: None,
            focus: None,
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

    /// 设置延展方式。
    pub fn set_extend(&mut self, extend: Extend) -> &mut Self {
        self.extend = Some(extend);
        self
    }

    /// 获取延展方式。
    pub fn extend(&self) -> Option<Extend> {
        self.extend
    }

    /// 设置圆心点。
    pub fn set_center(&mut self, center: ST_Pos) -> &mut Self {
        self.center = Some(center);
        self
    }

    /// 获取圆心点。
    pub fn center(&self) -> Option<&ST_Pos> {
        self.center.as_ref()
    }

    /// 设置半径。
    pub fn set_radius(&mut self, radius: f64) -> &mut Self {
        self.radius = Some(radius);
        self
    }

    /// 获取半径。
    pub fn radius(&self) -> Option<f64> {
        self.radius
    }

    /// 设置焦点。
    pub fn set_focus(&mut self, focus: ST_Pos) -> &mut Self {
        self.focus = Some(focus);
        self
    }

    /// 获取焦点。
    pub fn focus(&self) -> Option<&ST_Pos> {
        self.focus.as_ref()
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
        if let Some(c) = &self.center {
            parts.push(format!("Center=\"{}\"", c.to_xml_string()));
        }
        if let Some(r) = self.radius {
            parts.push(format!("Radius=\"{r}\""));
        }
        if let Some(f) = &self.focus {
            parts.push(format!("Focus=\"{}\"", f.to_xml_string()));
        }
        format!("<RadialShd {} />", parts.join(" "))
    }

    /// 从字符串解析 CT_RadialShd（简化格式：cx cy radius）。
    pub fn from_str(s: &str) -> Result<Self, String> {
        let parts: Vec<&str> = s.split_whitespace().collect();
        if parts.len() < 3 {
            return Err("CT_RadialShd 需要至少 3 个值 (cx cy radius)".to_string());
        }
        let cx: f64 = parts[0].parse().map_err(|e| format!("解析 cx 失败: {e}"))?;
        let cy: f64 = parts[1].parse().map_err(|e| format!("解析 cy 失败: {e}"))?;
        let radius: f64 = parts[2]
            .parse()
            .map_err(|e| format!("解析 radius 失败: {e}"))?;
        let mut shd = Self::new();
        shd.set_center(ST_Pos::new(cx, cy));
        shd.set_radius(radius);
        Ok(shd)
    }
}

impl Default for CT_RadialShd {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_creation() {
        let mut shd = CT_RadialShd::new();
        shd.set_center(ST_Pos::new(50.0, 50.0)).set_radius(100.0);
        assert_eq!(shd.center(), Some(&ST_Pos::new(50.0, 50.0)));
        assert_eq!(shd.radius(), Some(100.0));
    }

    #[test]
    fn test_segments() {
        let mut shd = CT_RadialShd::new();
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
        let mut shd = CT_RadialShd::new();
        shd.set_center(ST_Pos::new(50.0, 50.0)).set_radius(100.0);
        let xml = shd.to_xml_string();
        assert!(xml.contains("RadialShd"));
        assert!(xml.contains("Center"));
        assert!(xml.contains("Radius"));
    }

    #[test]
    fn test_from_str() {
        let shd = CT_RadialShd::from_str("50 50 100").unwrap();
        assert_eq!(shd.center(), Some(&ST_Pos::new(50.0, 50.0)));
        assert_eq!(shd.radius(), Some(100.0));
    }

    #[test]
    fn test_from_str_invalid() {
        assert!(CT_RadialShd::from_str("50 50").is_err());
    }
}
