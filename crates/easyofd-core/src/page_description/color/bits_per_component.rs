//! 每分量位数枚举。
//!
//! 对应 Java: org.ofdrw.core.pageDescription.color.colorSpace.BitsPerComponent

/// 每分量位数。
///
/// 对应 Java: org.ofdrw.core.pageDescription.color.colorSpace.BitsPerComponent
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BitsPerComponent {
    /// 1 位。
    One,
    /// 2 位。
    Two,
    /// 4 位。
    Four,
    /// 8 位。
    Eight,
    /// 16 位。
    Sixteen,
}

impl BitsPerComponent {
    /// 获取位数值。
    #[must_use]
    pub fn value(&self) -> u8 {
        match self {
            Self::One => 1,
            Self::Two => 2,
            Self::Four => 4,
            Self::Eight => 8,
            Self::Sixteen => 16,
        }
    }

    /// 从数值创建。
    pub fn from_value(v: u8) -> Option<Self> {
        match v {
            1 => Some(Self::One),
            2 => Some(Self::Two),
            4 => Some(Self::Four),
            8 => Some(Self::Eight),
            16 => Some(Self::Sixteen),
            _ => None,
        }
    }
}

impl std::fmt::Display for BitsPerComponent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bits_per_component_value() {
        assert_eq!(BitsPerComponent::Eight.value(), 8);
        assert_eq!(BitsPerComponent::Sixteen.value(), 16);
    }

    #[test]
    fn bits_per_component_from_value() {
        assert_eq!(
            BitsPerComponent::from_value(8),
            Some(BitsPerComponent::Eight)
        );
        assert_eq!(BitsPerComponent::from_value(3), None);
    }

    #[test]
    fn bits_per_component_display() {
        assert_eq!(BitsPerComponent::Four.to_string(), "4");
    }
}
