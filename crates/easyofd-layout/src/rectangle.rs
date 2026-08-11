//! 布局矩形。
//!
//! 对应 Java: org.ofdrw.layout.Rectangle

/// 布局矩形（x, y, width, height）。
///
/// 对应 Java: ofdrw Rectangle。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rectangle {
    /// 左上角 X。
    pub x: f64,
    /// 左上角 Y。
    pub y: f64,
    /// 宽度。
    pub width: f64,
    /// 高度。
    pub height: f64,
}

impl Rectangle {
    /// 空矩形（0 x 0）。
    pub const EMPTY: Self = Self {
        x: 0.0,
        y: 0.0,
        width: 0.0,
        height: 0.0,
    };

    /// 创建矩形（对应 Java: Rectangle(x, y, width, height)）。
    #[must_use]
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// 从尺寸创建矩形（左上角为原点，对应 Java: Rectangle(width, height)）。
    #[must_use]
    pub fn from_size(width: f64, height: f64) -> Self {
        Self::new(0.0, 0.0, width, height)
    }

    /// 宽度增量（对应 Java: addToWidth）。
    #[must_use]
    pub fn add_to_width(mut self, delta: f64) -> Self {
        self.width += delta;
        self
    }

    /// 高度增量（对应 Java: addToHeight）。
    #[must_use]
    pub fn add_to_height(mut self, delta: f64) -> Self {
        self.height += delta;
        self
    }

    /// 是否为空矩形。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.width == 0.0 || self.height == 0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f64, b: f64) -> bool {
        (a - b).abs() < f64::EPSILON
    }

    #[test]
    fn test_rectangle_new() {
        let r = Rectangle::new(1.0, 2.0, 100.0, 50.0);
        assert!(approx(r.x, 1.0));
        assert!(approx(r.width, 100.0));
        assert!(!r.is_empty());
    }

    #[test]
    fn test_from_size_and_empty() {
        let r = Rectangle::from_size(210.0, 297.0);
        assert!(approx(r.x, 0.0));
        assert!(approx(r.y, 0.0));
        assert!(Rectangle::EMPTY.is_empty());
        assert!(Rectangle::from_size(0.0, 10.0).is_empty());
    }

    #[test]
    fn test_add_delta() {
        let r = Rectangle::from_size(100.0, 50.0)
            .add_to_width(10.0)
            .add_to_height(-5.0);
        assert!(approx(r.width, 110.0));
        assert!(approx(r.height, 45.0));
    }
}
