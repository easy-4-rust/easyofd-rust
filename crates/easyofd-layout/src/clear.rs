//! 段的占用情况（浮动清除方式）。
//!
//! 对应 Java: org.ofdrw.layout.element.Clear

/// 段的占用情况，控制元素在浮动方向上的排布。
///
/// 对应 Java: ofdrw layout Clear 枚举。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Clear {
    /// 共享：两侧都允许出现元素。
    #[default]
    None,
    /// 左侧不允许出现元素。
    Left,
    /// 右侧不允许出现元素。
    Right,
    /// 两侧不允许出现元素。
    Both,
}

impl Clear {
    /// 是否需要清除左侧浮动。
    #[must_use]
    pub fn clears_left(self) -> bool {
        matches!(self, Self::Left | Self::Both)
    }

    /// 是否需要清除右侧浮动。
    #[must_use]
    pub fn clears_right(self) -> bool {
        matches!(self, Self::Right | Self::Both)
    }

    /// 是否需要清除两侧浮动。
    #[must_use]
    pub fn clears_both(self) -> bool {
        matches!(self, Self::Both)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_is_none() {
        assert_eq!(Clear::default(), Clear::None);
    }

    #[test]
    fn test_clears_left() {
        assert!(Clear::Left.clears_left());
        assert!(Clear::Both.clears_left());
        assert!(!Clear::None.clears_left());
        assert!(!Clear::Right.clears_left());
    }

    #[test]
    fn test_clears_right() {
        assert!(Clear::Right.clears_right());
        assert!(Clear::Both.clears_right());
        assert!(!Clear::None.clears_right());
        assert!(!Clear::Left.clears_right());
    }

    #[test]
    fn test_clears_both() {
        assert!(Clear::Both.clears_both());
        assert!(!Clear::None.clears_both());
        assert!(!Clear::Left.clears_both());
        assert!(!Clear::Right.clears_both());
    }

    #[test]
    fn test_clone_copy_eq() {
        let a = Clear::Both;
        let b = a;
        assert_eq!(a, b);
    }
}
