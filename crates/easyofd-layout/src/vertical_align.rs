//! 内容垂直对齐方式。
//!
//! 对应 Java: org.ofdrw.layout.element.canvas.VerticalAlign

/// 内容垂直对齐方式。
///
/// 对应 Java: ofdrw layout canvas VerticalAlign 枚举。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VerticalAlign {
    /// 顶部对齐。
    Top,
    /// 中间对齐。
    #[default]
    Center,
    /// 底部对齐。
    Bottom,
}

impl VerticalAlign {
    /// 是否为顶部对齐。
    #[must_use]
    pub fn is_top(self) -> bool {
        matches!(self, Self::Top)
    }

    /// 是否为中间对齐。
    #[must_use]
    pub fn is_center(self) -> bool {
        matches!(self, Self::Center)
    }

    /// 是否为底部对齐。
    #[must_use]
    pub fn is_bottom(self) -> bool {
        matches!(self, Self::Bottom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_is_center() {
        assert_eq!(VerticalAlign::default(), VerticalAlign::Center);
    }

    #[test]
    fn test_predicates() {
        assert!(VerticalAlign::Top.is_top());
        assert!(VerticalAlign::Center.is_center());
        assert!(VerticalAlign::Bottom.is_bottom());
        assert!(!VerticalAlign::Top.is_bottom());
        assert!(!VerticalAlign::Bottom.is_center());
    }

    #[test]
    fn test_clone_copy_eq() {
        let a = VerticalAlign::Bottom;
        let b = a;
        assert_eq!(a, b);
    }
}
