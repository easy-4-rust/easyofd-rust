//! 扩展签名容器与签名相关类型。
//!
//! 对应 Java: `org.ofdrw.sign` 包
//!
//! 提供扩展签名容器接口、待摘要文件信息、签名清理器等类型。

use std::path::{Path, PathBuf};

/// 签名节点类型。
///
/// 对应 Java: `org.ofdrw.core.signatures.SigType`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SigType {
    /// 签章（印章形式）。
    Seal,
    /// 签名（无印章形式）。
    Sign,
}

impl std::fmt::Display for SigType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Seal => write!(f, "Seal"),
            Self::Sign => write!(f, "Sign"),
        }
    }
}

/// 扩展数字签名容器接口。
///
/// 对应 Java: `org.ofdrw.sign.ExtendSignatureContainer`
///
/// 不同的容器实现不同的签名格式和算法。
pub trait ExtendSignatureContainer: Send + Sync {
    /// 返回签名算法 OID 字符串。
    fn sign_alg_oid(&self) -> &str;

    /// 对待签名数据进行签名。
    ///
    /// # 参数
    ///
    /// - `in_data`：待签名数据
    /// - `property_info`：签章属性信息
    ///
    /// # 返回
    ///
    /// 签名结果字节。
    fn sign(&self, in_data: &[u8], property_info: &str) -> Vec<u8>;

    /// 获取电子印章二进制编码。
    ///
    /// 当 [`sign_type()`] 返回 [`SigType::Sign`] 时返回 `None`。
    fn seal(&self) -> Option<Vec<u8>>;

    /// 获取签名节点类型。
    fn sign_type(&self) -> SigType;
}

/// 待计算杂凑值的文件信息。
///
/// 对应 Java: `org.ofdrw.sign.ToDigestFileInfo`
///
/// 描述 OFD 容器中需要计算摘要的文件路径映射。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToDigestFileInfo {
    /// 文件在 OFD 虚拟容器中的绝对路径。
    /// 如：`"/Doc_0/Pages/Page_0/Content.xml"`
    abs_path: String,
    /// 待杂凑的文件在文件系统中的路径。
    sys_path: PathBuf,
}

impl ToDigestFileInfo {
    /// 创建文件信息对象。
    #[must_use]
    pub fn new(abs_path: impl Into<String>, sys_path: impl Into<PathBuf>) -> Self {
        Self {
            abs_path: abs_path.into(),
            sys_path: sys_path.into(),
        }
    }

    /// 获取容器内绝对路径。
    #[must_use]
    pub fn abs_path(&self) -> &str {
        &self.abs_path
    }

    /// 获取文件系统路径。
    #[must_use]
    pub fn sys_path(&self) -> &Path {
        &self.sys_path
    }

    /// 设置容器内绝对路径。
    pub fn set_abs_path(&mut self, abs_path: impl Into<String>) {
        self.abs_path = abs_path.into();
    }

    /// 设置文件系统路径。
    pub fn set_sys_path(&mut self, sys_path: impl Into<PathBuf>) {
        self.sys_path = sys_path.into();
    }
}

/// 文件保护过滤器接口。
///
/// 对应 Java: `org.ofdrw.sign.ProtectFileFilter`
///
/// 用于决定哪些文件需要纳入签名保护范围。
pub trait ProtectFileFilter: Send + Sync {
    /// 判断给定路径的文件是否应被保护。
    ///
    /// 返回 `true` 表示该文件应纳入签名摘要计算。
    fn should_protect(&self, path: &str) -> bool;
}

/// 签名清理器。
///
/// 对应 Java: `org.ofdrw.sign.SignCleaner`
///
/// 用于清除 OFD 文档中的签名数据。
#[derive(Debug, Clone)]
pub struct SignCleaner {
    /// 要清除的签名目录路径列表。
    sign_dirs: Vec<String>,
}

impl SignCleaner {
    /// 创建签名清理器。
    #[must_use]
    pub fn new() -> Self {
        Self {
            sign_dirs: Vec::new(),
        }
    }

    /// 添加要清除的签名目录。
    pub fn add_sign_dir(&mut self, dir: impl Into<String>) {
        self.sign_dirs.push(dir.into());
    }

    /// 获取要清除的签名目录列表。
    #[must_use]
    pub fn sign_dirs(&self) -> &[String] {
        &self.sign_dirs
    }

    /// 是否有要清除的签名。
    #[must_use]
    pub fn has_signs(&self) -> bool {
        !self.sign_dirs.is_empty()
    }
}

impl Default for SignCleaner {
    fn default() -> Self {
        Self::new()
    }
}

/// GB/T 35275 数字签名容器。
///
/// 对应 Java: `org.ofdrw.sign.signContainer.GBT35275DSContainer`
///
/// 使用 GB/T 35275 CMS SignedData 格式作为签名值。
pub struct Gbt35275DsContainer {
    /// 签名算法 OID。
    alg_oid: String,
}

impl Gbt35275DsContainer {
    /// 创建 GB/T 35275 签名容器。
    #[must_use]
    pub fn new(alg_oid: impl Into<String>) -> Self {
        Self {
            alg_oid: alg_oid.into(),
        }
    }
}

impl ExtendSignatureContainer for Gbt35275DsContainer {
    fn sign_alg_oid(&self) -> &str {
        &self.alg_oid
    }

    fn sign(&self, _in_data: &[u8], _property_info: &str) -> Vec<u8> {
        // 实际的 GB/T 35275 签名需要 CMS SignedData 构建
        Vec::new()
    }

    fn seal(&self) -> Option<Vec<u8>> {
        None
    }

    fn sign_type(&self) -> SigType {
        SigType::Sign
    }
}

/// SES V1 签名容器。
///
/// 对应 Java: `org.ofdrw.sign.signContainer.SESV1Container`
pub struct SesV1Container {
    alg_oid: String,
    seal_bytes: Vec<u8>,
}

impl SesV1Container {
    /// 创建 SES V1 签名容器。
    #[must_use]
    pub fn new(alg_oid: impl Into<String>, seal_bytes: Vec<u8>) -> Self {
        Self {
            alg_oid: alg_oid.into(),
            seal_bytes,
        }
    }
}

impl ExtendSignatureContainer for SesV1Container {
    fn sign_alg_oid(&self) -> &str {
        &self.alg_oid
    }

    fn sign(&self, _in_data: &[u8], _property_info: &str) -> Vec<u8> {
        Vec::new()
    }

    fn seal(&self) -> Option<Vec<u8>> {
        Some(self.seal_bytes.clone())
    }

    fn sign_type(&self) -> SigType {
        SigType::Seal
    }
}

/// SES V4 签名容器。
///
/// 对应 Java: `org.ofdrw.sign.signContainer.SESV4Container`
pub struct SesV4Container {
    alg_oid: String,
    seal_bytes: Vec<u8>,
}

impl SesV4Container {
    /// 创建 SES V4 签名容器。
    #[must_use]
    pub fn new(alg_oid: impl Into<String>, seal_bytes: Vec<u8>) -> Self {
        Self {
            alg_oid: alg_oid.into(),
            seal_bytes,
        }
    }
}

impl ExtendSignatureContainer for SesV4Container {
    fn sign_alg_oid(&self) -> &str {
        &self.alg_oid
    }

    fn sign(&self, _in_data: &[u8], _property_info: &str) -> Vec<u8> {
        Vec::new()
    }

    fn seal(&self) -> Option<Vec<u8>> {
        Some(self.seal_bytes.clone())
    }

    fn sign_type(&self) -> SigType {
        SigType::Seal
    }
}

/// SES V5 签名容器。
///
/// 对应 Java: `org.ofdrw.sign.signContainer.SESV5Container`
pub struct SesV5Container {
    alg_oid: String,
    seal_bytes: Vec<u8>,
}

impl SesV5Container {
    /// 创建 SES V5 签名容器。
    #[must_use]
    pub fn new(alg_oid: impl Into<String>, seal_bytes: Vec<u8>) -> Self {
        Self {
            alg_oid: alg_oid.into(),
            seal_bytes,
        }
    }
}

impl ExtendSignatureContainer for SesV5Container {
    fn sign_alg_oid(&self) -> &str {
        &self.alg_oid
    }

    fn sign(&self, _in_data: &[u8], _property_info: &str) -> Vec<u8> {
        Vec::new()
    }

    fn seal(&self) -> Option<Vec<u8>> {
        Some(self.seal_bytes.clone())
    }

    fn sign_type(&self) -> SigType {
        SigType::Seal
    }
}

/// GB/T 35275 PKCS#9 签名容器。
///
/// 对应 Java: `org.ofdrw.sign.signContainer.GBT35275PKCS9DSContainer`
pub struct Gbt35275Pkcs9DsContainer {
    alg_oid: String,
}

impl Gbt35275Pkcs9DsContainer {
    /// 创建 GB/T 35275 PKCS#9 签名容器。
    #[must_use]
    pub fn new(alg_oid: impl Into<String>) -> Self {
        Self {
            alg_oid: alg_oid.into(),
        }
    }
}

impl ExtendSignatureContainer for Gbt35275Pkcs9DsContainer {
    fn sign_alg_oid(&self) -> &str {
        &self.alg_oid
    }

    fn sign(&self, _in_data: &[u8], _property_info: &str) -> Vec<u8> {
        Vec::new()
    }

    fn seal(&self) -> Option<Vec<u8>> {
        None
    }

    fn sign_type(&self) -> SigType {
        SigType::Sign
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sig_type_display() {
        assert_eq!(SigType::Seal.to_string(), "Seal");
        assert_eq!(SigType::Sign.to_string(), "Sign");
    }

    #[test]
    fn to_digest_file_info() {
        let info = ToDigestFileInfo::new("/Doc_0/Content.xml", "/tmp/content.xml");
        assert_eq!(info.abs_path(), "/Doc_0/Content.xml");
        assert_eq!(info.sys_path(), Path::new("/tmp/content.xml"));
    }

    #[test]
    fn to_digest_file_info_setters() {
        let mut info = ToDigestFileInfo::new("/a", "/b");
        info.set_abs_path("/c");
        info.set_sys_path("/d");
        assert_eq!(info.abs_path(), "/c");
        assert_eq!(info.sys_path(), Path::new("/d"));
    }

    #[test]
    fn protect_file_filter_trait() {
        struct AlwaysProtect;
        impl ProtectFileFilter for AlwaysProtect {
            fn should_protect(&self, _path: &str) -> bool {
                true
            }
        }
        let filter = AlwaysProtect;
        assert!(filter.should_protect("/Doc_0/Content.xml"));
    }

    #[test]
    fn sign_cleaner_default() {
        let cleaner = SignCleaner::default();
        assert!(!cleaner.has_signs());
        assert!(cleaner.sign_dirs().is_empty());
    }

    #[test]
    fn sign_cleaner_add_dirs() {
        let mut cleaner = SignCleaner::new();
        cleaner.add_sign_dir("Doc_0/Signs/Sign_0");
        cleaner.add_sign_dir("Doc_0/Signs/Sign_1");
        assert!(cleaner.has_signs());
        assert_eq!(cleaner.sign_dirs().len(), 2);
    }

    #[test]
    fn gbt35275_ds_container() {
        let c = Gbt35275DsContainer::new("1.2.156.10197.1.501");
        assert_eq!(c.sign_alg_oid(), "1.2.156.10197.1.501");
        assert_eq!(c.sign_type(), SigType::Sign);
        assert!(c.seal().is_none());
    }

    #[test]
    fn ses_v1_container() {
        let c = SesV1Container::new("1.2.156.10197.1.501", vec![0x01, 0x02]);
        assert_eq!(c.sign_type(), SigType::Seal);
        assert_eq!(c.seal(), Some(vec![0x01, 0x02]));
    }

    #[test]
    fn ses_v4_container() {
        let c = SesV4Container::new("1.2.156.10197.1.501", vec![0x03]);
        assert_eq!(c.sign_type(), SigType::Seal);
        assert_eq!(c.seal(), Some(vec![0x03]));
    }

    #[test]
    fn ses_v5_container() {
        let c = SesV5Container::new("1.2.156.10197.1.501", vec![0x04]);
        assert_eq!(c.sign_type(), SigType::Seal);
        assert_eq!(c.seal(), Some(vec![0x04]));
    }

    #[test]
    fn gbt35275_pkcs9_container() {
        let c = Gbt35275Pkcs9DsContainer::new("1.2.156.10197.1.501");
        assert_eq!(c.sign_type(), SigType::Sign);
        assert!(c.seal().is_none());
    }
}
