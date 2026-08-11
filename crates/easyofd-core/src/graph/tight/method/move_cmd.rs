//! 移动路径方法。
//!
//! 对应 Java: org.ofdrw.core.graph.tight.method.Move

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
