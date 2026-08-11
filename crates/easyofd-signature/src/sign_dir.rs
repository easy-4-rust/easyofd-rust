//! 签名资源容器（SignDir）。
//!
//! 对应 Java: org.ofdrw.pkg.container.SignDir
//!
//! 单个签章目录（Sign_N/），包含签名描述文件、
//! 电子印章文件和签名值文件。

use std::path::PathBuf;

/// 签名容器名称前缀。
pub const SIGN_CONTAINER_PREFIX: &str = "Sign_";

/// 电子印章文件名。
pub const SEAL_FILE_NAME: &str = "Seal.esl";

/// 签名/签章描述文件名。
pub const SIGNATURE_FILE_NAME: &str = "Signature.xml";

/// 签名值文件名。
pub const SIGNED_VALUE_FILE_NAME: &str = "SignedValue.dat";

/// 签名资源容器。
///
/// 描述单个签章目录的结构，管理签名描述文件、
/// 电子印章文件和签名值文件的路径。
#[derive(Debug, Clone)]
pub struct SignDir {
    /// 签名容器的完整路径。
    pub path: PathBuf,
    /// 签名容器索引字符串（支持非规范名称）。
    pub index_str: String,
}

impl SignDir {
    /// 创建新的签名容器。
    ///
    /// # Arguments
    /// * `path` - 签名容器的完整路径（如 `/Doc_0/Signs/Sign_0`）
    #[must_use]
    pub fn new(path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        let index_str = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .replace(SIGN_CONTAINER_PREFIX, "");
        Self { path, index_str }
    }

    /// 获取签名容器索引（从 1 开始）。
    ///
    /// # Errors
    /// 如果索引不是有效的数字，返回错误。
    pub fn index(&self) -> Result<u32, String> {
        self.index_str
            .parse::<u32>()
            .map_err(|e| format!("解析签名索引失败: {e}"))
    }

    /// 获取签名容器索引字符串。
    #[must_use]
    pub fn index_str(&self) -> &str {
        &self.index_str
    }

    /// 获取签名描述文件路径。
    #[must_use]
    pub fn signature_path(&self) -> PathBuf {
        self.path.join(SIGNATURE_FILE_NAME)
    }

    /// 获取电子印章文件路径。
    #[must_use]
    pub fn seal_path(&self) -> PathBuf {
        self.path.join(SEAL_FILE_NAME)
    }

    /// 获取签名值文件路径。
    #[must_use]
    pub fn signed_value_path(&self) -> PathBuf {
        self.path.join(SIGNED_VALUE_FILE_NAME)
    }

    /// 获取签名容器名称。
    #[must_use]
    pub fn container_name(&self) -> &str {
        self.path.file_name().and_then(|n| n.to_str()).unwrap_or("")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_dir_new() {
        let sd = SignDir::new("/Doc_0/Signs/Sign_0");
        assert_eq!(sd.index_str(), "0");
        assert_eq!(sd.index().unwrap(), 0);
        assert_eq!(sd.container_name(), "Sign_0");
    }

    #[test]
    fn test_sign_dir_paths() {
        let sd = SignDir::new("/Doc_0/Signs/Sign_1");
        assert_eq!(
            sd.signature_path(),
            PathBuf::from("/Doc_0/Signs/Sign_1/Signature.xml")
        );
        assert_eq!(
            sd.seal_path(),
            PathBuf::from("/Doc_0/Signs/Sign_1/Seal.esl")
        );
        assert_eq!(
            sd.signed_value_path(),
            PathBuf::from("/Doc_0/Signs/Sign_1/SignedValue.dat")
        );
    }

    #[test]
    fn test_sign_dir_constants() {
        assert_eq!(SIGN_CONTAINER_PREFIX, "Sign_");
        assert_eq!(SEAL_FILE_NAME, "Seal.esl");
        assert_eq!(SIGNATURE_FILE_NAME, "Signature.xml");
        assert_eq!(SIGNED_VALUE_FILE_NAME, "SignedValue.dat");
    }

    #[test]
    fn test_sign_dir_index_str() {
        let sd = SignDir::new("/Doc_0/Signs/Sign_abc");
        // Non-numeric index should return error on index()
        assert!(sd.index().is_err());
        assert_eq!(sd.index_str(), "abc");
    }
}
