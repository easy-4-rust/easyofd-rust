//! 图像边框颜色。
//!
//! 对应 Java: org.ofdrw.core.image.BorderColor

/// 图像边框颜色。
///
/// 对应 Java: org.ofdrw.core.image.BorderColor
#[derive(Debug, Clone, Default)]
pub struct BorderColor {
    /// 边框颜色值。
    pub value: Option<u32>,
    /// 边框宽度。
    pub width: Option<f64>,
}

impl BorderColor {
    /// 创建边框颜色。
    #[must_use]
    pub fn new(value: u32) -> Self {
        Self {
            value: Some(value),
            width: None,
        }
    }

    /// 设置边框宽度。
    #[must_use]
    pub fn width(mut self, width: f64) -> Self {
        self.width = Some(width);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn border_color_new() {
        let bc = BorderColor::new(0x0000_0000);
        assert_eq!(bc.value, Some(0x0000_0000));
    }

    #[test]
    fn border_color_builder() {
        let bc = BorderColor::new(0x00FF_0000).width(2.0);
        assert_eq!(bc.width, Some(2.0));
    }
}
