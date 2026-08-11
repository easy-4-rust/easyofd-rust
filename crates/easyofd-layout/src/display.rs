//! Div 在行内的表现形式（显示方式）。
//!
//! 对应 Java: org.ofdrw.layout.element.Display

/// 元素在行内的显示方式。
///
/// 对应 Java: ofdrw layout Display 枚举。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Display {
    /// 块级元素：在正常流中，该元素之前和之后产生换行。
    #[default]
    Block,
    /// 内联块级元素：在正常流中不产生换行，除非前一个元素是块级元素盒。
    InlineBlock,
}

impl Display {
    /// 是否为块级元素。
    #[must_use]
    pub fn is_block(self) -> bool {
        matches!(self, Self::Block)
    }

    /// 是否为内联块级元素。
    #[must_use]
    pub fn is_inline_block(self) -> bool {
        matches!(self, Self::InlineBlock)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_is_block() {
        assert_eq!(Display::default(), Display::Block);
    }

    #[test]
    fn test_predicates() {
        assert!(Display::Block.is_block());
        assert!(!Display::Block.is_inline_block());
        assert!(Display::InlineBlock.is_inline_block());
        assert!(!Display::InlineBlock.is_block());
    }

    #[test]
    fn test_clone_copy_eq() {
        let a = Display::InlineBlock;
        let b = a;
        assert_eq!(a, b);
    }
}
