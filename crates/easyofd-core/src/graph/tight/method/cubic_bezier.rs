//! 三次贝塞尔曲线路径方法。
//!
//! 对应 Java: org.ofdrw.core.graph.tight.method.CubicBezier

use crate::xml_element::{XmlElement, XmlElementError, XmlNode};

/// 三次贝塞尔曲线路径方法。
///
/// 图 53 三次贝塞尔曲线结构。
/// 公式: B(t) = (1-t)^3(P0) + 3t(1-t)^2(P1) + 3t^2(1-t)(P2) + t^3(P3), t in [0,1]
///
/// 对应 Java: org.ofdrw.core.graph.tight.method.CubicBezier
#[derive(Debug, Clone, PartialEq)]
pub struct CubicBezier {
    /// 第一个控制点 (x, y)。
    pub point1: (f64, f64),
    /// 第二个控制点 (x, y)。
    pub point2: (f64, f64),
    /// 结束点 (x, y)。
    pub point3: (f64, f64),
}

impl CubicBezier {
    /// 创建三次贝塞尔曲线。
    #[must_use]
    pub fn new(x1: f64, y1: f64, x2: f64, y2: f64, x3: f64, y3: f64) -> Self {
        Self {
            point1: (x1, y1),
            point2: (x2, y2),
            point3: (x3, y3),
        }
    }

    /// 序列化为缩写数据字符串（B 命令格式）。
    #[must_use]
    pub fn to_abbreviated_string(&self) -> String {
        format!(
            "B {} {} {} {} {} {}",
            self.point1.0,
            self.point1.1,
            self.point2.0,
            self.point2.1,
            self.point3.0,
            self.point3.1
        )
    }
}

impl std::fmt::Display for CubicBezier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_abbreviated_string())
    }
}

impl XmlElement for CubicBezier {
    /// 对应 Java: CubicBezier 元素名 "CubicBezier"。
    fn element_name(&self) -> &'static str {
        "CubicBezier"
    }

    fn attributes(&self) -> Vec<(String, String)> {
        Vec::new()
    }

    /// 覆写 write_xml：文本内容为 B 命令格式。
    fn write_xml(&self, out: &mut String) {
        out.push_str("<CubicBezier>");
        out.push_str(&crate::xml_element::xml_escape(
            &self.to_abbreviated_string(),
        ));
        out.push_str("</CubicBezier>");
    }

    fn from_xml(node: &XmlNode) -> Result<Self, XmlElementError> {
        let text = node
            .text
            .as_deref()
            .ok_or_else(|| XmlElementError("CubicBezier 缺少文本内容".to_string()))?;
        let parts: Vec<&str> = text.split_whitespace().collect();
        if parts.len() < 7 || parts[0] != "B" {
            return Err(XmlElementError(format!("CubicBezier 格式错误: {text}")));
        }
        let x1: f64 = parts[1]
            .parse()
            .map_err(|e| XmlElementError(format!("解析 CubicBezier.x1 失败: {e}")))?;
        let y1: f64 = parts[2]
            .parse()
            .map_err(|e| XmlElementError(format!("解析 CubicBezier.y1 失败: {e}")))?;
        let x2: f64 = parts[3]
            .parse()
            .map_err(|e| XmlElementError(format!("解析 CubicBezier.x2 失败: {e}")))?;
        let y2: f64 = parts[4]
            .parse()
            .map_err(|e| XmlElementError(format!("解析 CubicBezier.y2 失败: {e}")))?;
        let x3: f64 = parts[5]
            .parse()
            .map_err(|e| XmlElementError(format!("解析 CubicBezier.x3 失败: {e}")))?;
        let y3: f64 = parts[6]
            .parse()
            .map_err(|e| XmlElementError(format!("解析 CubicBezier.y3 失败: {e}")))?;
        Ok(Self::new(x1, y1, x2, y2, x3, y3))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::xml_parse::parse_xml_to_nodes;

    #[test]
    fn cubic_bezier_new() {
        let cb = CubicBezier::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0);
        assert_eq!(cb.point1, (1.0, 2.0));
        assert_eq!(cb.point2, (3.0, 4.0));
        assert_eq!(cb.point3, (5.0, 6.0));
    }

    #[test]
    fn cubic_bezier_to_string() {
        let cb = CubicBezier::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0);
        assert_eq!(cb.to_abbreviated_string(), "B 1 2 3 4 5 6");
    }

    #[test]
    fn cubic_bezier_display() {
        let cb = CubicBezier::new(0.0, 0.0, 10.0, 10.0, 20.0, 0.0);
        let s = format!("{cb}");
        assert!(s.starts_with("B 0 0 10 10 20 0"));
    }

    #[test]
    fn cubic_bezier_clone_eq() {
        let cb = CubicBezier::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0);
        let cb2 = cb.clone();
        assert_eq!(cb, cb2);
    }

    #[test]
    fn test_xml_element_name() {
        let cb = CubicBezier::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0);
        assert_eq!(cb.element_name(), "CubicBezier");
    }

    #[test]
    fn test_xml_element_roundtrip() {
        let cb = CubicBezier::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0);
        let xml = cb.to_xml();
        assert!(xml.contains("<CubicBezier>"));
        assert!(xml.contains("B 1 2 3 4 5 6"));
        let node = parse_xml_to_nodes(&xml).unwrap();
        let cb2 = CubicBezier::from_xml(&node).unwrap();
        assert_eq!(cb, cb2);
    }
}
