//! 线段路径方法。
//!
//! 对应 Java: org.ofdrw.core.graph.tight.method.Line

use crate::xml_element::{XmlElement, XmlElementError, XmlNode};

/// 线段路径方法。
///
/// 图 51 线段结构。从当前点绘制直线到指定结束点。
///
/// 对应 Java: org.ofdrw.core.graph.tight.method.Line
#[derive(Debug, Clone, PartialEq)]
pub struct Line {
    /// 线段的结束点 (x, y)。
    pub point: (f64, f64),
}

impl Line {
    /// 创建线段。
    #[must_use]
    pub fn new(x: f64, y: f64) -> Self {
        Self { point: (x, y) }
    }

    /// 序列化为缩写数据字符串（L 命令格式）。
    #[must_use]
    pub fn to_abbreviated_string(&self) -> String {
        format!("L {} {}", self.point.0, self.point.1)
    }
}

impl std::fmt::Display for Line {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_abbreviated_string())
    }
}

impl XmlElement for Line {
    /// 对应 Java: Line 元素名 "Line"。
    fn element_name(&self) -> &'static str {
        "Line"
    }

    fn attributes(&self) -> Vec<(String, String)> {
        Vec::new()
    }

    /// 覆写 write_xml：文本内容为 L 命令格式。
    fn write_xml(&self, out: &mut String) {
        out.push_str("<Line>");
        out.push_str(&crate::xml_element::xml_escape(
            &self.to_abbreviated_string(),
        ));
        out.push_str("</Line>");
    }

    fn from_xml(node: &XmlNode) -> Result<Self, XmlElementError> {
        let text = node
            .text
            .as_deref()
            .ok_or_else(|| XmlElementError("Line 缺少文本内容".to_string()))?;
        let parts: Vec<&str> = text.split_whitespace().collect();
        if parts.len() < 3 || parts[0] != "L" {
            return Err(XmlElementError(format!("Line 格式错误: {text}")));
        }
        let x: f64 = parts[1]
            .parse()
            .map_err(|e| XmlElementError(format!("解析 Line.x 失败: {e}")))?;
        let y: f64 = parts[2]
            .parse()
            .map_err(|e| XmlElementError(format!("解析 Line.y 失败: {e}")))?;
        Ok(Self::new(x, y))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::xml_parse::parse_xml_to_nodes;

    #[test]
    fn line_new() {
        let l = Line::new(10.0, 20.0);
        assert_eq!(l.point, (10.0, 20.0));
    }

    #[test]
    fn line_to_string() {
        let l = Line::new(100.5, 50.3);
        assert_eq!(l.to_abbreviated_string(), "L 100.5 50.3");
    }

    #[test]
    fn line_display() {
        let l = Line::new(1.0, 2.0);
        assert_eq!(format!("{l}"), "L 1 2");
    }

    #[test]
    fn line_clone_eq() {
        let l = Line::new(3.0, 4.0);
        let l2 = l.clone();
        assert_eq!(l, l2);
    }

    #[test]
    fn test_xml_element_name() {
        let l = Line::new(1.0, 2.0);
        assert_eq!(l.element_name(), "Line");
    }

    #[test]
    fn test_xml_element_roundtrip() {
        let l = Line::new(10.5, 20.5);
        let xml = l.to_xml();
        assert!(xml.contains("<Line>"));
        assert!(xml.contains("L 10.5 20.5"));
        let node = parse_xml_to_nodes(&xml).unwrap();
        let l2 = Line::from_xml(&node).unwrap();
        assert!((l.point.0 - l2.point.0).abs() < f64::EPSILON);
        assert!((l.point.1 - l2.point.1).abs() < f64::EPSILON);
    }
}
