//! 颜色值类型。
//!
//! 对应 Java: org.ofdrw.core.pageDescription.color.color.CT_Color

use crate::basic_type::{ST_Array, ST_RefID};

/// 颜色值，支持 RGB、CMYK、灰度、命名色。
///
/// 对应 Java: org.ofdrw.core.pageDescription.color.color.CT_Color
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq)]
pub struct CT_Color {
    /// 颜色值（RGB: 3 元素, CMYK: 4 元素, 灰度: 1 元素）
    value: Option<ST_Array>,
    /// 调色板索引
    index: Option<u32>,
    /// 颜色空间引用
    color_space: Option<ST_RefID>,
    /// 透明度 (0-255)
    alpha: Option<u8>,
    /// 颜色族类型
    color: Option<ColorClusterType>,
}

/// 颜色族类型
#[derive(Debug, Clone, PartialEq)]
pub enum ColorClusterType {
    /// RGB 颜色 (r, g, b)
    Rgb(u8, u8, u8),
    /// CMYK 颜色 (c, m, y, k)
    Cmyk(u8, u8, u8, u8),
    /// 灰度颜色 (0-255)
    Gray(u8),
    /// 命名颜色
    Name(String),
}

impl CT_Color {
    /// 创建空颜色。
    pub fn new() -> Self {
        Self {
            value: None,
            index: None,
            color_space: None,
            alpha: None,
            color: None,
        }
    }

    /// 创建 RGB 颜色。
    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        let mut c = Self::new();
        c.set_color(ColorClusterType::Rgb(r, g, b));
        c
    }

    /// 创建 CMYK 颜色。
    pub fn cmyk(c_val: u8, m: u8, y: u8, k: u8) -> Self {
        let mut c = Self::new();
        c.set_color(ColorClusterType::Cmyk(c_val, m, y, k));
        c
    }

    /// 创建灰度颜色。
    pub fn gray(val: u8) -> Self {
        let mut c = Self::new();
        c.set_color(ColorClusterType::Gray(val));
        c
    }

    /// 设置颜色值。
    pub fn set_value(&mut self, value: ST_Array) -> &mut Self {
        self.value = Some(value);
        self
    }

    /// 获取颜色值。
    pub fn value(&self) -> Option<&ST_Array> {
        self.value.as_ref()
    }

    /// 设置调色板索引。
    pub fn set_index(&mut self, index: u32) -> &mut Self {
        self.index = Some(index);
        self
    }

    /// 获取调色板索引。
    pub fn index(&self) -> Option<u32> {
        self.index
    }

    /// 设置颜色空间引用。
    pub fn set_color_space(&mut self, color_space: ST_RefID) -> &mut Self {
        self.color_space = Some(color_space);
        self
    }

    /// 获取颜色空间引用。
    pub fn color_space(&self) -> Option<ST_RefID> {
        self.color_space
    }

    /// 设置透明度。
    pub fn set_alpha(&mut self, alpha: u8) -> &mut Self {
        self.alpha = Some(alpha);
        self
    }

    /// 获取透明度。
    pub fn alpha(&self) -> Option<u8> {
        self.alpha
    }

    /// 设置颜色族类型。
    pub fn set_color(&mut self, color: ColorClusterType) -> &mut Self {
        match &color {
            ColorClusterType::Rgb(r, g, b) => {
                let mut arr = ST_Array::new();
                arr.push_number(f64::from(*r));
                arr.push_number(f64::from(*g));
                arr.push_number(f64::from(*b));
                self.value = Some(arr);
            }
            ColorClusterType::Cmyk(c, m, y, k) => {
                let mut arr = ST_Array::new();
                arr.push_number(f64::from(*c));
                arr.push_number(f64::from(*m));
                arr.push_number(f64::from(*y));
                arr.push_number(f64::from(*k));
                self.value = Some(arr);
            }
            ColorClusterType::Gray(val) => {
                let mut arr = ST_Array::new();
                arr.push_number(f64::from(*val));
                self.value = Some(arr);
            }
            ColorClusterType::Name(_) => {}
        }
        self.color = Some(color);
        self
    }

    /// 获取颜色族类型。
    pub fn color(&self) -> Option<&ColorClusterType> {
        self.color.as_ref()
    }

    /// 序列化为 OFD XML 字符串表示。
    pub fn to_xml_string(&self) -> String {
        let mut attrs = Vec::new();
        if let Some(ref value) = self.value {
            attrs.push(format!("Value=\"{}\"", value.to_xml_string()));
        }
        if let Some(index) = self.index {
            attrs.push(format!("Index=\"{index}\""));
        }
        if let Some(cs) = self.color_space {
            attrs.push(format!("ColorSpace=\"{}\"", cs.to_xml_string()));
        }
        if let Some(alpha) = self.alpha {
            attrs.push(format!("Alpha=\"{alpha}\""));
        }
        format!("<Color {} />", attrs.join(" "))
    }

    /// 从字符串解析 CT_Color（简化格式：R G B 或 R G B A）。
    pub fn from_str(s: &str) -> Result<Self, String> {
        let parts: Vec<&str> = s.split_whitespace().collect();
        match parts.len() {
            1 => {
                // 灰度
                let val: u8 = parts[0]
                    .parse()
                    .map_err(|e| format!("解析灰度值失败: {e}"))?;
                Ok(Self::gray(val))
            }
            3 => {
                let r: u8 = parts[0].parse().map_err(|e| format!("解析 R 失败: {e}"))?;
                let g: u8 = parts[1].parse().map_err(|e| format!("解析 G 失败: {e}"))?;
                let b: u8 = parts[2].parse().map_err(|e| format!("解析 B 失败: {e}"))?;
                Ok(Self::rgb(r, g, b))
            }
            4 => {
                let c: u8 = parts[0].parse().map_err(|e| format!("解析 C 失败: {e}"))?;
                let m: u8 = parts[1].parse().map_err(|e| format!("解析 M 失败: {e}"))?;
                let y: u8 = parts[2].parse().map_err(|e| format!("解析 Y 失败: {e}"))?;
                let k: u8 = parts[3].parse().map_err(|e| format!("解析 K 失败: {e}"))?;
                Ok(Self::cmyk(c, m, y, k))
            }
            _ => Err(format!("CT_Color 不支持 {} 个值", parts.len())),
        }
    }
}

impl Default for CT_Color {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rgb_color() {
        let c = CT_Color::rgb(255, 128, 0);
        assert_eq!(c.color(), Some(&ColorClusterType::Rgb(255, 128, 0)));
    }

    #[test]
    fn test_cmyk_color() {
        let c = CT_Color::cmyk(100, 50, 0, 10);
        assert_eq!(c.color(), Some(&ColorClusterType::Cmyk(100, 50, 0, 10)));
    }

    #[test]
    fn test_gray_color() {
        let c = CT_Color::gray(128);
        assert_eq!(c.color(), Some(&ColorClusterType::Gray(128)));
    }

    #[test]
    fn test_alpha() {
        let mut c = CT_Color::rgb(255, 0, 0);
        c.set_alpha(128);
        assert_eq!(c.alpha(), Some(128));
    }

    #[test]
    fn test_to_xml_string() {
        let c = CT_Color::rgb(255, 128, 0);
        let xml = c.to_xml_string();
        assert!(xml.contains("Color"));
    }

    #[test]
    fn test_from_str_rgb() {
        let c = CT_Color::from_str("255 128 0").unwrap();
        assert_eq!(c.color(), Some(&ColorClusterType::Rgb(255, 128, 0)));
    }

    #[test]
    fn test_from_str_gray() {
        let c = CT_Color::from_str("128").unwrap();
        assert_eq!(c.color(), Some(&ColorClusterType::Gray(128)));
    }
}
