//! 圆弧路径方法。
//!
//! 对应 Java: org.ofdrw.core.graph.tight.method.Arc

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
