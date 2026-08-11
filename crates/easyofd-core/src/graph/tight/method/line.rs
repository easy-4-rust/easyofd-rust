//! 线段路径方法。
//!
//! 对应 Java: org.ofdrw.core.graph.tight.method.Line

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
