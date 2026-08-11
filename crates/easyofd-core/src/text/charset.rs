//! 字符集枚举。
//!
//! 对应 Java: org.ofdrw.core.text.font.Charset

/// 字符集。
///
/// 对应 Java: org.ofdrw.core.text.font.Charset
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Charset {
    /// 符号字符集。
    Symbol,
    /// Unicode 字符集。
    Unicode,
    /// GBK 字符集。
    Gbk,
    /// GB2312 字符集。
    Gb2312,
    /// Big5 字符集。
    Big5,
}

impl Charset {
    /// 转为字符串。
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Symbol => "Symbol",
            Self::Unicode => "Unicode",
            Self::Gbk => "GBK",
            Self::Gb2312 => "GB2312",
            Self::Big5 => "Big5",
        }
    }

    /// 从字符串解析。
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s.trim() {
            "Symbol" => Some(Self::Symbol),
            "Unicode" => Some(Self::Unicode),
            "GBK" => Some(Self::Gbk),
            "GB2312" => Some(Self::Gb2312),
            "Big5" => Some(Self::Big5),
            _ => None,
        }
    }
}

impl std::fmt::Display for Charset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn charset_as_str() {
        assert_eq!(Charset::Unicode.as_str(), "Unicode");
        assert_eq!(Charset::Gbk.as_str(), "GBK");
    }

    #[test]
    fn charset_from_str() {
        assert_eq!(Charset::from_str("Unicode"), Some(Charset::Unicode));
        assert_eq!(Charset::from_str("GBK"), Some(Charset::Gbk));
        assert_eq!(Charset::from_str("Unknown"), None);
    }

    #[test]
    fn charset_display() {
        assert_eq!(Charset::Gb2312.to_string(), "GB2312");
    }
}
