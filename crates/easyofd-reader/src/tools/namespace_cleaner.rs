//! 命名空间清理工具。
//!
//! 对应 Java: org.ofdrw.reader.tools.NameSpaceCleaner
//!
//! 从 OFD XML 中移除命名空间前缀，使其可以被不支持命名空间的解析器处理。

/// OFD 命名空间清理器。
///
/// 对应 Java: `org.ofdrw.reader.tools.NameSpaceCleaner`
///
/// 将 XML 中的命名空间前缀（如 `ofd:`）移除，使元素名变为
/// 无前缀的本地名称。这在需要与不支持 XML 命名空间的工具交互时有用。
#[derive(Debug, Clone, Copy)]
pub struct NamespaceCleaner;

impl NamespaceCleaner {
    /// 从 XML 字符串中移除指定的命名空间前缀。
    ///
    /// 将所有 `ofd:ElementName` 替换为 `ElementName`。
    #[must_use]
    pub fn remove_prefix(xml: &str, prefix: &str) -> String {
        let with_colon = format!("{prefix}:");
        xml.replace(&with_colon, "")
    }

    /// 从 XML 字符串中移除 OFD 命名空间前缀。
    #[must_use]
    pub fn remove_ofd_prefix(xml: &str) -> String {
        Self::remove_prefix(xml, "ofd")
    }

    /// 从 XML 字符串中移除命名空间声明属性。
    ///
    /// 移除 `xmlns:ofd="..."` 形式的属性。
    #[must_use]
    pub fn remove_namespace_declarations(xml: &str) -> String {
        let mut result = String::with_capacity(xml.len());
        let mut remaining = xml;

        while let Some(start) = remaining.find("xmlns:") {
            result.push_str(&remaining[..start]);
            // 找到属性值的结束引号
            let after_attr = &remaining[start..];
            if let Some(eq_pos) = after_attr.find('=').map(|p| p + 1) {
                let after_eq = &after_attr[eq_pos..];
                // 跳过引号内的内容
                if let Some(quote_end) = find_closing_quote(after_eq) {
                    remaining = &after_eq[quote_end + 1..];
                } else {
                    // 没有找到关闭引号，保留剩余内容
                    result.push_str(after_attr);
                    break;
                }
            } else {
                result.push_str(after_attr);
                break;
            }
        }
        result.push_str(remaining);
        result
    }
}

/// 查找引号字符串的关闭位置。
fn find_closing_quote(s: &str) -> Option<usize> {
    let s = s.trim_start();
    let quote_char = s.as_bytes().first()?;
    if *quote_char != b'"' && *quote_char != b'\'' {
        return None;
    }
    let content = &s[1..];
    content
        .find(*quote_char as char)
        .map(|pos| pos + 1 + (s.len() - content.len()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_ofd_prefix() {
        let xml = "<ofd:OFD><ofd:DocBody><ofd:DocRoot/></ofd:DocBody></ofd:OFD>";
        let result = NamespaceCleaner::remove_ofd_prefix(xml);
        assert!(!result.contains("ofd:"));
        assert!(result.contains("<OFD>"));
        assert!(result.contains("<DocBody>"));
        assert!(result.contains("<DocRoot/>"));
        assert!(result.contains("</OFD>"));
    }

    #[test]
    fn test_remove_custom_prefix() {
        let xml = "<ns:Root><ns:Child/></ns:Root>";
        let result = NamespaceCleaner::remove_prefix(xml, "ns");
        assert_eq!(result, "<Root><Child/></Root>");
    }

    #[test]
    fn test_remove_namespace_declarations() {
        let xml =
            r#"<ofd:OFD xmlns:ofd="http://www.ofdspec.org/2016" xmlns:custom="http://custom">"#;
        let result = NamespaceCleaner::remove_namespace_declarations(xml);
        assert!(!result.contains("xmlns:ofd"));
        assert!(!result.contains("xmlns:custom"));
        assert!(result.contains("<ofd:OFD"));
    }

    #[test]
    fn test_no_prefix() {
        let xml = "<Root><Child/></Root>";
        let result = NamespaceCleaner::remove_ofd_prefix(xml);
        assert_eq!(result, xml);
    }

    #[test]
    fn test_empty_xml() {
        let result = NamespaceCleaner::remove_ofd_prefix("");
        assert!(result.is_empty());
    }
}
