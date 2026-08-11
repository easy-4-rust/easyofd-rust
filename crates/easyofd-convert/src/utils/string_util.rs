//! 字符串工具函数。
//!
//! 对应 Java: org.ofdrw.converter.utils.StringUtils

/// 转义 XML 特殊字符。
///
/// 将 `&`、`<`、`>`、`"`、`'` 转义为对应的 XML 实体。
pub fn escape_xml(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    for c in input.chars() {
        match c {
            '&' => output.push_str("&amp;"),
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            '"' => output.push_str("&quot;"),
            '\'' => output.push_str("&apos;"),
            _ => output.push(c),
        }
    }
    output
}

/// 反转义 XML 实体。
///
/// 对应 Java 中常见的 `replaceAll("&lt;","<")` 等操作。
/// 将 `&lt;`、`&gt;`、`&amp;`、`&nbsp;`、`&quot;`、`&apos;`、`&copy;` 还原。
pub fn unescape_xml(input: &str) -> String {
    input
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
        .replace("&nbsp;", " ")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&copy;", "")
}

/// 判断字符串是否为空或仅包含空白字符。
pub fn is_blank(s: &str) -> bool {
    s.trim().is_empty()
}

/// 截断字符串到指定最大长度，超出部分用 `...` 替代。
pub fn truncate_with_ellipsis(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else if max_len <= 3 {
        "...".to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_xml_basic() {
        assert_eq!(escape_xml("hello"), "hello");
        assert_eq!(escape_xml("a&b"), "a&amp;b");
        assert_eq!(escape_xml("<tag>"), "&lt;tag&gt;");
        assert_eq!(escape_xml("\"'"), "&quot;&apos;");
    }

    #[test]
    fn test_escape_xml_empty() {
        assert_eq!(escape_xml(""), "");
    }

    #[test]
    fn test_escape_xml_mixed() {
        assert_eq!(escape_xml("a<b&c>d"), "a&lt;b&amp;c&gt;d");
    }

    #[test]
    fn test_unescape_xml_basic() {
        assert_eq!(unescape_xml("hello"), "hello");
        assert_eq!(unescape_xml("&lt;tag&gt;"), "<tag>");
        assert_eq!(unescape_xml("a&amp;b"), "a&b");
        assert_eq!(unescape_xml("&nbsp;"), " ");
        assert_eq!(unescape_xml("&quot;"), "\"");
        assert_eq!(unescape_xml("&apos;"), "'");
        assert_eq!(unescape_xml("&copy;"), "");
    }

    #[test]
    fn test_unescape_xml_empty() {
        assert_eq!(unescape_xml(""), "");
    }

    #[test]
    fn test_escape_unescape_roundtrip() {
        let original = "Hello <World> & \"Rust\"";
        let escaped = escape_xml(original);
        let unescaped = unescape_xml(&escaped);
        assert_eq!(unescaped, original);
    }

    #[test]
    fn test_is_blank() {
        assert!(is_blank(""));
        assert!(is_blank("   "));
        assert!(is_blank("\t\n"));
        assert!(!is_blank("a"));
        assert!(!is_blank(" a "));
    }

    #[test]
    fn test_truncate_with_ellipsis_short() {
        assert_eq!(truncate_with_ellipsis("abc", 10), "abc");
    }

    #[test]
    fn test_truncate_with_ellipsis_exact() {
        assert_eq!(truncate_with_ellipsis("abcde", 5), "abcde");
    }

    #[test]
    fn test_truncate_with_ellipsis_long() {
        assert_eq!(truncate_with_ellipsis("abcdefgh", 5), "ab...");
    }

    #[test]
    fn test_truncate_with_ellipsis_very_short() {
        assert_eq!(truncate_with_ellipsis("abc", 2), "...");
    }
}
