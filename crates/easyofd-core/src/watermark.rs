//! 水印和注解支持。

/// 水印配置。支持文本水印和图片水印。
#[derive(Debug, Clone)]
pub struct Watermark {
    /// 水印文本（文本水印）。
    pub text: Option<String>,
    /// 水印图片数据（图片水印）。
    pub image: Option<Vec<u8>>,
    /// 位置 (x, y) mm。
    pub position: (f64, f64),
    /// 文本水印字号。
    pub font_size: f64,
    /// 文本水印字体。
    pub font: String,
    /// 文本颜色 RGB hex。
    pub color: u32,
    /// 透明度 0.0（透明）– 1.0（不透明）。默认 0.3。
    pub opacity: f64,
    /// 旋转角度（度）。默认 45.0（对角线）。
    pub rotation: f64,
    /// 目标页码（1-based）。None = 所有页。
    pub page: Option<usize>,
}

impl Default for Watermark {
    fn default() -> Self {
        Self {
            text: None,
            image: None,
            position: (0.0, 0.0),
            font_size: 24.0,
            font: "SimSun".into(),
            color: 0xCC_CC_CC,
            opacity: 0.3,
            rotation: 45.0,
            page: None,
        }
    }
}

impl Watermark {
    /// 创建文本水印。
    #[must_use]
    pub fn text(content: impl Into<String>) -> Self {
        Self {
            text: Some(content.into()),
            ..Self::default()
        }
    }

    /// 设置位置。
    #[must_use]
    pub fn position(mut self, x: f64, y: f64) -> Self {
        self.position = (x, y);
        self
    }

    /// 设置字号。
    #[must_use]
    pub fn font_size(mut self, size: f64) -> Self {
        self.font_size = size;
        self
    }

    /// 设置透明度。
    #[must_use]
    pub fn opacity(mut self, opacity: f64) -> Self {
        self.opacity = opacity;
        self
    }

    /// 设置旋转角度。
    #[must_use]
    pub fn rotation(mut self, degrees: f64) -> Self {
        self.rotation = degrees;
        self
    }

    /// 设置目标页码。
    #[must_use]
    pub fn page(mut self, page: usize) -> Self {
        self.page = Some(page);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_watermark_default() {
        let wm = Watermark::default();
        assert_eq!(wm.font_size, 24.0);
        assert_eq!(wm.opacity, 0.3);
        assert_eq!(wm.rotation, 45.0);
        assert!(wm.text.is_none());
        assert!(wm.page.is_none());
    }

    #[test]
    fn test_watermark_text_builder() {
        let wm = Watermark::text("CONFIDENTIAL")
            .position(50.0, 100.0)
            .font_size(36.0)
            .opacity(0.5)
            .rotation(30.0)
            .page(1);
        assert_eq!(wm.text.as_deref(), Some("CONFIDENTIAL"));
        assert_eq!(wm.position, (50.0, 100.0));
        assert_eq!(wm.font_size, 36.0);
        assert_eq!(wm.opacity, 0.5);
        assert_eq!(wm.rotation, 30.0);
        assert_eq!(wm.page, Some(1));
    }

    #[test]
    fn test_watermark_clone_debug() {
        let wm = Watermark::text("test");
        let wm2 = wm.clone();
        assert_eq!(wm2.text, wm.text);
        assert!(format!("{wm:?}").contains("Watermark"));
    }
}
