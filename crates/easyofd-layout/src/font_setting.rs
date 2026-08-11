//! 字体设置。
//!
//! 对应 Java: org.ofdrw.layout.element.canvas.FontSetting

use crate::paragraph::TextAlign;

/// 允许的字符方向/阅读方向值。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// 水平（从左到右）。
    Deg0,
    /// 垂直（从上到下）。
    Deg90,
    /// 水平翻转。
    Deg180,
    /// 垂直翻转。
    Deg270,
}

impl Direction {
    /// 从角度值创建方向（仅允许 0/90/180/270）。
    #[must_use]
    pub fn from_degrees(deg: i32) -> Option<Self> {
        match deg {
            0 => Some(Self::Deg0),
            90 => Some(Self::Deg90),
            180 => Some(Self::Deg180),
            270 => Some(Self::Deg270),
            _ => None,
        }
    }

    /// 转换为角度值。
    #[must_use]
    pub fn to_degrees(self) -> i32 {
        match self {
            Self::Deg0 => 0,
            Self::Deg90 => 90,
            Self::Deg180 => 180,
            Self::Deg270 => 270,
        }
    }
}

/// 字体设置，包含字号、粗细、斜体、字间距、方向等。
///
/// 对应 Java: ofdrw layout canvas FontSetting。
#[derive(Debug, Clone, PartialEq)]
pub struct FontSetting {
    /// 字体名称（如 `"宋体"`、`"SimSun"`）。
    pub font_name: String,
    /// 字号（mm），默认 1.0。
    pub font_size: f64,
    /// 是否斜体。
    pub italic: bool,
    /// 字体粗细，可选值：100-900（步长 100），默认 400。
    pub font_weight: u32,
    /// 字间距（mm），默认 0。
    pub letter_spacing: f64,
    /// 字符方向（基线方向），允许值：0/90/180/270。
    pub char_direction: Direction,
    /// 阅读方向（文字排列方向），允许值：0/90/180/270。
    pub read_direction: Direction,
    /// 文本对齐方式。
    pub text_align: TextAlign,
}

impl Default for FontSetting {
    fn default() -> Self {
        Self {
            font_name: "宋体".to_owned(),
            font_size: 1.0,
            italic: false,
            font_weight: 400,
            letter_spacing: 0.0,
            char_direction: Direction::Deg0,
            read_direction: Direction::Deg0,
            text_align: TextAlign::Left,
        }
    }
}

impl FontSetting {
    /// 创建字体设置（对应 Java: FontSetting(fontSize, fontObj)）。
    #[must_use]
    pub fn new(font_size: f64, font_name: impl Into<String>) -> Self {
        Self {
            font_name: font_name.into(),
            font_size,
            ..Self::default()
        }
    }

    /// 默认宋体实例（对应 Java: FontSetting.getInstance()）。
    #[must_use]
    pub fn default_simsun() -> Self {
        Self::new(5.0, "宋体")
    }

    /// 指定字号的宋体实例（对应 Java: FontSetting.getInstance(double)）。
    #[must_use]
    pub fn simsun_with_size(font_size: f64) -> Self {
        Self::new(font_size, "宋体")
    }

    /// 设置字号（对应 Java: FontSetting#setFontSize）。
    #[must_use]
    pub fn font_size(mut self, size: f64) -> Self {
        self.font_size = size;
        self
    }

    /// 设置字体名称（对应 Java: FontSetting#setFont）。
    #[must_use]
    pub fn font_name(mut self, name: impl Into<String>) -> Self {
        self.font_name = name.into();
        self
    }

    /// 设置斜体（对应 Java: FontSetting#setItalic）。
    #[must_use]
    pub fn italic(mut self, italic: bool) -> Self {
        self.italic = italic;
        self
    }

    /// 设置加粗（对应 Java: FontSetting#setBold，粗细 800）。
    #[must_use]
    pub fn bold(mut self) -> Self {
        self.font_weight = 800;
        self
    }

    /// 设置字体粗细（对应 Java: FontSetting#setFontWeight）。
    ///
    /// 允许值：100、200、300、400、500、600、700、800、900。
    /// 返回 `false` 表示值不合法（不修改）。
    pub fn set_font_weight(&mut self, weight: u32) -> bool {
        if (100..=900).contains(&weight) && weight.is_multiple_of(100) {
            self.font_weight = weight;
            true
        } else {
            false
        }
    }

    /// 设置字间距（对应 Java: FontSetting#setLetterSpacing）。
    ///
    /// 若小于 0 则自动修正为 0。
    #[must_use]
    pub fn letter_spacing(mut self, spacing: f64) -> Self {
        self.letter_spacing = spacing.max(0.0);
        self
    }

    /// 设置字符方向（对应 Java: FontSetting#setCharDirection）。
    #[must_use]
    pub fn char_direction(mut self, dir: Direction) -> Self {
        self.char_direction = dir;
        self
    }

    /// 设置阅读方向（对应 Java: FontSetting#setReadDirection）。
    #[must_use]
    pub fn read_direction(mut self, dir: Direction) -> Self {
        self.read_direction = dir;
        self
    }

    /// 设置文本对齐方式（对应 Java: FontSetting#setTextAlign）。
    #[must_use]
    pub fn text_align(mut self, align: TextAlign) -> Self {
        self.text_align = align;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let fs = FontSetting::default();
        assert_eq!(fs.font_name, "宋体");
        assert!((fs.font_size - 1.0).abs() < f64::EPSILON);
        assert!(!fs.italic);
        assert_eq!(fs.font_weight, 400);
        assert!((fs.letter_spacing - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_new() {
        let fs = FontSetting::new(12.0, "SimSun");
        assert!((fs.font_size - 12.0).abs() < f64::EPSILON);
        assert_eq!(fs.font_name, "SimSun");
    }

    #[test]
    fn test_default_simsun() {
        let fs = FontSetting::default_simsun();
        assert!((fs.font_size - 5.0).abs() < f64::EPSILON);
        assert_eq!(fs.font_name, "宋体");
    }

    #[test]
    fn test_builders() {
        let fs = FontSetting::new(10.0, "SimHei")
            .italic(true)
            .bold()
            .letter_spacing(1.5)
            .char_direction(Direction::Deg90)
            .read_direction(Direction::Deg0)
            .text_align(TextAlign::Center);
        assert!(fs.italic);
        assert_eq!(fs.font_weight, 800);
        assert!((fs.letter_spacing - 1.5).abs() < f64::EPSILON);
        assert_eq!(fs.char_direction, Direction::Deg90);
        assert_eq!(fs.text_align, TextAlign::Center);
    }

    #[test]
    fn test_set_font_weight_valid() {
        let mut fs = FontSetting::default();
        assert!(fs.set_font_weight(700));
        assert_eq!(fs.font_weight, 700);
    }

    #[test]
    fn test_set_font_weight_invalid() {
        let mut fs = FontSetting::default();
        assert!(!fs.set_font_weight(750));
        assert_eq!(fs.font_weight, 400); // unchanged
        assert!(!fs.set_font_weight(0));
        assert!(!fs.set_font_weight(1000));
    }

    #[test]
    fn test_letter_spacing_clamp() {
        let fs = FontSetting::default().letter_spacing(-1.0);
        assert!((fs.letter_spacing - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_direction_from_degrees() {
        assert_eq!(Direction::from_degrees(0), Some(Direction::Deg0));
        assert_eq!(Direction::from_degrees(90), Some(Direction::Deg90));
        assert_eq!(Direction::from_degrees(180), Some(Direction::Deg180));
        assert_eq!(Direction::from_degrees(270), Some(Direction::Deg270));
        assert_eq!(Direction::from_degrees(45), None);
    }

    #[test]
    fn test_direction_to_degrees() {
        assert_eq!(Direction::Deg0.to_degrees(), 0);
        assert_eq!(Direction::Deg90.to_degrees(), 90);
        assert_eq!(Direction::Deg180.to_degrees(), 180);
        assert_eq!(Direction::Deg270.to_degrees(), 270);
    }

    #[test]
    fn test_clone_eq() {
        let a = FontSetting::new(10.0, "SimSun").bold();
        let b = a.clone();
        assert_eq!(a, b);
    }
}
