//! 字体测量工具。
//!
//! 对应 Java: org.ofdrw.layout.element.canvas.TextMeasureTool

use crate::font_setting::FontSetting;
use crate::measure_body::MeasureBody;

/// 字体测量工具。
///
/// 对应 Java: ofdrw layout canvas TextMeasureTool。
pub struct TextMeasureTool;

impl TextMeasureTool {
    /// 分析字间距偏移量并计算文字宽度。
    ///
    /// 对应 Java: TextMeasureTool#measureWithWith。
    ///
    /// 这是一个简化实现，仅处理水平阅读方向（read_direction=0, char_direction=0）的场景。
    /// 完整的方向处理需要字体度量数据支持。
    #[must_use]
    pub fn measure_with_width(text: &str, font_setting: &FontSetting) -> MeasureBody {
        let mut body = MeasureBody::new();

        if text.is_empty() {
            return body;
        }

        let chars: Vec<char> = text.chars().collect();
        let font_size = font_setting.font_size;
        let letter_spacing = font_setting.letter_spacing;

        if chars.len() > 1 {
            // 简化：每个字符宽度用 font_size 估算 + letter_spacing
            let mut offsets = Vec::with_capacity(chars.len() - 1);
            for _ in 0..chars.len() - 1 {
                offsets.push(font_size * 0.6 + letter_spacing);
            }
            body.offset = offsets;
        }

        // 最后一个字符的估算宽度
        let last_char_width = font_size * 0.6;
        body.with_char_len(last_char_width);

        body
    }

    /// 测量文本各个字符的偏移量数组（对应 Java: TextMeasureTool#measure）。
    #[must_use]
    pub fn measure(text: &str, font_setting: &FontSetting) -> Vec<f64> {
        Self::measure_with_width(text, font_setting).offset
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_text() {
        let fs = FontSetting::default();
        let body = TextMeasureTool::measure_with_width("", &fs);
        assert!(body.offset.is_empty());
        assert!((body.width - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_single_char() {
        let fs = FontSetting::new(5.0, "SimSun");
        let body = TextMeasureTool::measure_with_width("A", &fs);
        assert!(body.offset.is_empty());
        // width = 0.6 * 5.0 = 3.0
        assert!((body.width - 3.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_multiple_chars() {
        let fs = FontSetting::new(5.0, "SimSun");
        let body = TextMeasureTool::measure_with_width("ABC", &fs);
        assert_eq!(body.offset.len(), 2); // n-1 offsets
        assert!(body.width > 0.0);
    }

    #[test]
    fn test_measure_returns_offsets() {
        let fs = FontSetting::new(5.0, "SimSun");
        let offsets = TextMeasureTool::measure("Hello", &fs);
        assert_eq!(offsets.len(), 4); // n-1
    }
}
