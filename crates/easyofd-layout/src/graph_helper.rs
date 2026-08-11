//! 图形工具，用于快速构建以路径为基础的图形图像数据。
//!
//! 对应 Java: org.ofdrw.layout.engine.GraphHelper

/// 图形工具。
///
/// 对应 Java: ofdrw layout engine GraphHelper。
pub struct GraphHelper;

impl GraphHelper {
    /// 创建矩形轮廓的缩写路径数据（对应 Java: GraphHelper#rect）。
    ///
    /// 返回 SVG 路径命令字符串：`M x y L x+w y L x+w y+h L x y+h Z`。
    #[must_use]
    pub fn rect(x: f64, y: f64, w: f64, h: f64) -> String {
        format!(
            "M {x} {y} L {} {y} L {} {} L {x} {} Z",
            x + w,
            x + w,
            y + h,
            y + h
        )
    }

    /// 创建水平线路径。
    #[must_use]
    pub fn hline(x: f64, y: f64, length: f64) -> String {
        format!("M {x} {y} L {} {y}", x + length)
    }

    /// 创建垂直线路径。
    #[must_use]
    pub fn vline(x: f64, y: f64, length: f64) -> String {
        format!("M {x} {y} L {x} {}", y + length)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rect() {
        let path = GraphHelper::rect(10.0, 20.0, 100.0, 50.0);
        assert!(path.starts_with('M'));
        assert!(path.contains("110 20")); // x+w
        assert!(path.contains("110 70")); // x+w, y+h
        assert!(path.contains("10 70")); // x, y+h
    }

    #[test]
    fn test_hline() {
        let path = GraphHelper::hline(0.0, 10.0, 100.0);
        assert_eq!(path, "M 0 10 L 100 10");
    }

    #[test]
    fn test_vline() {
        let path = GraphHelper::vline(10.0, 0.0, 50.0);
        assert_eq!(path, "M 10 0 L 10 50");
    }
}
