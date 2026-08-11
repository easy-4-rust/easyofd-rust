use crate::font_descriptor::FontDescriptor;

/// 估算文本宽度（单位：mm）。
///
/// 计算方式：`字符数 × 平均字符宽度(pt) × 字号(pt) / 72 × 25.4(mm/in)`
///
/// 其中 1 pt = 1/72 inch，1 inch = 25.4 mm。
///
/// # 参数
/// - `text`：待测量的文本
/// - `font_size`：字号（单位：pt）
/// - `font`：字体描述符，提供平均字符宽度
#[must_use]
pub fn estimate_text_width(text: &str, font_size: f64, font: &FontDescriptor) -> f64 {
    let char_count = text.chars().count();
    #[allow(clippy::cast_precision_loss)]
    let count_f = char_count as f64;
    count_f * font.char_width * font_size / 72.0 * 25.4
}

/// 估算文本高度（单位：mm）。
///
/// 通常等于字号转换为毫米：`font_size(pt) / 72 × 25.4`。
///
/// # 参数
/// - `font_size`：字号（单位：pt）
#[must_use]
pub fn estimate_text_height(font_size: f64) -> f64 {
    font_size / 72.0 * 25.4
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_font(char_width: f64) -> FontDescriptor {
        FontDescriptor::new("TestFont", "测试字体", char_width)
    }

    #[test]
    fn test_empty_text_width() {
        let font = make_font(10.0);
        let width = estimate_text_width("", 12.0, &font);
        assert!((width - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_single_char_width() {
        // 字符宽度 10pt，字号 72pt → 宽度 = 10 * 72 / 72 * 25.4 = 254.0 mm
        let font = make_font(10.0);
        let width = estimate_text_width("A", 72.0, &font);
        assert!((width - 254.0).abs() < 0.01);
    }

    #[test]
    fn test_multi_char_width() {
        let font = make_font(8.0);
        let width = estimate_text_width("Hello", 12.0, &font);
        let expected = 5.0 * 8.0 * 12.0 / 72.0 * 25.4;
        assert!((width - expected).abs() < 0.01);
    }

    #[test]
    fn test_chinese_text_width() {
        let font = make_font(12.0);
        let width = estimate_text_width("你好", 14.0, &font);
        let expected = 2.0 * 12.0 * 14.0 / 72.0 * 25.4;
        assert!((width - expected).abs() < 0.01);
    }

    #[test]
    fn test_text_height() {
        // 72pt → 25.4mm
        let height = estimate_text_height(72.0);
        assert!((height - 25.4).abs() < 0.01);
    }
}
