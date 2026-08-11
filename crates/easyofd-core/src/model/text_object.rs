//! 文本对象。

/// 带位置、字体和内容的文本对象。
#[derive(Debug, Clone)]
pub struct TextObject {
    /// 距左边缘的 X 位置（mm）。
    pub x: f64,
    /// 距顶部的 Y 位置（mm）。
    pub y: f64,
    /// 字体族名称（如 "SimSun"、"SimHei"）。
    pub font: String,
    /// 字号（pt）。
    pub size: f64,
    /// 字重: 400 = 正常, 700 = 粗体。
    pub weight: u32,
    /// 是否斜体。
    pub italic: bool,
    /// 文本颜色（RGB 十六进制，如 0x000000 为黑色）。
    pub color: u32,
    /// 实际文本内容。
    pub text: String,
    /// 可选的文本宽度覆盖（mm）。
    /// 如果为 None，写入器将根据字符数估算。
    pub width: Option<f64>,
    /// 可选的文本高度覆盖（mm）。
    /// 如果为 None，写入器将使用字号。
    pub height: Option<f64>,
}

impl TextObject {
    /// 使用默认样式创建新的文本对象。
    #[must_use]
    pub fn new(x: f64, y: f64, text: impl Into<String>) -> Self {
        Self {
            x,
            y,
            font: "SimSun".to_string(),
            size: 12.0,
            weight: 400,
            italic: false,
            color: 0x000_000,
            text: text.into(),
            width: None,
            height: None,
        }
    }

    /// 设置字体族。
    #[must_use]
    pub fn font(mut self, font: impl Into<String>) -> Self {
        self.font = font.into();
        self
    }

    /// 设置字号（pt）。
    #[must_use]
    pub fn size(mut self, size: f64) -> Self {
        self.size = size;
        self
    }

    /// 设置粗体。
    #[must_use]
    pub fn bold(mut self) -> Self {
        self.weight = 700;
        self
    }

    /// 设置斜体。
    #[must_use]
    pub fn italic(mut self) -> Self {
        self.italic = true;
        self
    }

    /// 设置文本颜色（RGB 十六进制）。
    #[must_use]
    pub fn color(mut self, color: u32) -> Self {
        self.color = color;
        self
    }
}
