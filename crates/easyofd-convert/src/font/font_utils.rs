//! 字体工具函数。
//!
//! 对应 Java: org.ofdrw.converter.utils.FontUtils
//!
//! Java 版提供字体名称规范化、字体族判断等工具函数。
//! Rust 版提供等价的纯函数。

/// 规范化字体名称。
///
/// 对应 Java: `FontUtils.normalizeFontName(String name)`
///
/// 移除多余空格，统一大小写，去除引号。
#[must_use]
pub fn normalize_font_name(name: &str) -> String {
    name.trim()
        .trim_matches('"')
        .trim_matches('\'')
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// 判断字体名称是否为 CJK（中日韩）字体。
///
/// 对应 Java: `FontUtils.isCJKFont(String fontName)`
///
/// 通过字体名称中的关键字判断。
#[must_use]
pub fn is_cjk_font(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.contains("song")
        || lower.contains("hei")
        || lower.contains("kai")
        || lower.contains("fang")
        || lower.contains("ming")
        || lower.contains("gothic")
        || lower.contains("mincho")
        || lower.contains("simsun")
        || lower.contains("simhei")
        || lower.contains("nsimsun")
        || lower.contains("kaiti")
        || lower.contains("stzhongs")
        || lower.contains("stfangsong")
        || lower.contains("fzshu")
        || lower.contains("fzkai")
        || lower.contains("fzhei")
        || lower.contains("fzsong")
        || lower.contains("microsoft yahei")
        || lower.contains("dengxian")
        || lower.contains("wenquanyi")
        || lower.contains("noto sans cjk")
        || lower.contains("noto serif cjk")
        || lower.contains("source han")
        || lower.contains("adobe heiti")
        || lower.contains("adobe song")
        || lower.contains("adobe kaiti")
        || lower.contains("adobe fangsong")
}

/// 判断字体名称是否为等宽字体。
#[must_use]
pub fn is_monospace_font(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.contains("mono")
        || lower.contains("courier")
        || lower.contains("consolas")
        || lower.contains("menlo")
        || lower.contains("dejavu sans mono")
        || lower.contains("source code")
        || lower.contains("fira code")
}

/// 获取标准字体族名称。
///
/// 对应 Java: `FontUtils.getFontFamily(String fontName)`
///
/// 返回 CSS font-family 等价字符串。
#[must_use]
pub fn font_family(name: &str) -> &'static str {
    let lower = name.to_lowercase();
    if lower.contains("serif") && !lower.contains("sans") {
        "serif"
    } else if lower.contains("sans") || lower.contains("arial") || lower.contains("helvetica") {
        "sans-serif"
    } else if is_monospace_font(name) {
        "monospace"
    } else if lower.contains("cursive") || lower.contains("script") {
        "cursive"
    } else if lower.contains("fantasy") {
        "fantasy"
    } else {
        "sans-serif"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_font_name() {
        assert_eq!(normalize_font_name("  SimSun  "), "SimSun");
        assert_eq!(normalize_font_name("\"Arial\""), "Arial");
        assert_eq!(normalize_font_name("'Helvetica'"), "Helvetica");
        assert_eq!(normalize_font_name("  DejaVu   Sans  "), "DejaVu Sans");
    }

    #[test]
    fn test_is_cjk_font() {
        assert!(is_cjk_font("SimSun"));
        assert!(is_cjk_font("SimHei"));
        assert!(is_cjk_font("NSimSun"));
        assert!(is_cjk_font("KaiTi"));
        assert!(is_cjk_font("FangSong"));
        assert!(is_cjk_font("Microsoft YaHei"));
        assert!(is_cjk_font("Noto Sans CJK SC"));
        assert!(is_cjk_font("Source Han Sans CN"));
        assert!(is_cjk_font("WenQuanYi Micro Hei"));
        assert!(!is_cjk_font("Arial"));
        assert!(!is_cjk_font("Helvetica"));
        assert!(!is_cjk_font("Times New Roman"));
    }

    #[test]
    fn test_is_monospace_font() {
        assert!(is_monospace_font("Courier New"));
        assert!(is_monospace_font("Consolas"));
        assert!(is_monospace_font("DejaVu Sans Mono"));
        assert!(is_monospace_font("Source Code Pro"));
        assert!(!is_monospace_font("Arial"));
        assert!(!is_monospace_font("SimSun"));
    }

    #[test]
    fn test_font_family() {
        assert_eq!(font_family("DejaVu Serif"), "serif");
        assert_eq!(font_family("Noto Serif CJK"), "serif");
        assert_eq!(font_family("Arial"), "sans-serif");
        assert_eq!(font_family("Helvetica"), "sans-serif");
        assert_eq!(font_family("Courier New"), "monospace");
        assert_eq!(font_family("Consolas"), "monospace");
        assert_eq!(font_family("Unknown Font"), "sans-serif");
    }

    #[test]
    fn test_normalize_empty() {
        assert_eq!(normalize_font_name(""), "");
        assert_eq!(normalize_font_name("   "), "");
    }
}
