//! AWT 转换器配置。
//!
//! 对应 Java: org.ofdrw.converter.AWTMaker.Config
//!
//! Java 版 `AWTMaker` 内部类 `Config` 存储渲染配置参数。
//! Rust 版提供等价的配置结构，用于 OFD 到图片/SVG 转换时的渲染参数。

/// OFD 渲染配置。
///
/// 对应 Java: `org.ofdrw.converter.AWTMaker.Config`（内部类）
///
/// 控制 OFD 页面渲染行为的参数集合。
#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)] // 与 Java AWTMaker.Config 字段一一对应
pub struct Config {
    /// 渲染 DPI（每英寸点数）。
    pub dpi: f32,
    /// 页面缩放比例。
    pub scale: f32,
    /// 是否启用抗锯齿。
    pub anti_aliasing: bool,
    /// 是否渲染文本。
    pub render_text: bool,
    /// 是否渲染图片。
    pub render_image: bool,
    /// 是否渲染路径（线条、形状）。
    pub render_path: bool,
    /// 背景颜色（ARGB）。
    pub background_color: u32,
}

impl Config {
    /// 创建默认配置。
    ///
    /// 对应 Java: `AWTMaker` 的默认配置值。
    #[must_use]
    pub fn new() -> Self {
        Self {
            dpi: 72.0,
            scale: 1.0,
            anti_aliasing: true,
            render_text: true,
            render_image: true,
            render_path: true,
            background_color: 0xFFFF_FFFF, // 白色
        }
    }

    /// 设置 DPI。
    #[must_use]
    pub fn with_dpi(mut self, dpi: f32) -> Self {
        self.dpi = dpi;
        self
    }

    /// 设置缩放比例。
    #[must_use]
    pub fn with_scale(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
    }

    /// 设置抗锯齿。
    #[must_use]
    pub fn with_anti_aliasing(mut self, enabled: bool) -> Self {
        self.anti_aliasing = enabled;
        self
    }

    /// 设置是否渲染文本。
    #[must_use]
    pub fn with_render_text(mut self, enabled: bool) -> Self {
        self.render_text = enabled;
        self
    }

    /// 设置是否渲染图片。
    #[must_use]
    pub fn with_render_image(mut self, enabled: bool) -> Self {
        self.render_image = enabled;
        self
    }

    /// 设置是否渲染路径。
    #[must_use]
    pub fn with_render_path(mut self, enabled: bool) -> Self {
        self.render_path = enabled;
        self
    }

    /// 设置背景颜色。
    #[must_use]
    pub fn with_background_color(mut self, color: u32) -> Self {
        self.background_color = color;
        self
    }

    /// 获取 DPI 对应的像素/毫米比率。
    #[must_use]
    pub fn pixel_per_mm(&self) -> f64 {
        f64::from(self.dpi) * (1.0 / 25.4)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_new() {
        let config = Config::new();
        assert!((config.dpi - 72.0).abs() < f32::EPSILON);
        assert!((config.scale - 1.0).abs() < f32::EPSILON);
        assert!(config.anti_aliasing);
        assert!(config.render_text);
        assert!(config.render_image);
        assert!(config.render_path);
        assert_eq!(config.background_color, 0xFFFF_FFFF);
    }

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert!((config.dpi - 72.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_config_builder() {
        let config = Config::new()
            .with_dpi(300.0)
            .with_scale(2.0)
            .with_anti_aliasing(false)
            .with_render_text(false)
            .with_render_image(false)
            .with_render_path(false)
            .with_background_color(0xFF00_0000);
        assert!((config.dpi - 300.0).abs() < f32::EPSILON);
        assert!((config.scale - 2.0).abs() < f32::EPSILON);
        assert!(!config.anti_aliasing);
        assert!(!config.render_text);
        assert_eq!(config.background_color, 0xFF00_0000);
    }

    #[test]
    fn test_pixel_per_mm() {
        let config = Config::new().with_dpi(254.0);
        assert!((config.pixel_per_mm() - 10.0).abs() < 0.01);
    }

    #[test]
    fn test_config_clone() {
        let c1 = Config::new().with_dpi(150.0);
        let c2 = c1.clone();
        assert!((c1.dpi - c2.dpi).abs() < f32::EPSILON);
    }
}
