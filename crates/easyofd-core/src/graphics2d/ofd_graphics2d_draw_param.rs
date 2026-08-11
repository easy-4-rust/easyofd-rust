//! OFD 2D 图形绘制参数。
//!
//! 对应 Java: org.ofdrw.graphics2d.OFDGraphics2DDrawParam
//!
//! Java 版中封装 `java.awt.Graphics2D` 的绘制状态（颜色、字体、变换等）。
//! Rust 版提供简化结构，保留核心绘制参数。

/// OFD 2D 图形绘制参数。
///
/// 对应 Java: org.ofdrw.graphics2d.OFDGraphics2DDrawParam
///
/// 描述绘制上下文的当前状态：描边颜色、填充颜色、线宽、
/// 透明度等。Java 版依赖 AWT Color/Font；Rust 版用基础类型。
#[derive(Debug, Clone)]
pub struct OfdGraphics2DDrawParam {
    /// 描边颜色（RGB，如 0xFF0000 = 红色）。
    pub stroke_color: Option<u32>,
    /// 填充颜色（RGB）。
    pub fill_color: Option<u32>,
    /// 线宽（mm）。
    pub line_width: f64,
    /// 全局透明度（0.0 = 全透明，1.0 = 不透明）。
    pub opacity: f64,
    /// 字体大小（pt）。
    pub font_size: f64,
    /// 字体名称。
    pub font_name: Option<String>,
}

impl OfdGraphics2DDrawParam {
    /// 创建默认绘制参数。
    #[must_use]
    pub fn new() -> Self {
        Self {
            stroke_color: None,
            fill_color: None,
            line_width: 1.0,
            opacity: 1.0,
            font_size: 12.0,
            font_name: None,
        }
    }

    /// 设置描边颜色。
    #[must_use]
    pub fn stroke_color(mut self, color: u32) -> Self {
        self.stroke_color = Some(color);
        self
    }

    /// 设置填充颜色。
    #[must_use]
    pub fn fill_color(mut self, color: u32) -> Self {
        self.fill_color = Some(color);
        self
    }

    /// 设置线宽。
    #[must_use]
    pub fn line_width(mut self, width: f64) -> Self {
        self.line_width = width;
        self
    }

    /// 设置透明度。
    #[must_use]
    pub fn opacity(mut self, opacity: f64) -> Self {
        self.opacity = opacity.clamp(0.0, 1.0);
        self
    }

    /// 设置字体大小。
    #[must_use]
    pub fn font_size(mut self, size: f64) -> Self {
        self.font_size = size;
        self
    }

    /// 设置字体名称。
    #[must_use]
    pub fn font_name(mut self, name: impl Into<String>) -> Self {
        self.font_name = Some(name.into());
        self
    }
}

impl Default for OfdGraphics2DDrawParam {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        let p = OfdGraphics2DDrawParam::new();
        assert!(p.stroke_color.is_none());
        assert!(p.fill_color.is_none());
        assert!((p.line_width - 1.0).abs() < f64::EPSILON);
        assert!((p.opacity - 1.0).abs() < f64::EPSILON);
        assert!((p.font_size - 12.0).abs() < f64::EPSILON);
        assert!(p.font_name.is_none());
    }

    #[test]
    fn test_builder() {
        let p = OfdGraphics2DDrawParam::new()
            .stroke_color(0xFF_0000)
            .fill_color(0x00_FF00)
            .line_width(2.5)
            .opacity(0.8)
            .font_size(14.0)
            .font_name("SimSun");
        assert_eq!(p.stroke_color, Some(0xFF_0000));
        assert_eq!(p.fill_color, Some(0x00_FF00));
        assert!((p.line_width - 2.5).abs() < f64::EPSILON);
        assert!((p.opacity - 0.8).abs() < f64::EPSILON);
        assert_eq!(p.font_name.as_deref(), Some("SimSun"));
    }

    #[test]
    fn test_opacity_clamp() {
        let p = OfdGraphics2DDrawParam::new().opacity(1.5);
        assert!((p.opacity - 1.0).abs() < f64::EPSILON);
        let p2 = OfdGraphics2DDrawParam::new().opacity(-0.5);
        assert!((p2.opacity - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_default() {
        let p = OfdGraphics2DDrawParam::default();
        assert!((p.line_width - 1.0).abs() < f64::EPSILON);
    }
}
