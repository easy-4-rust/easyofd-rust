//! 颜色空间定义类型。
//!
//! 对应 Java: org.ofdrw.core.pageDescription.color.colorSpace.CT_ColorSpace

use crate::basic_type::{ST_ID, ST_Loc};

/// 颜色空间。
///
/// 本标准支持 GRAY、RGB、CMYK 颜色空间。
///
/// 对应 Java: org.ofdrw.core.pageDescription.color.colorSpace.CT_ColorSpace
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq)]
pub struct CT_ColorSpace {
    /// 对象 ID
    id: Option<ST_ID>,
    /// 颜色空间类型
    color_type: Option<OFDColorSpaceType>,
    /// 每通道位数
    bits_per_component: Option<u8>,
    /// 颜色配置文件路径
    profile: Option<ST_Loc>,
}

/// OFD 颜色空间类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OFDColorSpaceType {
    /// 灰度
    Gray,
    /// RGB
    Rgb,
    /// CMYK
    Cmyk,
}

impl std::fmt::Display for OFDColorSpaceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Gray => write!(f, "GRAY"),
            Self::Rgb => write!(f, "RGB"),
            Self::Cmyk => write!(f, "CMYK"),
        }
    }
}

impl CT_ColorSpace {
    /// 创建空颜色空间。
    pub fn new() -> Self {
        Self {
            id: None,
            color_type: None,
            bits_per_component: None,
            profile: None,
        }
    }

    /// 创建指定类型的颜色空间。
    pub fn with_type(color_type: OFDColorSpaceType) -> Self {
        let mut cs = Self::new();
        cs.set_type(color_type);
        cs
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

    /// 设置颜色空间类型。
    pub fn set_type(&mut self, color_type: OFDColorSpaceType) -> &mut Self {
        self.color_type = Some(color_type);
        self
    }

    /// 获取颜色空间类型。
    pub fn color_type(&self) -> Option<OFDColorSpaceType> {
        self.color_type
    }

    /// 设置每通道位数。
    pub fn set_bits_per_component(&mut self, bits: u8) -> &mut Self {
        self.bits_per_component = Some(bits);
        self
    }

    /// 获取每通道位数。
    pub fn bits_per_component(&self) -> Option<u8> {
        self.bits_per_component
    }

    /// 设置颜色配置文件路径。
    pub fn set_profile(&mut self, profile: ST_Loc) -> &mut Self {
        self.profile = Some(profile);
        self
    }

    /// 获取颜色配置文件路径。
    pub fn profile(&self) -> Option<&ST_Loc> {
        self.profile.as_ref()
    }

    /// 序列化为 OFD XML 字符串表示。
    pub fn to_xml_string(&self) -> String {
        let mut attrs = Vec::new();
        if let Some(id) = self.id {
            attrs.push(format!("ID=\"{}\"", id.to_xml_string()));
        }
        if let Some(ct) = self.color_type {
            attrs.push(format!("Type=\"{ct}\""));
        }
        if let Some(bits) = self.bits_per_component {
            attrs.push(format!("BitsPerComponent=\"{bits}\""));
        }
        if let Some(ref profile) = self.profile {
            attrs.push(format!("Profile=\"{profile}\""));
        }
        format!("<ColorSpace {} />", attrs.join(" "))
    }

    /// 从字符串解析 CT_ColorSpace。
    pub fn from_str(s: &str) -> Result<Self, String> {
        let s = s.trim();
        let color_type = match s.to_uppercase().as_str() {
            "GRAY" => OFDColorSpaceType::Gray,
            "RGB" => OFDColorSpaceType::Rgb,
            "CMYK" => OFDColorSpaceType::Cmyk,
            _ => return Err(format!("未知颜色空间类型: {s}")),
        };
        Ok(Self::with_type(color_type))
    }
}

impl Default for CT_ColorSpace {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rgb_color_space() {
        let cs = CT_ColorSpace::with_type(OFDColorSpaceType::Rgb);
        assert_eq!(cs.color_type(), Some(OFDColorSpaceType::Rgb));
    }

    #[test]
    fn test_with_id() {
        let mut cs = CT_ColorSpace::new();
        cs.set_id(ST_ID::new(1).unwrap())
            .set_type(OFDColorSpaceType::Cmyk);
        assert_eq!(cs.id(), Some(ST_ID::new(1).unwrap()));
        assert_eq!(cs.color_type(), Some(OFDColorSpaceType::Cmyk));
    }

    #[test]
    fn test_to_xml_string() {
        let cs = CT_ColorSpace::with_type(OFDColorSpaceType::Rgb);
        let xml = cs.to_xml_string();
        assert!(xml.contains("ColorSpace"));
        assert!(xml.contains("RGB"));
    }

    #[test]
    fn test_from_str() {
        let cs = CT_ColorSpace::from_str("RGB").unwrap();
        assert_eq!(cs.color_type(), Some(OFDColorSpaceType::Rgb));
    }

    #[test]
    fn test_from_str_invalid() {
        assert!(CT_ColorSpace::from_str("XYZ").is_err());
    }
}
