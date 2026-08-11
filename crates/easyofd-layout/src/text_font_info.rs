//! 文本字体信息接口。
//!
//! 对应 Java: org.ofdrw.layout.element.TextFontInfo

/// 文本字体信息接口，描述字体的基本属性。
///
/// 对应 Java: ofdrw layout TextFontInfo（interface）。
pub trait TextFontInfo {
    /// 获取字号（mm）。
    fn font_size(&self) -> f64;

    /// 获取字间距（mm）。
    fn letter_spacing(&self) -> f64;

    /// 获取字体名称。
    fn font_name(&self) -> &str;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockFontInfo {
        size: f64,
        spacing: f64,
        name: String,
    }

    impl TextFontInfo for MockFontInfo {
        fn font_size(&self) -> f64 {
            self.size
        }
        fn letter_spacing(&self) -> f64 {
            self.spacing
        }
        fn font_name(&self) -> &str {
            &self.name
        }
    }

    #[test]
    fn test_text_font_info() {
        let info = MockFontInfo {
            size: 12.0,
            spacing: 1.5,
            name: "SimSun".to_owned(),
        };
        assert!((info.font_size() - 12.0).abs() < f64::EPSILON);
        assert!((info.letter_spacing() - 1.5).abs() < f64::EPSILON);
        assert_eq!(info.font_name(), "SimSun");
    }
}
