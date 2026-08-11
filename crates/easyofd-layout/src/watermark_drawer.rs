//! 水印绘制器。
//!
//! 对应 Java: org.ofdrw.layout.edit.WatermarkDrawer

/// 水印旋转方向。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WatermarkRotation {
    /// 从左下到右上。
    #[default]
    LeftBottomToRightTop,
    /// 从左上到右下。
    LeftTopToRightBottom,
}

/// 水印绘制器配置。
///
/// 对应 Java: ofdrw layout edit WatermarkDrawer。
#[derive(Debug, Clone, PartialEq)]
pub struct WatermarkDrawer {
    /// 水印文本。
    pub text: String,
    /// 字体大小（mm），默认 5.0。
    pub font_size: f64,
    /// 字体颜色 `[r, g, b]`，默认灰色。
    pub color: [u8; 3],
    /// 透明度，范围 `[0.0, 1.0]`，默认 0.3。
    pub opacity: f64,
    /// 旋转方向。
    pub rotation: WatermarkRotation,
    /// 水印间距 X（mm），默认 50.0。
    pub spacing_x: f64,
    /// 水印间距 Y（mm），默认 50.0。
    pub spacing_y: f64,
}

impl WatermarkDrawer {
    /// 创建水印绘制器（对应 Java: WatermarkDrawer 的属性设置）。
    #[must_use]
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            font_size: 5.0,
            color: [192, 192, 192],
            opacity: 0.3,
            rotation: WatermarkRotation::LeftBottomToRightTop,
            spacing_x: 50.0,
            spacing_y: 50.0,
        }
    }

    /// 设置字体大小。
    #[must_use]
    pub fn font_size(mut self, size: f64) -> Self {
        self.font_size = size;
        self
    }

    /// 设置颜色。
    #[must_use]
    pub fn color(mut self, r: u8, g: u8, b: u8) -> Self {
        self.color = [r, g, b];
        self
    }

    /// 设置透明度。
    #[must_use]
    pub fn opacity(mut self, opacity: f64) -> Self {
        self.opacity = opacity.clamp(0.0, 1.0);
        self
    }

    /// 设置旋转方向。
    #[must_use]
    pub fn rotation(mut self, rotation: WatermarkRotation) -> Self {
        self.rotation = rotation;
        self
    }

    /// 设置间距。
    #[must_use]
    pub fn spacing(mut self, x: f64, y: f64) -> Self {
        self.spacing_x = x;
        self.spacing_y = y;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let wd = WatermarkDrawer::new("机密");
        assert_eq!(wd.text, "机密");
        assert!((wd.font_size - 5.0).abs() < f64::EPSILON);
        assert!((wd.opacity - 0.3).abs() < f64::EPSILON);
    }

    #[test]
    fn test_builders() {
        let wd = WatermarkDrawer::new("WATERMARK")
            .font_size(8.0)
            .color(255, 0, 0)
            .opacity(0.5)
            .rotation(WatermarkRotation::LeftTopToRightBottom)
            .spacing(100.0, 80.0);
        assert!((wd.font_size - 8.0).abs() < f64::EPSILON);
        assert_eq!(wd.color, [255, 0, 0]);
        assert!((wd.opacity - 0.5).abs() < f64::EPSILON);
        assert_eq!(wd.rotation, WatermarkRotation::LeftTopToRightBottom);
        assert!((wd.spacing_x - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_opacity_clamp() {
        let wd = WatermarkDrawer::new("test").opacity(2.0);
        assert!((wd.opacity - 1.0).abs() < f64::EPSILON);
        let wd = WatermarkDrawer::new("test").opacity(-0.5);
        assert!((wd.opacity - 0.0).abs() < f64::EPSILON);
    }
}
