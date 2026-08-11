//! 边标志枚举。
//!
//! 对应 Java: org.ofdrw.core.pageDescription.color.color.EdgeFlag

/// 边标志。
///
/// 对应 Java: org.ofdrw.core.pageDescription.color.color.EdgeFlag
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EdgeFlag {
    /// 无边。
    None,
    /// Top 边。
    Top,
    /// Right 边。
    Right,
    /// Bottom 边。
    Bottom,
    /// Left 边。
    Left,
}

impl EdgeFlag {
    /// 转为字符串。
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::None => "None",
            Self::Top => "Top",
            Self::Right => "Right",
            Self::Bottom => "Bottom",
            Self::Left => "Left",
        }
    }
}

impl std::fmt::Display for EdgeFlag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edge_flag_as_str() {
        assert_eq!(EdgeFlag::Top.as_str(), "Top");
        assert_eq!(EdgeFlag::None.as_str(), "None");
    }

    #[test]
    fn edge_flag_display() {
        assert_eq!(EdgeFlag::Right.to_string(), "Right");
    }
}
