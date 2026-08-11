//! 摘要算法枚举（CheckMethod）。
//!
//! 对应 Java: org.ofdrw.core.signatures.range.CheckMethod
//!
//! GB/T 33190 第 18.2.2 节，用于签名范围（References）中指定摘要算法。

/// 摘要算法枚举。
///
/// 对应 Java: `org.ofdrw.core.signatures.range.CheckMethod`
///
/// 在签名的范围（References）中使用，标识对受保护文件计算摘要时所用的算法。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CheckMethod {
    /// MD5 摘要算法（默认值）。
    Md5,
    /// SHA-1 摘要算法。
    Sha1,
    /// SHA-256 摘要算法。
    Sha256,
    /// SM3 国密杂凑算法。
    Sm3,
}

impl CheckMethod {
    /// 返回与 Java `CheckMethod.toString()` 一致的属性字符串。
    ///
    /// 产出值直接用于 XML 属性 `CheckMethod="..."` 的值。
    ///
    /// # 示例
    ///
    /// ```
    /// use easyofd_core::signatures::CheckMethod;
    ///
    /// assert_eq!(CheckMethod::Md5.as_str(), "MD5");
    /// assert_eq!(CheckMethod::Sha1.as_str(), "SHA1");
    /// assert_eq!(CheckMethod::Sha256.as_str(), "SHA256");
    /// assert_eq!(CheckMethod::Sm3.as_str(), "SM3");
    /// ```
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Md5 => "MD5",
            Self::Sha1 => "SHA1",
            Self::Sha256 => "SHA256",
            Self::Sm3 => "SM3",
        }
    }

    /// 从字符串解析摘要算法（不区分大小写）。
    ///
    /// # 错误
    ///
    /// 无法识别的算法名称返回 `Err`。
    ///
    /// # 示例
    ///
    /// ```
    /// use easyofd_core::signatures::CheckMethod;
    ///
    /// assert_eq!(CheckMethod::try_from_str("SM3").unwrap(), CheckMethod::Sm3);
    /// assert_eq!(CheckMethod::try_from_str("sha256").unwrap(), CheckMethod::Sha256);
    /// assert!(CheckMethod::try_from_str("UNKNOWN").is_err());
    /// ```
    pub fn try_from_str(s: &str) -> Result<Self, String> {
        match s.to_uppercase().as_str() {
            "MD5" => Ok(Self::Md5),
            "SHA1" => Ok(Self::Sha1),
            "SHA256" => Ok(Self::Sha256),
            "SM3" => Ok(Self::Sm3),
            other => Err(format!("未知的摘要算法: {other}")),
        }
    }
}

/// 为 `CheckMethod` 实现 `Display`，产出与 `as_str()` 一致。
impl std::fmt::Display for CheckMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn as_str_matches_java() {
        // 确保与 Java 枚举 toString() 产出完全一致
        assert_eq!(CheckMethod::Md5.as_str(), "MD5");
        assert_eq!(CheckMethod::Sha1.as_str(), "SHA1");
        assert_eq!(CheckMethod::Sha256.as_str(), "SHA256");
        assert_eq!(CheckMethod::Sm3.as_str(), "SM3");
    }

    #[test]
    fn display_matches_as_str() {
        for method in [
            CheckMethod::Md5,
            CheckMethod::Sha1,
            CheckMethod::Sha256,
            CheckMethod::Sm3,
        ] {
            assert_eq!(method.to_string(), method.as_str());
        }
    }

    #[test]
    fn try_from_str_roundtrip() {
        for method in [
            CheckMethod::Md5,
            CheckMethod::Sha1,
            CheckMethod::Sha256,
            CheckMethod::Sm3,
        ] {
            let s = method.as_str();
            let parsed = CheckMethod::try_from_str(s).unwrap();
            assert_eq!(parsed, method);
        }
    }

    #[test]
    fn try_from_str_case_insensitive() {
        assert_eq!(
            CheckMethod::try_from_str("sha1").unwrap(),
            CheckMethod::Sha1
        );
        assert_eq!(
            CheckMethod::try_from_str("Sha256").unwrap(),
            CheckMethod::Sha256
        );
        assert_eq!(CheckMethod::try_from_str("sm3").unwrap(), CheckMethod::Sm3);
        assert_eq!(CheckMethod::try_from_str("md5").unwrap(), CheckMethod::Md5);
    }

    #[test]
    fn try_from_str_unknown() {
        assert!(CheckMethod::try_from_str("UNKNOWN").is_err());
        assert!(CheckMethod::try_from_str("").is_err());
    }

    #[test]
    fn copy_eq() {
        let a = CheckMethod::Sm3;
        let b = a;
        assert_eq!(a, b);
    }

    #[test]
    fn hash_works() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(CheckMethod::Md5);
        set.insert(CheckMethod::Sha1);
        set.insert(CheckMethod::Sha256);
        set.insert(CheckMethod::Sm3);
        assert_eq!(set.len(), 4);
        assert!(set.contains(&CheckMethod::Sm3));
    }
}
