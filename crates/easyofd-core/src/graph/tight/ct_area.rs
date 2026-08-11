//! 紧凑图形区域。
//!
//! 对应 Java: org.ofdrw.core.graph.tight.CT_Area

/// 紧凑图形区域，定义路径对象的绘制区域。
///
/// 对应 Java: org.ofdrw.core.graph.tight.CT_Area
///
/// 用于指定路径对象的有效绘制区域，通常以边界框形式表示。
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq)]
pub struct CT_Area {
    /// 边界框 "x y width height"（单位 mm）。
    pub boundary: String,
}

impl CT_Area {
    /// 创建新的区域。
    #[must_use]
    pub fn new(boundary: impl Into<String>) -> Self {
        Self {
            boundary: boundary.into(),
        }
    }

    /// 获取边界框字符串。
    #[must_use]
    pub fn boundary(&self) -> &str {
        &self.boundary
    }

    /// 序列化为 OFD XML 字符串。
    #[must_use]
    pub fn to_xml_string(&self) -> String {
        format!("<ofd:Area Boundary=\"{}\" />", self.boundary)
    }
}

impl std::fmt::Display for CT_Area {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.boundary)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ct_area_new() {
        let area = CT_Area::new("0 0 100 50");
        assert_eq!(area.boundary(), "0 0 100 50");
    }

    #[test]
    fn test_ct_area_display() {
        let area = CT_Area::new("10 20 30 40");
        assert_eq!(area.to_string(), "10 20 30 40");
    }

    #[test]
    fn test_ct_area_to_xml() {
        let area = CT_Area::new("0 0 210 297");
        let xml = area.to_xml_string();
        assert!(xml.contains("Boundary=\"0 0 210 297\""));
        assert!(xml.contains("<ofd:Area"));
    }

    #[test]
    fn test_ct_area_clone_debug() {
        let area = CT_Area::new("0 0 100 100");
        let area2 = area.clone();
        assert_eq!(area2.boundary(), "0 0 100 100");
        assert!(format!("{area:?}").contains("CT_Area"));
    }
}
