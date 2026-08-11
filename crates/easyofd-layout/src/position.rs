//! 布局定位方式。
//!
//! 对应 Java: org.ofdrw.layout.element.Position

/// 元素定位方式（对应 Java: ofdrw layout Position）。
///
/// 对应 Java: ofdrw Position。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Position {
    /// 静态定位（由渲染器决定）。
    #[default]
    Static,
    /// 相对定位（相对父容器）。
    Relative,
    /// 绝对定位（相对页面/文档）。
    Absolute,
}

impl Position {
    /// 是否为绝对定位。
    #[must_use]
    pub fn is_absolute(self) -> bool {
        matches!(self, Self::Absolute)
    }

    /// 是否为相对定位。
    #[must_use]
    pub fn is_relative(self) -> bool {
        matches!(self, Self::Relative)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_is_static() {
        assert_eq!(Position::default(), Position::Static);
    }

    #[test]
    fn test_predicates() {
        assert!(Position::Absolute.is_absolute());
        assert!(Position::Relative.is_relative());
        assert!(!Position::Static.is_absolute());
    }
}
