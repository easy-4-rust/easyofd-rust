//! 文字粗细值。
//!
//! 对应 Java: org.ofdrw.core.text.text.Weight

/// 文字对象的粗细值（GB/T 33190-2016 §11.3 表 45）。
///
/// 对应 Java: ofdrw Weight。可选值为 100 到 900，默认 400。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Weight {
    /// 100
    W100,
    /// 200
    W200,
    /// 300
    W300,
    /// 400（默认值）。
    W400,
    /// 500
    W500,
    /// 600
    W600,
    /// 700
    W700,
    /// 800
    W800,
    /// 900
    W900,
}

impl Weight {
    /// 数值（100-900）。
    #[must_use]
    pub fn value(self) -> u32 {
        match self {
            Self::W100 => 100,
            Self::W200 => 200,
            Self::W300 => 300,
            Self::W400 => 400,
            Self::W500 => 500,
            Self::W600 => 600,
            Self::W700 => 700,
            Self::W800 => 800,
            Self::W900 => 900,
        }
    }

    /// 根据数值解析（对应 Java: Weight.getInstance(int)）。
    ///
    /// 对应 Java: ofdrw Weight#getInstance。空值或非法值回退到 400。
    #[must_use]
    pub fn get_instance(weight: u32) -> Self {
        match weight {
            100 => Self::W100,
            200 => Self::W200,
            300 => Self::W300,
            500 => Self::W500,
            600 => Self::W600,
            700 => Self::W700,
            800 => Self::W800,
            900 => Self::W900,
            _ => Self::W400,
        }
    }

    /// 根据字符串解析（对应 Java: Weight.getInstance(String)）。
    #[must_use]
    pub fn get_instance_str(weight: &str) -> Self {
        Self::get_instance(weight.trim().parse().unwrap_or(400))
    }
}

impl From<u32> for Weight {
    fn from(value: u32) -> Self {
        Self::get_instance(value)
    }
}

impl std::fmt::Display for Weight {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_values() {
        assert_eq!(Weight::W100.value(), 100);
        assert_eq!(Weight::W400.value(), 400);
        assert_eq!(Weight::W900.value(), 900);
    }

    #[test]
    fn test_get_instance() {
        assert_eq!(Weight::get_instance(100), Weight::W100);
        assert_eq!(Weight::get_instance(700), Weight::W700);
        assert_eq!(Weight::get_instance(123), Weight::W400);
    }

    #[test]
    fn test_get_instance_str() {
        assert_eq!(Weight::get_instance_str("300"), Weight::W300);
        assert_eq!(Weight::get_instance_str(""), Weight::W400);
        assert_eq!(Weight::get_instance_str("bad"), Weight::W400);
    }

    #[test]
    fn test_display_and_from() {
        assert_eq!(Weight::W500.to_string(), "500");
        assert_eq!(Weight::from(900), Weight::W900);
    }
}
