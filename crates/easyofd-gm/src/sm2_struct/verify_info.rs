//! 签名验证信息。
//!
//! 对应 Java: `org.ofdrw.gm.sm2strut.VerifyInfo`

/// 签名验证结果信息。
///
/// 对应 Java: `org.ofdrw.gm.sm2strut.VerifyInfo`
///
/// 用于包装 SM2 签名验证的结果，包含是否通过与错误描述。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifyInfo {
    /// 验证是否通过。
    pub result: bool,
    /// 不通过时的错误描述（通过时为空字符串）。
    pub hit: String,
}

impl VerifyInfo {
    /// 创建验证信息。
    #[must_use]
    pub fn new(result: bool, hit: impl Into<String>) -> Self {
        Self {
            result,
            hit: hit.into(),
        }
    }

    /// 验证成功。
    #[must_use]
    pub fn ok() -> Self {
        Self {
            result: true,
            hit: String::new(),
        }
    }

    /// 验证失败。
    #[must_use]
    pub fn err(description: impl Into<String>) -> Self {
        Self {
            result: false,
            hit: description.into(),
        }
    }

    /// 是否验证通过。
    #[must_use]
    pub fn is_ok(&self) -> bool {
        self.result
    }

    /// 是否验证失败。
    #[must_use]
    pub fn is_err(&self) -> bool {
        !self.result
    }
}

impl std::fmt::Display for VerifyInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.result {
            write!(f, "验证通过")
        } else {
            write!(f, "验证失败: {}", self.hit)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ok_state() {
        let info = VerifyInfo::ok();
        assert!(info.is_ok());
        assert!(!info.is_err());
        assert!(info.hit.is_empty());
    }

    #[test]
    fn err_state() {
        let info = VerifyInfo::err("证书过期");
        assert!(!info.is_ok());
        assert!(info.is_err());
        assert_eq!(info.hit, "证书过期");
    }

    #[test]
    fn new_constructor() {
        let info = VerifyInfo::new(true, "");
        assert!(info.is_ok());

        let info = VerifyInfo::new(false, "签名值不匹配");
        assert!(info.is_err());
        assert_eq!(info.hit, "签名值不匹配");
    }

    #[test]
    fn display_format() {
        assert_eq!(VerifyInfo::ok().to_string(), "验证通过");
        assert_eq!(
            VerifyInfo::err("摘要不一致").to_string(),
            "验证失败: 摘要不一致"
        );
    }
}
