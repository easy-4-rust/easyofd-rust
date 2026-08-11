//! 画布上下文中的绘制参数状态。
//!
//! 对应 Java: org.ofdrw.layout.element.canvas.CanvasState

use crate::font_setting::FontSetting;

/// 变换矩阵 `[a, b, c, d, e, f]`。
pub type TransformMatrix = [f64; 6];

/// 画布上下文中的绘制参数状态，支持 save/restore。
///
/// 对应 Java: ofdrw layout canvas CanvasState。
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CanvasState {
    /// 路径数据（缩写路径命令字符串）。
    pub path: String,
    /// 变换矩阵 `[a, b, c, d, e, f]`（可选）。
    pub ctm: Option<TransformMatrix>,
    /// 绘制文字设置。
    pub font: FontSetting,
    /// 透明值，范围 `[0.0, 1.0]`（可选）。
    pub global_alpha: Option<f64>,
    /// 裁剪区域路径数据（可选）。
    pub clip_area: Option<String>,
    /// 填充颜色（16 进制格式如 `#000000`，或颜色名，或渐变/模式标识）。
    pub fill_style: Option<String>,
    /// 描边颜色（16 进制格式如 `#000000`，或颜色名，或渐变/模式标识）。
    pub stroke_style: Option<String>,
    /// 字体样式字符串（如 `"bold 3mm SimSun"`）。
    pub font_style: Option<String>,
}

impl CanvasState {
    /// 创建默认画布状态。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置变换矩阵。
    #[must_use]
    pub fn ctm(mut self, ctm: TransformMatrix) -> Self {
        self.ctm = Some(ctm);
        self
    }

    /// 设置透明值。
    #[must_use]
    pub fn global_alpha(mut self, alpha: f64) -> Self {
        self.global_alpha = Some(alpha.clamp(0.0, 1.0));
        self
    }

    /// 设置填充颜色。
    #[must_use]
    pub fn fill_style(mut self, style: impl Into<String>) -> Self {
        self.fill_style = Some(style.into());
        self
    }

    /// 设置描边颜色。
    #[must_use]
    pub fn stroke_style(mut self, style: impl Into<String>) -> Self {
        self.stroke_style = Some(style.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let state = CanvasState::new();
        assert!(state.path.is_empty());
        assert!(state.ctm.is_none());
        assert!(state.global_alpha.is_none());
        assert!(state.fill_style.is_none());
        assert!(state.stroke_style.is_none());
    }

    #[test]
    fn test_builders() {
        let state = CanvasState::new()
            .ctm([1.0, 0.0, 0.0, 1.0, 10.0, 20.0])
            .global_alpha(0.5)
            .fill_style("#FF0000")
            .stroke_style("#0000FF");
        assert_eq!(state.ctm, Some([1.0, 0.0, 0.0, 1.0, 10.0, 20.0]));
        assert!((state.global_alpha.unwrap() - 0.5).abs() < f64::EPSILON);
        assert_eq!(state.fill_style.as_deref(), Some("#FF0000"));
        assert_eq!(state.stroke_style.as_deref(), Some("#0000FF"));
    }

    #[test]
    fn test_global_alpha_clamp() {
        let state = CanvasState::new().global_alpha(2.0);
        assert!((state.global_alpha.unwrap() - 1.0).abs() < f64::EPSILON);

        let state = CanvasState::new().global_alpha(-0.5);
        assert!((state.global_alpha.unwrap() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_clone_eq() {
        let a = CanvasState::new().fill_style("#ABC");
        let b = a.clone();
        assert_eq!(a, b);
    }
}
