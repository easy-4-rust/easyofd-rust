//! 字形变换类型。
//!
//! 对应 Java: org.ofdrw.core.pageDescription.CT_CGTransform

use crate::basic_type::ST_Array;

/// 字形变换。
///
/// 对应 Java: org.ofdrw.core.pageDescription.CT_CGTransform
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq)]
pub struct CT_CGTransform {
    /// 变换后字符索引集合
    code_position: Option<u32>,
    /// 变换前字符索引集合
    glyph_count: Option<u32>,
    /// 变换后字符索引集合
    glyph_position: Option<u32>,
    /// 变换矩阵
    transform: Option<ST_Array>,
}

impl CT_CGTransform {
    /// 创建空字形变换。
    pub fn new() -> Self {
        Self {
            code_position: None,
            glyph_count: None,
            glyph_position: None,
            transform: None,
        }
    }

    /// 设置变换前字符索引。
    pub fn set_code_position(&mut self, pos: u32) -> &mut Self {
        self.code_position = Some(pos);
        self
    }

    /// 获取变换前字符索引。
    pub fn code_position(&self) -> Option<u32> {
        self.code_position
    }

    /// 设置字形数量。
    pub fn set_glyph_count(&mut self, count: u32) -> &mut Self {
        self.glyph_count = Some(count);
        self
    }

    /// 获取字形数量。
    pub fn glyph_count(&self) -> Option<u32> {
        self.glyph_count
    }

    /// 设置变换后字符索引。
    pub fn set_glyph_position(&mut self, pos: u32) -> &mut Self {
        self.glyph_position = Some(pos);
        self
    }

    /// 获取变换后字符索引。
    pub fn glyph_position(&self) -> Option<u32> {
        self.glyph_position
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
        if let Some(cp) = self.code_position {
            attrs.push(format!("CodePosition=\"{cp}\""));
        }
        if let Some(gc) = self.glyph_count {
            attrs.push(format!("GlyphCount=\"{gc}\""));
        }
        if let Some(gp) = self.glyph_position {
            attrs.push(format!("GlyphPosition=\"{gp}\""));
        }
        format!("<CGTransform {} />", attrs.join(" "))
    }

    /// 从字符串解析 CT_CGTransform（简化格式：code_position glyph_count glyph_position）。
    pub fn from_str(s: &str) -> Result<Self, String> {
        let parts: Vec<&str> = s.split_whitespace().collect();
        if parts.len() < 3 {
            return Err("CT_CGTransform 需要至少 3 个值".to_string());
        }
        let code_position: u32 = parts[0]
            .parse()
            .map_err(|e| format!("解析 code_position 失败: {e}"))?;
        let glyph_count: u32 = parts[1]
            .parse()
            .map_err(|e| format!("解析 glyph_count 失败: {e}"))?;
        let glyph_position: u32 = parts[2]
            .parse()
            .map_err(|e| format!("解析 glyph_position 失败: {e}"))?;
        let mut t = Self::new();
        t.set_code_position(code_position)
            .set_glyph_count(glyph_count)
            .set_glyph_position(glyph_position);
        Ok(t)
    }
}

impl Default for CT_CGTransform {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_creation() {
        let mut t = CT_CGTransform::new();
        t.set_code_position(0)
            .set_glyph_count(3)
            .set_glyph_position(0);
        assert_eq!(t.code_position(), Some(0));
        assert_eq!(t.glyph_count(), Some(3));
        assert_eq!(t.glyph_position(), Some(0));
    }

    #[test]
    fn test_with_transform() {
        let mut t = CT_CGTransform::new();
        let transform = ST_Array::transform(1.0, 0.0, 0.0, 1.0, 5.0, 0.0);
        t.set_transform(transform);
        assert!(t.transform().is_some());
    }

    #[test]
    fn test_to_xml_string() {
        let mut t = CT_CGTransform::new();
        t.set_code_position(0).set_glyph_count(3);
        let xml = t.to_xml_string();
        assert!(xml.contains("CGTransform"));
        assert!(xml.contains("CodePosition"));
        assert!(xml.contains("GlyphCount"));
    }

    #[test]
    fn test_from_str() {
        let t = CT_CGTransform::from_str("0 3 0").unwrap();
        assert_eq!(t.code_position(), Some(0));
        assert_eq!(t.glyph_count(), Some(3));
        assert_eq!(t.glyph_position(), Some(0));
    }

    #[test]
    fn test_from_str_invalid() {
        assert!(CT_CGTransform::from_str("0 3").is_err());
    }
}
