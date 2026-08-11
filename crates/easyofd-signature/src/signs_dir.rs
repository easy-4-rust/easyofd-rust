//! 签名列表容器（SignsDir）。
//!
//! 对应 Java: org.ofdrw.pkg.container.SignsDir
//!
//! 签章列表目录（Signs/），包含 Signatures.xml 签名列表文件
//! 和多个签章子目录（Sign_N/）。

use std::path::PathBuf;

use crate::sign_dir::{SIGN_CONTAINER_PREFIX, SignDir};

/// 签名列表文件名称。
pub const SIGNATURES_FILE_NAME: &str = "Signatures.xml";

/// 签名列表容器。
///
/// 管理文档中的所有签章目录和签名列表文件。
/// 签名容器的索引从 0 开始。
#[derive(Debug, Clone)]
pub struct SignsDir {
    /// 签名列表容器的完整路径。
    pub path: PathBuf,
    /// 签名索引最大值 + 1（即下一个新签名的索引）。
    pub max_sign_index: u32,
}

impl SignsDir {
    /// 创建新的签名列表容器。
    ///
    /// # Arguments
    /// * `path` - 签名列表容器的完整路径（如 `/Doc_0/Signs`）
    #[must_use]
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            max_sign_index: 0,
        }
    }

    /// 从已有目录初始化，扫描已有的 Sign_N 子目录来设置索引。
    ///
    /// # Arguments
    /// * `path` - 签名列表容器的完整路径
    #[must_use]
    pub fn from_existing(path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        let mut max_index: u32 = 0;
        if let Ok(entries) = std::fs::read_dir(&path) {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str()
                    && name.starts_with(SIGN_CONTAINER_PREFIX)
                    && let Ok(num) = name.replace(SIGN_CONTAINER_PREFIX, "").parse::<u32>()
                    && max_index <= num
                {
                    max_index = num + 1;
                }
            }
        }
        Self {
            path,
            max_sign_index: max_index,
        }
    }

    /// 获取签名列表文件路径。
    #[must_use]
    pub fn signatures_path(&self) -> PathBuf {
        self.path.join(SIGNATURES_FILE_NAME)
    }

    /// 创建一个新的签名目录，返回 SignDir。
    pub fn new_sign_dir(&mut self) -> SignDir {
        let name = format!("{SIGN_CONTAINER_PREFIX}{}", self.max_sign_index);
        self.max_sign_index += 1;
        SignDir::new(self.path.join(name))
    }

    /// 获取指定索引的签名目录。
    ///
    /// # Arguments
    /// * `index` - 签名索引（从 0 开始）
    #[must_use]
    pub fn get_by_index(&self, index: u32) -> SignDir {
        let name = format!("{SIGN_CONTAINER_PREFIX}{index}");
        SignDir::new(self.path.join(name))
    }

    /// 获取签名目录数量（基于已创建的索引计数）。
    pub fn sign_count(&self) -> u32 {
        self.max_sign_index
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signs_dir_new() {
        let sd = SignsDir::new("/Doc_0/Signs");
        assert_eq!(sd.path, PathBuf::from("/Doc_0/Signs"));
        assert_eq!(sd.max_sign_index, 0);
        assert_eq!(sd.sign_count(), 0);
    }

    #[test]
    fn test_signs_dir_new_sign_dir() {
        let mut sd = SignsDir::new("/Doc_0/Signs");
        let sign0 = sd.new_sign_dir();
        assert_eq!(sign0.container_name(), "Sign_0");
        assert_eq!(sd.sign_count(), 1);

        let sign1 = sd.new_sign_dir();
        assert_eq!(sign1.container_name(), "Sign_1");
        assert_eq!(sd.sign_count(), 2);
    }

    #[test]
    fn test_signs_dir_get_by_index() {
        let sd = SignsDir::new("/Doc_0/Signs");
        let sign = sd.get_by_index(3);
        assert_eq!(sign.container_name(), "Sign_3");
        assert_eq!(sign.path, PathBuf::from("/Doc_0/Signs/Sign_3"));
    }

    #[test]
    fn test_signs_dir_signatures_path() {
        let sd = SignsDir::new("/Doc_0/Signs");
        assert_eq!(
            sd.signatures_path(),
            PathBuf::from("/Doc_0/Signs/Signatures.xml")
        );
    }

    #[test]
    fn test_signs_dir_constants() {
        assert_eq!(SIGNATURES_FILE_NAME, "Signatures.xml");
    }
}
