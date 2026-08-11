//! 底纹/图案填充类型。
//!
//! 对应 Java: org.ofdrw.core.pageDescription.color.color.CT_Pattern

use crate::basic_type::{ST_Array, ST_ID, ST_Loc};

/// 底纹/图案填充。
///
/// 对应 Java: org.ofdrw.core.pageDescription.color.color.CT_Pattern
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq)]
pub struct CT_Pattern {
    /// 对象 ID
    id: Option<ST_ID>,
    /// 图案类型
    pattern_type: Option<PatternType>,
    /// 宽度
    width: Option<f64>,
    /// 高度
    height: Option<f64>,
    /// X 偏移
    x_offset: Option<f64>,
    /// Y 偏移
    y_offset: Option<f64>,
    /// 变换矩阵
    transform: Option<ST_Array>,
    /// 资源路径
    relative: Option<ST_Loc>,
}

/// 图案类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatternType {
    /// 底纹
    Pattern,
    /// 图像
    Image,
}

impl CT_Pattern {
    /// 创建空图案填充。
    pub fn new() -> Self {
        Self {
            id: None,
            pattern_type: None,
            width: None,
            height: None,
            x_offset: None,
            y_offset: None,
            transform: None,
            relative: None,
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

    /// 设置图案类型。
    pub fn set_pattern_type(&mut self, pattern_type: PatternType) -> &mut Self {
        self.pattern_type = Some(pattern_type);
        self
    }

    /// 获取图案类型。
    pub fn pattern_type(&self) -> Option<PatternType> {
        self.pattern_type
    }

    /// 设置宽度。
    pub fn set_width(&mut self, width: f64) -> &mut Self {
        self.width = Some(width);
        self
    }

    /// 获取宽度。
    pub fn width(&self) -> Option<f64> {
        self.width
    }

    /// 设置高度。
    pub fn set_height(&mut self, height: f64) -> &mut Self {
        self.height = Some(height);
        self
    }

    /// 获取高度。
    pub fn height(&self) -> Option<f64> {
        self.height
    }

    /// 设置 X 偏移。
    pub fn set_x_offset(&mut self, x_offset: f64) -> &mut Self {
        self.x_offset = Some(x_offset);
        self
    }

    /// 获取 X 偏移。
    pub fn x_offset(&self) -> Option<f64> {
        self.x_offset
    }

    /// 设置 Y 偏移。
    pub fn set_y_offset(&mut self, y_offset: f64) -> &mut Self {
        self.y_offset = Some(y_offset);
        self
    }

    /// 获取 Y 偏移。
    pub fn y_offset(&self) -> Option<f64> {
        self.y_offset
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

    /// 设置资源路径。
    pub fn set_relative(&mut self, relative: ST_Loc) -> &mut Self {
        self.relative = Some(relative);
        self
    }

    /// 获取资源路径。
    pub fn relative(&self) -> Option<&ST_Loc> {
        self.relative.as_ref()
    }

    /// 序列化为 OFD XML 字符串表示。
    pub fn to_xml_string(&self) -> String {
        let mut attrs = Vec::new();
        if let Some(id) = self.id {
            attrs.push(format!("ID=\"{}\"", id.to_xml_string()));
        }
        if let Some(w) = self.width {
            attrs.push(format!("Width=\"{w}\""));
        }
        if let Some(h) = self.height {
            attrs.push(format!("Height=\"{h}\""));
        }
        format!("<Pattern {} />", attrs.join(" "))
    }

    /// 从字符串解析 CT_Pattern（简化格式：width height）。
    pub fn from_str(s: &str) -> Result<Self, String> {
        let parts: Vec<&str> = s.split_whitespace().collect();
        if parts.len() < 2 {
            return Err("CT_Pattern 需要至少 2 个值 (width height)".to_string());
        }
        let width: f64 = parts[0]
            .parse()
            .map_err(|e| format!("解析 width 失败: {e}"))?;
        let height: f64 = parts[1]
            .parse()
            .map_err(|e| format!("解析 height 失败: {e}"))?;
        let mut pattern = Self::new();
        pattern.set_width(width);
        pattern.set_height(height);
        Ok(pattern)
    }
}

impl Default for CT_Pattern {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_creation() {
        let mut p = CT_Pattern::new();
        p.set_width(100.0).set_height(50.0);
        assert_eq!(p.width(), Some(100.0));
        assert_eq!(p.height(), Some(50.0));
    }

    #[test]
    fn test_pattern_type() {
        let mut p = CT_Pattern::new();
        p.set_pattern_type(PatternType::Image);
        assert_eq!(p.pattern_type(), Some(PatternType::Image));
    }

    #[test]
    fn test_to_xml_string() {
        let mut p = CT_Pattern::new();
        p.set_width(100.0).set_height(50.0);
        let xml = p.to_xml_string();
        assert!(xml.contains("Pattern"));
        assert!(xml.contains("Width"));
        assert!(xml.contains("Height"));
    }

    #[test]
    fn test_from_str() {
        let p = CT_Pattern::from_str("100 50").unwrap();
        assert_eq!(p.width(), Some(100.0));
        assert_eq!(p.height(), Some(50.0));
    }

    #[test]
    fn test_from_str_invalid() {
        assert!(CT_Pattern::from_str("100").is_err());
    }
}
