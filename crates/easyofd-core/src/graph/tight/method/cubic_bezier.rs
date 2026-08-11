//! 三次贝塞尔曲线路径方法。
//!
//! 对应 Java: org.ofdrw.core.graph.tight.method.CubicBezier

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
