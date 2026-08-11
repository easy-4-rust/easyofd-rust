//! 二次贝塞尔曲线路径方法。
//!
//! 对应 Java: org.ofdrw.core.graph.tight.method.QuadraticBezier

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
