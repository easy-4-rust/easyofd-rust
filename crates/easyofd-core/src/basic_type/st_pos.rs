//! 二维坐标点类型。
//!
//! 对应 Java: org.ofdrw.core.basicType.ST_Pos

/// 点坐标，以空格分割，前者为 x 值，后者为 y 值，可以是整数或浮点数。
///
/// 示例：`0 0`
///
/// 对应 Java: org.ofdrw.core.basicType.ST_Pos
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq)]
pub struct ST_Pos {
    /// X 坐标
    pub x: f64,
    /// Y 坐标
    pub y: f64,
}

impl ST_Pos {
    /// 创建新的坐标点。
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// 序列化为 OFD XML 字符串表示。
    pub fn to_xml_string(&self) -> String {
        format!("{} {}", self.x, self.y)
    }

    /// 从字符串解析 ST_Pos。
    pub fn from_str(s: &str) -> Result<Self, String> {
        let parts: Vec<&str> = s.split_whitespace().collect();
        if parts.len() != 2 {
            return Err(format!("ST_Pos 需要 2 个值，得到 {} 个", parts.len()));
        }
        let x: f64 = parts[0].parse().map_err(|e| format!("解析 x 失败: {e}"))?;
        let y: f64 = parts[1].parse().map_err(|e| format!("解析 y 失败: {e}"))?;
        Ok(Self { x, y })
    }
}

impl crate::xml_element::XmlElement for ST_Pos {
    /// 对应 Java: ST_Pos 元素名 "ST_Pos"。
    fn element_name(&self) -> &'static str {
        "ST_Pos"
    }

    fn attributes(&self) -> Vec<(String, String)> {
        Vec::new()
    }

    /// 覆写 write_xml 以处理计算得到的文本内容。
    fn write_xml(&self, out: &mut String) {
        out.push_str("<ST_Pos>");
        out.push_str(&crate::xml_element::xml_escape(&self.to_xml_string()));
        out.push_str("</ST_Pos>");
    }

    fn from_xml(
        node: &crate::xml_element::XmlNode,
    ) -> Result<Self, crate::xml_element::XmlElementError> {
        let text = node.text.as_deref().ok_or_else(|| {
            crate::xml_element::XmlElementError("ST_Pos 缺少文本内容".to_string())
        })?;
        Self::from_str(text)
            .map_err(|e| crate::xml_element::XmlElementError(format!("解析 ST_Pos 失败: {e}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_xml_string() {
        let p = ST_Pos::new(10.0, 20.0);
        assert_eq!(p.to_xml_string(), "10 20");
    }

    #[test]
    fn test_from_str() {
        let p = ST_Pos::from_str("10 20").unwrap();
        assert_eq!(p.x, 10.0);
        assert_eq!(p.y, 20.0);
    }

    #[test]
    fn test_from_str_floats() {
        let p = ST_Pos::from_str("1.5 2.5").unwrap();
        assert_eq!(p.x, 1.5);
        assert_eq!(p.y, 2.5);
    }

    #[test]
    fn test_from_str_invalid_count() {
        assert!(ST_Pos::from_str("1 2 3").is_err());
    }

    #[test]
    fn test_roundtrip() {
        let p = ST_Pos::new(100.5, 200.5);
        let s = p.to_xml_string();
        let p2 = ST_Pos::from_str(&s).unwrap();
        assert_eq!(p, p2);
    }

    #[test]
    fn test_xml_element_roundtrip() {
        use crate::xml_element::XmlElement;
        use crate::xml_parse::parse_xml_to_nodes;
        let p = ST_Pos::new(10.5, 20.5);
        let xml = p.to_xml();
        assert!(xml.contains("<ST_Pos>"));
        assert!(xml.contains("10.5 20.5"));
        let node = parse_xml_to_nodes(&xml).unwrap();
        let p2 = ST_Pos::from_xml(&node).unwrap();
        assert_eq!(p, p2);
    }

    #[test]
    fn test_xml_element_name() {
        use crate::xml_element::XmlElement;
        let p = ST_Pos::new(0.0, 0.0);
        assert_eq!(p.element_name(), "ST_Pos");
    }
}
