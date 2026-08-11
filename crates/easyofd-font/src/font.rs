//! OFD 逻辑字体。
//!
//! 对应 Java: org.ofdrw.font.Font

use std::path::PathBuf;

use crate::font_name::FontName;

/// 逻辑字体（ofdrw Font）。
///
/// 对应 Java: ofdrw Font。记录字体名、族名、字体文件路径与可选的可打印
/// ASCII 宽度表；`get_char_width_scale` 返回单个字符的宽度比例。
#[derive(Debug, Clone, PartialEq)]
pub struct Font {
    /// 字体名。
    pub name: String,
    /// 字体族名。
    pub family_name: String,
    /// 字体文件路径（可选）。
    pub font_file: Option<PathBuf>,
    /// 可打印 ASCII 宽度表（字符 32..=126，可选）。
    pub printable_ascii_width_map: Option<Vec<f64>>,
}

impl Font {
    /// 创建逻辑字体（对应 Java: Font(name, familyName, fontFile)）。
    #[must_use]
    pub fn new(
        name: impl Into<String>,
        family_name: impl Into<String>,
        font_file: Option<PathBuf>,
    ) -> Self {
        Self {
            name: name.into(),
            family_name: family_name.into(),
            font_file,
            printable_ascii_width_map: None,
        }
    }

    /// 创建仅含名字的字体（对应 Java: Font(name, familyName)）。
    #[must_use]
    pub fn named(name: impl Into<String>, family_name: impl Into<String>) -> Self {
        Self::new(name, family_name, None)
    }

    /// 默认字体（宋体，对应 Java: Font#getDefault）。
    #[must_use]
    pub fn default_font() -> Self {
        Self::named("宋体", "宋体")
    }

    /// 从标准字体名创建（对应 Java: FontName#font）。
    #[must_use]
    pub fn from_font_name(font_name: FontName) -> Self {
        let family = font_name.family_name();
        let mut font = Self::named(family, family);
        font.printable_ascii_width_map = Some(font_name.printable_ascii_width().to_vec());
        font
    }

    /// 设置可打印 ASCII 宽度表（对应 Java: Font#setPrintableAsciiWidthMap）。
    #[must_use]
    pub fn with_ascii_width_map(mut self, map: &[f64]) -> Self {
        self.printable_ascii_width_map = Some(map.to_vec());
        self
    }

    /// 返回单个字符的宽度比例。
    ///
    /// 对应 Java: ofdrw Font#getCharWidthScale。ASCII 字符 [32, 126] 查表，
    /// 空格为半个字符宽；非 ASCII 按全角（1.0）处理；无宽度表时按
    /// ASCII 0.5 / 其他 1.0 估算。
    #[must_use]
    pub fn get_char_width_scale(&self, ch: char) -> f64 {
        let code = ch as u32;
        if let Some(map) = &self.printable_ascii_width_map {
            if (32..=126).contains(&code) {
                return map[(code - 32) as usize];
            }
            // 空格与 CJK 全角
            return if ch == ' ' { 0.5 } else { 1.0 };
        }
        if ch.is_ascii() { 0.5 } else { 1.0 }
    }

    /// 完整字体名（对应 Java: Font#getCompleteFontName）。
    #[must_use]
    pub fn complete_font_name(&self) -> String {
        if self.name.is_empty() {
            self.family_name.clone()
        } else {
            self.name.clone()
        }
    }

    /// 字体文件名（对应 Java: Font#getFontFileName）。
    #[must_use]
    pub fn font_file_name(&self) -> Option<String> {
        self.font_file
            .as_ref()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().into_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_font() {
        let f = Font::default_font();
        assert_eq!(f.family_name, "宋体");
        assert!(f.font_file.is_none());
    }

    #[test]
    fn test_from_font_name() {
        let f = Font::from_font_name(FontName::SimSun);
        assert_eq!(f.family_name, "宋体");
        assert!(f.printable_ascii_width_map.is_some());
    }

    #[test]
    fn test_char_width_scale() {
        // 无宽度表：ASCII 0.5，CJK 1.0
        let f = Font::default_font();
        assert!((f.get_char_width_scale('a') - 0.5).abs() < f64::EPSILON);
        assert!((f.get_char_width_scale('中') - 1.0).abs() < f64::EPSILON);
        // 有宽度表：空格 0.5（NOTO 表首项）
        let g = Font::from_font_name(FontName::SimSun);
        assert!((g.get_char_width_scale(' ') - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_complete_name_and_file() {
        let f = Font::new("MyFont", "MyFamily", Some(PathBuf::from("/tmp/f.ttf")));
        assert_eq!(f.complete_font_name(), "MyFont");
        assert_eq!(f.font_file_name().as_deref(), Some("f.ttf"));
        assert_eq!(Font::named("", "Fam").complete_font_name(), "Fam");
    }
}
