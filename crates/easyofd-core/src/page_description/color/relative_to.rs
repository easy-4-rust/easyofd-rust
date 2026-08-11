//! 相对位置枚举。
//!
//! 对应 Java: org.ofdrw.core.pageDescription.color.pattern.RelativeTo

/// 相对位置。
///
/// 对应 Java: org.ofdrw.core.pageDescription.color.pattern.RelativeTo
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RelativeTo {
    /// 页面。
    Page,
    /// 对象。
    Object,
}

impl RelativeTo {
    /// 转为字符串。
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Page => "Page",
            Self::Object => "Object",
        }
    }
}

impl std::fmt::Display for RelativeTo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relative_to_display() {
        assert_eq!(RelativeTo::Page.to_string(), "Page");
        assert_eq!(RelativeTo::Object.to_string(), "Object");
    }
}
