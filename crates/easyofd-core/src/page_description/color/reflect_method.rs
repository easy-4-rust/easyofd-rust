//! 反射方法枚举。
//!
//! 对应 Java: org.ofdrw.core.pageDescription.color.pattern.ReflectMethod

/// 反射方法。
///
/// 对应 Java: org.ofdrw.core.pageDescription.color.pattern.ReflectMethod
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReflectMethod {
    /// 无反射。
    None,
    /// 水平反射。
    Horizontal,
    /// 垂直反射。
    Vertical,
}

impl ReflectMethod {
    /// 转为字符串。
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::None => "None",
            Self::Horizontal => "Horizontal",
            Self::Vertical => "Vertical",
        }
    }
}

impl std::fmt::Display for ReflectMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reflect_method_display() {
        assert_eq!(ReflectMethod::Horizontal.to_string(), "Horizontal");
    }
}
