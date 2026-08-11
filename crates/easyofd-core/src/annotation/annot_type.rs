//! 注释类型枚举。

/// 对应 Java: org.ofdrw.core.annotation.AnnotType
///
/// 注释类型，对应 GB/T 33190 第 16 章中定义的各类注释。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnnotType {
    /// 链接注释。
    Link,
    /// 文本注释（便签）。
    Text,
    /// 高亮注释。
    Highlight,
    /// 图章注释。
    Stamp,
    /// 手写注释。
    Handwritten,
    /// 水印注释。
    Watermark,
}

impl AnnotType {
    /// 返回 OFD XML 中对应的类型字符串。
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Link => "Link",
            Self::Text => "Text",
            Self::Highlight => "Highlight",
            Self::Stamp => "Stamp",
            Self::Handwritten => "Handwritten",
            Self::Watermark => "Watermark",
        }
    }

    /// 从字符串解析注释类型。
    ///
    /// # Errors
    ///
    /// 当字符串不匹配任何已知类型时返回错误。
    pub fn from_str_opt(s: &str) -> Option<Self> {
        match s {
            "Link" => Some(Self::Link),
            "Text" => Some(Self::Text),
            "Highlight" => Some(Self::Highlight),
            "Stamp" => Some(Self::Stamp),
            "Handwritten" => Some(Self::Handwritten),
            "Watermark" => Some(Self::Watermark),
            _ => None,
        }
    }
}

impl std::fmt::Display for AnnotType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_annot_type_as_str() {
        assert_eq!(AnnotType::Link.as_str(), "Link");
        assert_eq!(AnnotType::Text.as_str(), "Text");
        assert_eq!(AnnotType::Highlight.as_str(), "Highlight");
        assert_eq!(AnnotType::Stamp.as_str(), "Stamp");
        assert_eq!(AnnotType::Handwritten.as_str(), "Handwritten");
        assert_eq!(AnnotType::Watermark.as_str(), "Watermark");
    }

    #[test]
    fn test_annot_type_from_str_opt() {
        assert_eq!(AnnotType::from_str_opt("Link"), Some(AnnotType::Link));
        assert_eq!(AnnotType::from_str_opt("Text"), Some(AnnotType::Text));
        assert_eq!(
            AnnotType::from_str_opt("Highlight"),
            Some(AnnotType::Highlight)
        );
        assert_eq!(AnnotType::from_str_opt("Stamp"), Some(AnnotType::Stamp));
        assert_eq!(
            AnnotType::from_str_opt("Handwritten"),
            Some(AnnotType::Handwritten)
        );
        assert_eq!(
            AnnotType::from_str_opt("Watermark"),
            Some(AnnotType::Watermark)
        );
        assert_eq!(AnnotType::from_str_opt("Unknown"), None);
    }

    #[test]
    fn test_annot_type_display() {
        assert_eq!(format!("{}", AnnotType::Link), "Link");
        assert_eq!(format!("{}", AnnotType::Stamp), "Stamp");
    }

    #[test]
    fn test_annot_type_clone_copy_debug() {
        let t = AnnotType::Highlight;
        let t2 = t;
        assert_eq!(t2, AnnotType::Highlight);
        let dbg = format!("{t:?}");
        assert!(dbg.contains("Highlight"));
    }
}
