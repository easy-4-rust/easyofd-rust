//! 圆弧路径方法。
//!
//! 对应 Java: org.ofdrw.core.graph.tight.method.Arc

use crate::xml_element::{XmlElement, XmlElementError, XmlNode};

/// 圆弧路径方法。
///
/// 图 56 圆弧的结构。用于描述椭圆弧线段。
///
/// 对应 Java: org.ofdrw.core.graph.tight.method.Arc
#[derive(Debug, Clone, PartialEq)]
pub struct Arc {
    /// 椭圆长轴半径。
    pub rx: f64,
    /// 椭圆短轴半径。
    pub ry: f64,
    /// 旋转角度（度），正值顺时针。
    pub rotation_angle: f64,
    /// 是否大圆弧（角度 > 180）。
    pub large_arc: bool,
    /// 是否顺时针方向。
    pub sweep_direction: bool,
    /// 结束点 (x, y)。
    pub end_point: (f64, f64),
}

impl Arc {
    /// 创建圆弧。
    #[must_use]
    pub fn new(
        rx: f64,
        ry: f64,
        rotation_angle: f64,
        large_arc: bool,
        sweep_direction: bool,
        end_x: f64,
        end_y: f64,
    ) -> Self {
        Self {
            rx,
            ry,
            rotation_angle: rotation_angle % 360.0,
            large_arc,
            sweep_direction,
            end_point: (end_x, end_y),
        }
    }

    /// 序列化为缩写数据字符串（A 命令格式）。
    #[must_use]
    pub fn to_abbreviated_string(&self) -> String {
        let large = i32::from(self.large_arc);
        let sweep = i32::from(self.sweep_direction);
        format!(
            "A {} {} {} {} {} {} {}",
            self.rx, self.ry, self.rotation_angle, large, sweep, self.end_point.0, self.end_point.1
        )
    }
}

impl std::fmt::Display for Arc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_abbreviated_string())
    }
}

impl XmlElement for Arc {
    /// 对应 Java: Arc 元素名 "Arc"。
    fn element_name(&self) -> &'static str {
        "Arc"
    }

    fn attributes(&self) -> Vec<(String, String)> {
        Vec::new()
    }

    /// 覆写 write_xml：文本内容为 A 命令格式。
    fn write_xml(&self, out: &mut String) {
        out.push_str("<Arc>");
        out.push_str(&crate::xml_element::xml_escape(
            &self.to_abbreviated_string(),
        ));
        out.push_str("</Arc>");
    }

    fn from_xml(node: &XmlNode) -> Result<Self, XmlElementError> {
        let text = node
            .text
            .as_deref()
            .ok_or_else(|| XmlElementError("Arc 缺少文本内容".to_string()))?;
        // 解析 "A rx ry angle large sweep x y"
        let parts: Vec<&str> = text.split_whitespace().collect();
        if parts.len() < 8 || parts[0] != "A" {
            return Err(XmlElementError(format!("Arc 格式错误: {text}")));
        }
        let rx: f64 = parts[1]
            .parse()
            .map_err(|e| XmlElementError(format!("解析 Arc.rx 失败: {e}")))?;
        let ry: f64 = parts[2]
            .parse()
            .map_err(|e| XmlElementError(format!("解析 Arc.ry 失败: {e}")))?;
        let rotation_angle: f64 = parts[3]
            .parse()
            .map_err(|e| XmlElementError(format!("解析 Arc.rotation_angle 失败: {e}")))?;
        let large_arc: i32 = parts[4]
            .parse()
            .map_err(|e| XmlElementError(format!("解析 Arc.large_arc 失败: {e}")))?;
        let sweep: i32 = parts[5]
            .parse()
            .map_err(|e| XmlElementError(format!("解析 Arc.sweep 失败: {e}")))?;
        let end_x: f64 = parts[6]
            .parse()
            .map_err(|e| XmlElementError(format!("解析 Arc.end_x 失败: {e}")))?;
        let end_y: f64 = parts[7]
            .parse()
            .map_err(|e| XmlElementError(format!("解析 Arc.end_y 失败: {e}")))?;
        Ok(Self::new(
            rx,
            ry,
            rotation_angle,
            large_arc != 0,
            sweep != 0,
            end_x,
            end_y,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::xml_parse::parse_xml_to_nodes;

    #[test]
    fn arc_new() {
        let a = Arc::new(5.0, 5.0, 0.0, true, false, 10.0, 10.0);
        assert!((a.rx - 5.0).abs() < f64::EPSILON);
        assert!((a.ry - 5.0).abs() < f64::EPSILON);
        assert!(a.large_arc);
        assert!(!a.sweep_direction);
        assert!((a.end_point.0 - 10.0).abs() < f64::EPSILON);
    }

    #[test]
    fn arc_rotation_modulo() {
        let a = Arc::new(1.0, 1.0, 720.0, false, false, 0.0, 0.0);
        assert!((a.rotation_angle - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn arc_to_string() {
        let a = Arc::new(5.0, 5.0, 0.0, true, false, 10.0, 10.0);
        let s = a.to_abbreviated_string();
        assert!(s.starts_with("A 5 5 0 1 0 10 10"));
    }

    #[test]
    fn arc_display() {
        let a = Arc::new(1.0, 2.0, 45.0, false, true, 3.0, 4.0);
        let s = format!("{a}");
        assert!(s.contains("A 1 2 45 0 1 3 4"));
    }

    #[test]
    fn arc_clone_eq() {
        let a = Arc::new(1.0, 2.0, 30.0, true, true, 5.0, 6.0);
        let b = a.clone();
        assert!((a.rx - b.rx).abs() < f64::EPSILON);
    }

    #[test]
    fn test_xml_element_name() {
        let a = Arc::new(1.0, 2.0, 0.0, false, false, 3.0, 4.0);
        assert_eq!(a.element_name(), "Arc");
    }

    #[test]
    fn test_xml_element_roundtrip() {
        let a = Arc::new(5.0, 5.0, 45.0, true, false, 10.0, 10.0);
        let xml = a.to_xml();
        assert!(xml.contains("<Arc>"));
        assert!(xml.contains("A 5 5 45 1 0 10 10"));
        let node = parse_xml_to_nodes(&xml).unwrap();
        let a2 = Arc::from_xml(&node).unwrap();
        assert!((a.rx - a2.rx).abs() < f64::EPSILON);
        assert!((a.ry - a2.ry).abs() < f64::EPSILON);
        assert_eq!(a.large_arc, a2.large_arc);
        assert_eq!(a.sweep_direction, a2.sweep_direction);
    }
}
