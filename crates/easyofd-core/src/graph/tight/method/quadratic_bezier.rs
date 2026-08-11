//! 二次贝塞尔曲线路径方法。
//!
//! 对应 Java: org.ofdrw.core.graph.tight.method.QuadraticBezier

use crate::xml_element::{XmlElement, XmlElementError, XmlNode};

/// 二次贝塞尔曲线路径方法。
///
/// 图 52 二次贝塞尔曲线结构。
/// 公式: B(t) = (1-t)^2(P0) + 2t(1-t)(P1) + t^2(P2), t in [0,1]
///
/// 对应 Java: org.ofdrw.core.graph.tight.method.QuadraticBezier
#[derive(Debug, Clone, PartialEq)]
pub struct QuadraticBezier {
    /// 控制点 (x, y)。
    pub point1: (f64, f64),
    /// 结束点 (x, y)。
    pub point2: (f64, f64),
}

impl QuadraticBezier {
    /// 创建二次贝塞尔曲线。
    #[must_use]
    pub fn new(x1: f64, y1: f64, x2: f64, y2: f64) -> Self {
        Self {
            point1: (x1, y1),
            point2: (x2, y2),
        }
    }

    /// 序列化为缩写数据字符串（Q 命令格式）。
    #[must_use]
    pub fn to_abbreviated_string(&self) -> String {
        format!(
            "Q {} {} {} {}",
            self.point1.0, self.point1.1, self.point2.0, self.point2.1
        )
    }
}

impl std::fmt::Display for QuadraticBezier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_abbreviated_string())
    }
}

impl XmlElement for QuadraticBezier {
    /// 对应 Java: QuadraticBezier 元素名 "QuadraticBezier"。
    fn element_name(&self) -> &'static str {
        "QuadraticBezier"
    }

    fn attributes(&self) -> Vec<(String, String)> {
        Vec::new()
    }

    /// 覆写 write_xml：文本内容为 Q 命令格式。
    fn write_xml(&self, out: &mut String) {
        out.push_str("<QuadraticBezier>");
        out.push_str(&crate::xml_element::xml_escape(
            &self.to_abbreviated_string(),
        ));
        out.push_str("</QuadraticBezier>");
    }

    fn from_xml(node: &XmlNode) -> Result<Self, XmlElementError> {
        let text = node
            .text
            .as_deref()
            .ok_or_else(|| XmlElementError("QuadraticBezier 缺少文本内容".to_string()))?;
        let parts: Vec<&str> = text.split_whitespace().collect();
        if parts.len() < 5 || parts[0] != "Q" {
            return Err(XmlElementError(format!("QuadraticBezier 格式错误: {text}")));
        }
        let x1: f64 = parts[1]
            .parse()
            .map_err(|e| XmlElementError(format!("解析 QuadraticBezier.x1 失败: {e}")))?;
        let y1: f64 = parts[2]
            .parse()
            .map_err(|e| XmlElementError(format!("解析 QuadraticBezier.y1 失败: {e}")))?;
        let x2: f64 = parts[3]
            .parse()
            .map_err(|e| XmlElementError(format!("解析 QuadraticBezier.x2 失败: {e}")))?;
        let y2: f64 = parts[4]
            .parse()
            .map_err(|e| XmlElementError(format!("解析 QuadraticBezier.y2 失败: {e}")))?;
        Ok(Self::new(x1, y1, x2, y2))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::xml_parse::parse_xml_to_nodes;

    #[test]
    fn quadratic_bezier_new() {
        let qb = QuadraticBezier::new(1.0, 2.0, 3.0, 4.0);
        assert_eq!(qb.point1, (1.0, 2.0));
        assert_eq!(qb.point2, (3.0, 4.0));
    }

    #[test]
    fn quadratic_bezier_to_string() {
        let qb = QuadraticBezier::new(5.0, 5.0, 10.0, 0.0);
        assert_eq!(qb.to_abbreviated_string(), "Q 5 5 10 0");
    }

    #[test]
    fn quadratic_bezier_display() {
        let qb = QuadraticBezier::new(0.0, 0.0, 1.0, 1.0);
        assert_eq!(format!("{qb}"), "Q 0 0 1 1");
    }

    #[test]
    fn quadratic_bezier_clone_eq() {
        let qb = QuadraticBezier::new(1.0, 2.0, 3.0, 4.0);
        let qb2 = qb.clone();
        assert_eq!(qb, qb2);
    }

    #[test]
    fn test_xml_element_name() {
        let qb = QuadraticBezier::new(1.0, 2.0, 3.0, 4.0);
        assert_eq!(qb.element_name(), "QuadraticBezier");
    }

    #[test]
    fn test_xml_element_roundtrip() {
        let qb = QuadraticBezier::new(5.0, 5.0, 10.0, 0.0);
        let xml = qb.to_xml();
        assert!(xml.contains("<QuadraticBezier>"));
        assert!(xml.contains("Q 5 5 10 0"));
        let node = parse_xml_to_nodes(&xml).unwrap();
        let qb2 = QuadraticBezier::from_xml(&node).unwrap();
        assert_eq!(qb, qb2);
    }
}
