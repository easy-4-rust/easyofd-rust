//! 路径绘制点。
//!
//! 对应 Java: org.ofdrw.converter.point.PathPoint

/// 路径绘制点，表示 OFD 路径命令中的一个操作及其坐标参数。
///
/// 对应 Java `PathPoint`。OFD 路径使用压缩缩写数据（AbbreviatedData）描述，
/// 每个操作符（M/L/C/B/Q/A/S）携带不同的坐标参数。
///
/// ## 操作符含义
///
/// | 操作符 | 含义 | 使用的字段 |
/// |--------|------|-----------|
/// | `M`    | 移动到 | `x1, y1` |
/// | `L`    | 直线到 | `x1, y1` |
/// | `C`    | 闭合 | 无 |
/// | `S`    | 二次贝塞尔起始 | `x1, y1` |
/// | `B`    | 三次贝塞尔曲线 | `x1..y3` |
/// | `Q`    | 二次贝塞尔曲线 | `x1, y1, x2, y2` |
/// | `A`    | 椭圆弧 | `rx, ry, rotation, arc, sweep, x, y` |
#[derive(Debug, Clone, PartialEq)]
pub struct PathPoint {
    /// 操作符类型（"M", "L", "C", "S", "B", "Q", "A"）。
    pub point_type: String,
    /// 控制点 1 的 X 坐标 / 弧线 X 半径。
    pub x1: f32,
    /// 控制点 1 的 Y 坐标 / 弧线 Y 半径。
    pub y1: f32,
    /// 控制点 2 的 X 坐标 / 旋转角度。
    pub x2: f32,
    /// 控制点 2 的 Y 坐标 / 弧线弧度。
    pub y2: f32,
    /// 终点 X 坐标 / 弧线扫掠角度。
    pub x3: f32,
    /// 终点 Y 坐标。
    pub y3: f32,
    /// 椭圆弧 X 半径（操作符 A 专用）。
    pub rx: f32,
    /// 椭圆弧 Y 半径（操作符 A 专用）。
    pub ry: f32,
    /// 椭圆弧旋转角度（操作符 A 专用）。
    pub rotation: f32,
    /// 椭圆弧弧度标志（操作符 A 专用）。
    pub arc: f32,
    /// 椭圆弧扫掠方向（操作符 A 专用）。
    pub sweep: f32,
    /// 终点 X（操作符 A 专用）。
    pub x: f32,
    /// 终点 Y（操作符 A 专用）。
    pub y: f32,
}

impl PathPoint {
    /// 创建直线/移动/起始点（M/L/S/C 操作符）。
    ///
    /// # 参数
    /// - `point_type`：操作符
    /// - `x1, y1`：目标坐标
    pub fn new(point_type: impl Into<String>, x1: f32, y1: f32) -> Self {
        Self {
            point_type: point_type.into(),
            x1,
            y1,
            x2: 0.0,
            y2: 0.0,
            x3: 0.0,
            y3: 0.0,
            rx: 0.0,
            ry: 0.0,
            rotation: 0.0,
            arc: 0.0,
            sweep: 0.0,
            x: 0.0,
            y: 0.0,
        }
    }

    /// 创建三次贝塞尔曲线点（B 操作符）。
    ///
    /// # 参数
    /// - `x1, y1`：第一个控制点
    /// - `x2, y2`：第二个控制点
    /// - `x3, y3`：终点
    pub fn bezier(x1: f32, y1: f32, x2: f32, y2: f32, x3: f32, y3: f32) -> Self {
        Self {
            point_type: "B".to_string(),
            x1,
            y1,
            x2,
            y2,
            x3,
            y3,
            rx: 0.0,
            ry: 0.0,
            rotation: 0.0,
            arc: 0.0,
            sweep: 0.0,
            x: 0.0,
            y: 0.0,
        }
    }

    /// 创建二次贝塞尔曲线点（Q 操作符）。
    ///
    /// # 参数
    /// - `x1, y1`：控制点
    /// - `x2, y2`：终点
    pub fn quad_bezier(x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        Self {
            point_type: "Q".to_string(),
            x1,
            y1,
            x2,
            y2,
            x3: 0.0,
            y3: 0.0,
            rx: 0.0,
            ry: 0.0,
            rotation: 0.0,
            arc: 0.0,
            sweep: 0.0,
            x: 0.0,
            y: 0.0,
        }
    }

    /// 创建椭圆弧点（A 操作符）。
    ///
    /// # 参数
    /// - `rx, ry`：椭圆半径
    /// - `rotation`：旋转角度
    /// - `arc`：弧度标志
    /// - `sweep`：扫掠方向
    /// - `x, y`：终点坐标
    pub fn arc(rx: f32, ry: f32, rotation: f32, arc: f32, sweep: f32, x: f32, y: f32) -> Self {
        Self {
            point_type: "A".to_string(),
            x1: 0.0,
            y1: 0.0,
            x2: 0.0,
            y2: 0.0,
            x3: 0.0,
            y3: 0.0,
            rx,
            ry,
            rotation,
            arc,
            sweep,
            x,
            y,
        }
    }

    /// 创建闭合路径点（C 操作符）。
    pub fn close() -> Self {
        Self {
            point_type: "C".to_string(),
            x1: 0.0,
            y1: 0.0,
            x2: 0.0,
            y2: 0.0,
            x3: 0.0,
            y3: 0.0,
            rx: 0.0,
            ry: 0.0,
            rotation: 0.0,
            arc: 0.0,
            sweep: 0.0,
            x: 0.0,
            y: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_move_to() {
        let p = PathPoint::new("M", 10.0, 20.0);
        assert_eq!(p.point_type, "M");
        assert_eq!(p.x1, 10.0);
        assert_eq!(p.y1, 20.0);
    }

    #[test]
    fn test_new_line_to() {
        let p = PathPoint::new("L", 5.0, 15.0);
        assert_eq!(p.point_type, "L");
        assert_eq!(p.x1, 5.0);
    }

    #[test]
    fn test_bezier() {
        let p = PathPoint::bezier(1.0, 2.0, 3.0, 4.0, 5.0, 6.0);
        assert_eq!(p.point_type, "B");
        assert_eq!(p.x1, 1.0);
        assert_eq!(p.y2, 4.0);
        assert_eq!(p.x3, 5.0);
        assert_eq!(p.y3, 6.0);
    }

    #[test]
    fn test_quad_bezier() {
        let p = PathPoint::quad_bezier(1.0, 2.0, 3.0, 4.0);
        assert_eq!(p.point_type, "Q");
        assert_eq!(p.x1, 1.0);
        assert_eq!(p.y1, 2.0);
        assert_eq!(p.x2, 3.0);
        assert_eq!(p.y2, 4.0);
    }

    #[test]
    fn test_arc() {
        let p = PathPoint::arc(10.0, 20.0, 45.0, 1.0, 0.0, 30.0, 40.0);
        assert_eq!(p.point_type, "A");
        assert_eq!(p.rx, 10.0);
        assert_eq!(p.ry, 20.0);
        assert_eq!(p.rotation, 45.0);
        assert_eq!(p.x, 30.0);
        assert_eq!(p.y, 40.0);
    }

    #[test]
    fn test_close() {
        let p = PathPoint::close();
        assert_eq!(p.point_type, "C");
    }

    #[test]
    fn test_clone() {
        let p = PathPoint::new("M", 1.0, 2.0);
        let p2 = p.clone();
        assert_eq!(p, p2);
    }
}
