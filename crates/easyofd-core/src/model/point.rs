//! 坐标点。
//!
//! 对应 Java: org.ofdrw.core.basicType.Point

use crate::page_description::color::CT_Color;

/// 坐标点（ofd:Point），含可选边缘标志与颜色。
///
/// 对应 Java: ofdrw Point。
#[derive(Debug, Clone, PartialEq)]
pub struct Point {
    /// X 坐标。
    pub x: f64,
    /// Y 坐标。
    pub y: f64,
    /// 边缘标志（0 或 1，可选）。
    pub edge_flag: Option<u8>,
    /// 颜色（可选）。
    pub color: Option<CT_Color>,
}

/// 边缘标志值。
pub mod edge_flag {
    /// 闭合路径边缘点。
    pub const CLOSED: u8 = 0;
    /// 非闭合路径边缘点。
    pub const OPEN: u8 = 1;
}

impl Point {
    /// 创建坐标点。
    #[must_use]
    pub fn new(x: f64, y: f64) -> Self {
        Self {
            x,
            y,
            edge_flag: None,
            color: None,
        }
    }

    /// 设置边缘标志。
    #[must_use]
    pub fn with_edge_flag(mut self, flag: u8) -> Self {
        self.edge_flag = Some(flag);
        self
    }

    /// 设置颜色。
    #[must_use]
    pub fn with_color(mut self, color: CT_Color) -> Self {
        self.color = Some(color);
        self
    }

    /// 设置 X 坐标（对应 Java: Point#setX）。
    #[must_use]
    pub fn set_x(mut self, x: f64) -> Self {
        self.x = x;
        self
    }

    /// 设置 Y 坐标（对应 Java: Point#setY）。
    #[must_use]
    pub fn set_y(mut self, y: f64) -> Self {
        self.y = y;
        self
    }
}

impl From<(f64, f64)> for Point {
    fn from((x, y): (f64, f64)) -> Self {
        Self::new(x, y)
    }
}

impl std::fmt::Display for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.x, self.y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f64, b: f64) -> bool {
        (a - b).abs() < f64::EPSILON
    }

    #[test]
    fn test_point_new() {
        let p = Point::new(1.5, 2.5);
        assert!(approx(p.x, 1.5));
        assert!(approx(p.y, 2.5));
        assert!(p.edge_flag.is_none());
    }

    #[test]
    fn test_builders() {
        let p = Point::new(0.0, 0.0)
            .with_edge_flag(edge_flag::OPEN)
            .set_x(10.0)
            .set_y(20.0);
        assert!(approx(p.x, 10.0));
        assert_eq!(p.edge_flag, Some(1));
        assert!(p.color.is_none());
    }

    #[test]
    fn test_from_tuple_and_display() {
        let p = Point::from((3.0, 4.0));
        assert_eq!(p.to_string(), "3 4");
    }
}
