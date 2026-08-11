//! 路径对象。

/// 矢量路径对象（直线、矩形、曲线）。
#[derive(Debug, Clone)]
pub struct PathObject {
    /// 距左边缘的 X 位置（mm）。
    pub x: f64,
    /// 距顶部的 Y 位置（mm）。
    pub y: f64,
    /// 描边颜色（RGB 十六进制）。
    pub stroke_color: u32,
    /// 描边宽度（mm）。
    pub stroke_width: f64,
    /// 填充颜色（RGB 十六进制，可选）。
    pub fill_color: Option<u32>,
    /// SVG 风格的路径数据字符串。
    pub path_data: String,
}

impl PathObject {
    /// 创建新的路径对象。
    #[must_use]
    pub fn new(x: f64, y: f64, path_data: impl Into<String>) -> Self {
        Self {
            x,
            y,
            stroke_color: 0x000_000,
            stroke_width: 0.35,
            fill_color: None,
            path_data: path_data.into(),
        }
    }

    /// 创建水平线。
    #[must_use]
    pub fn hline(x1: f64, y: f64, x2: f64) -> Self {
        Self::new(x1, y, format!("M{x1} {y}L{x2} {y}"))
    }

    /// 创建垂直线。
    #[must_use]
    pub fn vline(x: f64, y1: f64, y2: f64) -> Self {
        Self::new(x, y1, format!("M{x} {y1}L{x} {y2}"))
    }

    /// 创建矩形轮廓。
    #[must_use]
    #[allow(clippy::many_single_char_names)]
    pub fn rect(x: f64, y: f64, w: f64, h: f64) -> Self {
        let d = format!("M{x} {y}L{} {y}L{} {}L{x} {}Z", x + w, x + w, y + h, y + h);
        Self::new(x, y, d)
    }

    /// 设置描边颜色。
    #[must_use]
    pub fn stroke_color(mut self, color: u32) -> Self {
        self.stroke_color = color;
        self
    }

    /// 设置描边宽度。
    #[must_use]
    pub fn stroke_width(mut self, width: f64) -> Self {
        self.stroke_width = width;
        self
    }

    /// 设置填充颜色。
    #[must_use]
    pub fn fill_color(mut self, color: u32) -> Self {
        self.fill_color = Some(color);
        self
    }
}
