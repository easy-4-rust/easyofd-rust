//! 电子签章版本号枚举。
//!
//! 对应 Java: `org.ofdrw.gm.ses.parse.SESVersion`

/// 电子签章版本号。
///
/// 对应 Java: `org.ofdrw.gm.ses.parse.SESVersion`
///
/// - `V1`：GM/T 0031-2014 安全电子签章密码技术规范
/// - `V4`：GB/T 38540-2020（展平结构、GeneralizedTime）
/// - `V5`：GM/T 0031-2025（V4 + 可选 timeStamp）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SESVersion {
    /// V1 版本（GM/T 0031-2014）。
    V1,
    /// V4 版本（GB/T 38540-2020）。
    V4,
    /// V5 版本（GM/T 0031-2025）。
    V5,
}

impl SESVersion {
    /// 获取版本对应的整数值。
    #[must_use]
    pub fn version_number(self) -> u32 {
        match self {
            Self::V1 => 1,
            Self::V4 => 4,
            Self::V5 => 5,
        }
    }

    /// 从整数版本号解析。
    ///
    /// # 错误
    ///
    /// 版本号不在 `{1, 4, 5}` 范围内时返回错误字符串。
    pub fn from_version(n: u32) -> Result<Self, &'static str> {
        match n {
            1 => Ok(Self::V1),
            4 => Ok(Self::V4),
            5 => Ok(Self::V5),
            _ => Err("未知的 SES 版本号"),
        }
    }
}

impl std::fmt::Display for SESVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::V1 => write!(f, "v1"),
            Self::V4 => write!(f, "v4"),
            Self::V5 => write!(f, "v5"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_number_roundtrip() {
        for v in [SESVersion::V1, SESVersion::V4, SESVersion::V5] {
            assert_eq!(SESVersion::from_version(v.version_number()).unwrap(), v);
        }
    }

    #[test]
    fn from_version_invalid() {
        assert!(SESVersion::from_version(0).is_err());
        assert!(SESVersion::from_version(2).is_err());
        assert!(SESVersion::from_version(99).is_err());
    }

    #[test]
    fn display_format() {
        assert_eq!(SESVersion::V1.to_string(), "v1");
        assert_eq!(SESVersion::V4.to_string(), "v4");
        assert_eq!(SESVersion::V5.to_string(), "v5");
    }

    #[test]
    fn version_number_values() {
        assert_eq!(SESVersion::V1.version_number(), 1);
        assert_eq!(SESVersion::V4.version_number(), 4);
        assert_eq!(SESVersion::V5.version_number(), 5);
    }
}
