//! 边框。
//!
//! 对应 Java: org.ofdrw.core.image.Border

/// 边框样式（ofdrw layout Border）。
///
/// 对应 Java: ofdrw Border，含线宽、圆角半径、虚线偏移/模式与边框颜色。
#[derive(Debug, Clone, PartialEq)]
pub struct Border {
    /// 线宽（默认 0.353，约 0.01 mm）。
    pub line_width: f64,
    /// 水平圆角半径。
    pub horizontal_corner_radius: Option<f64>,
    /// 垂直圆角半径。
    pub vertical_corner_radius: Option<f64>,
    /// 虚线偏移。
    pub dash_offset: Option<f64>,
    /// 虚线模式（线段长度序列）。
    pub dash_pattern: Vec<f64>,
    /// 边框颜色（RGB，可选）。
    pub color: Option<[u8; 3]>,
}

impl Default for Border {
    fn default() -> Self {
        Self {
            line_width: 0.353,
            horizontal_corner_radius: None,
            vertical_corner_radius: None,
            dash_offset: None,
            dash_pattern: Vec::new(),
            color: None,
        }
    }
}

impl Border {
    /// 创建默认边框（对应 Java: Border()）。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置线宽（对应 Java: Border#setLineWidth）。
    #[must_use]
    pub fn line_width(mut self, width: f64) -> Self {
        self.line_width = width;
        self
    }

    /// 设置水平圆角半径（对应 Java: setHorizonalCornerRadius）。
    #[must_use]
    pub fn horizontal_corner_radius(mut self, radius: f64) -> Self {
        self.horizontal_corner_radius = Some(radius);
        self
    }

    /// 设置垂直圆角半径（对应 Java: setVerticalCornerRadius）。
    #[must_use]
    pub fn vertical_corner_radius(mut self, radius: f64) -> Self {
        self.vertical_corner_radius = Some(radius);
        self
    }

    /// 设置虚线模式（对应 Java: Border#setDashPattern）。
    #[must_use]
    pub fn dash_pattern(mut self, pattern: &[f64]) -> Self {
        self.dash_pattern = pattern.to_vec();
        self
    }

    /// 设置边框颜色（对应 Java: Border#setBorderColor）。
    #[must_use]
    pub fn color(mut self, r: u8, g: u8, b: u8) -> Self {
        self.color = Some([r, g, b]);
        self
    }

    /// 是否为实线（无虚线模式）。
    #[must_use]
    pub fn is_solid(&self) -> bool {
        self.dash_pattern.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let b = Border::new();
        assert!((b.line_width - 0.353).abs() < f64::EPSILON);
        assert!(b.is_solid());
        assert!(b.color.is_none());
    }

    #[test]
    fn test_builders() {
        let b = Border::new()
            .line_width(1.0)
            .horizontal_corner_radius(2.0)
            .vertical_corner_radius(3.0)
            .dash_pattern(&[4.0, 2.0])
            .color(255, 0, 0);
        assert!((b.line_width - 1.0).abs() < f64::EPSILON);
        assert_eq!(b.horizontal_corner_radius, Some(2.0));
        assert_eq!(b.dash_pattern, vec![4.0, 2.0]);
        assert!(!b.is_solid());
        assert_eq!(b.color, Some([255, 0, 0]));
    }
}
