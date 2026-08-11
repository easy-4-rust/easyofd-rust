//! 移动路径方法。
//!
//! 对应 Java: org.ofdrw.core.graph.tight.method.Move

use crate::xml_element::{XmlElement, XmlElementError, XmlNode};

/// 移动路径方法。
///
/// 用于表示到新的绘制点指令。
///
/// 对应 Java: org.ofdrw.core.graph.tight.method.Move
#[derive(Debug, Clone, PartialEq)]
pub struct Move {
    /// 移动后新的当前绘制点 (x, y)。
    pub point: (f64, f64),
}

impl Move {
    /// 创建移动命令。
    #[must_use]
    pub fn new(x: f64, y: f64) -> Self {
        Self { point: (x, y) }
    }

    /// 序列化为缩写数据字符串（M 命令格式）。
    #[must_use]
    pub fn to_abbreviated_string(&self) -> String {
        format!("M {} {}", self.point.0, self.point.1)
    }
}

impl std::fmt::Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_abbreviated_string())
    }
}

impl XmlElement for Move {
    /// 对应 Java: Move 元素名 "Move"。
    fn element_name(&self) -> &'static str {
        "Move"
    }

    fn attributes(&self) -> Vec<(String, String)> {
        Vec::new()
    }

    /// 覆写 write_xml：文本内容为 M 命令格式。
    fn write_xml(&self, out: &mut String) {
        out.push_str("<Move>");
        out.push_str(&crate::xml_element::xml_escape(
            &self.to_abbreviated_string(),
        ));
        out.push_str("</Move>");
    }

    fn from_xml(node: &XmlNode) -> Result<Self, XmlElementError> {
        let text = node
            .text
            .as_deref()
            .ok_or_else(|| XmlElementError("Move 缺少文本内容".to_string()))?;
        let parts: Vec<&str> = text.split_whitespace().collect();
        if parts.len() < 3 || parts[0] != "M" {
            return Err(XmlElementError(format!("Move 格式错误: {text}")));
        }
        let x: f64 = parts[1]
            .parse()
            .map_err(|e| XmlElementError(format!("解析 Move.x 失败: {e}")))?;
        let y: f64 = parts[2]
            .parse()
            .map_err(|e| XmlElementError(format!("解析 Move.y 失败: {e}")))?;
        Ok(Self::new(x, y))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::xml_parse::parse_xml_to_nodes;

    #[test]
    fn move_new() {
        let m = Move::new(10.0, 20.0);
        assert_eq!(m.point, (10.0, 20.0));
    }

    #[test]
    fn move_to_string() {
        let m = Move::new(0.0, 0.0);
        assert_eq!(m.to_abbreviated_string(), "M 0 0");
    }

    #[test]
    fn move_display() {
        let m = Move::new(5.5, 3.3);
        let s = format!("{m}");
        assert!(s.contains("M 5.5 3.3"));
    }

    #[test]
    fn move_clone_eq() {
        let m = Move::new(1.0, 2.0);
        let m2 = m.clone();
        assert_eq!(m, m2);
    }

    #[test]
    fn test_xml_element_name() {
        let m = Move::new(1.0, 2.0);
        assert_eq!(m.element_name(), "Move");
    }

    #[test]
    fn test_xml_element_roundtrip() {
        let m = Move::new(10.5, 20.5);
        let xml = m.to_xml();
        assert!(xml.contains("<Move>"));
        assert!(xml.contains("M 10.5 20.5"));
        let node = parse_xml_to_nodes(&xml).unwrap();
        let m2 = Move::from_xml(&node).unwrap();
        assert!((m.point.0 - m2.point.0).abs() < f64::EPSILON);
        assert!((m.point.1 - m2.point.1).abs() < f64::EPSILON);
    }
}
