//! 段中的浮动方向。
//!
//! 对应 Java: org.ofdrw.layout.element.AFloat

/// 元素在段中的浮动方向。
///
/// 对应 Java: ofdrw layout AFloat 枚举。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AFloat {
    /// 向左浮动。
    #[default]
    Left,
    /// 向右浮动。
    Right,
    /// 居中浮动。
    Center,
}

impl AFloat {
    /// 是否为左浮动。
    #[must_use]
    pub fn is_left(self) -> bool {
        matches!(self, Self::Left)
    }

    /// 是否为右浮动。
    #[must_use]
    pub fn is_right(self) -> bool {
        matches!(self, Self::Right)
    }

    /// 是否为居中浮动。
    #[must_use]
    pub fn is_center(self) -> bool {
        matches!(self, Self::Center)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_is_left() {
        assert_eq!(AFloat::default(), AFloat::Left);
    }

    #[test]
    fn test_predicates() {
        assert!(AFloat::Left.is_left());
        assert!(AFloat::Right.is_right());
        assert!(AFloat::Center.is_center());
        assert!(!AFloat::Left.is_right());
        assert!(!AFloat::Right.is_center());
        assert!(!AFloat::Center.is_left());
    }

    #[test]
    fn test_clone_copy_eq() {
        let a = AFloat::Center;
        let b = a;
        assert_eq!(a, b);
    }
}
