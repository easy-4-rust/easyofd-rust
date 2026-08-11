//! 扩展签名容器与签名相关类型。
//!
//! 对应 Java: `org.ofdrw.sign.signContainer` 包
//!
//! 每个容器实现 [`ExtendSignatureContainer`] trait，提供不同格式的签名值。

mod gbt35275_ds_container;
mod gbt35275_pkcs9_ds_container;
mod ses_v1_container;
mod ses_v4_container;
mod ses_v5_container;

use std::path::{Path, PathBuf};

pub use gbt35275_ds_container::Gbt35275DsContainer;
pub use gbt35275_pkcs9_ds_container::Gbt35275Pkcs9DsContainer;
pub use ses_v1_container::SesV1Container;
pub use ses_v4_container::SesV4Container;
pub use ses_v5_container::SesV5Container;

// ── 公共常量 ─────────────────────────────────────────────────────────────

/// SM2WithSM3 签名算法 OID 字符串。
///
/// 对应 Java: `GMObjectIdentifiers.sm2sign_with_sm3`
pub(crate) const SM2_SM3_OID_STR: &str = "1.2.156.10197.1.501";

/// SM2WithSM3 OID 弧段。
pub(crate) const SM2_SM3_OID_ARCS: &[u32] = &[1, 2, 156, 10_197, 1, 501];

/// SM3WithSM2 签名的默认用户 ID。
///
/// 对应 GB/T 32918.2-2016，推荐默认 ID 为 "1234567812345678"。
pub(crate) const SM2_DEFAULT_USER_ID: &str = "1234567812345678";

// ── 公共辅助函数 ─────────────────────────────────────────────────────────

/// SM3WithSM2 签名。
///
/// 对应 Java: `Signature.getInstance("SM3WithSM2", new BouncyCastleProvider())`
///
/// 使用 SM2 默认用户 ID ("1234567812345678") 对 `data` 进行 SM3WithSM2 签名。
pub(crate) fn sm2_sign_with_sm3(secret_key: &sm2::SecretKey, data: &[u8]) -> Vec<u8> {
    use sm2::dsa::signature::Signer;
    let signing_key =
        sm2::dsa::SigningKey::new(SM2_DEFAULT_USER_ID, secret_key).expect("SM2 密钥派生失败");
    signing_key.sign(data).to_bytes().to_vec()
}

/// 当前 UTC 时间 GeneralizedTime 格式 "YYYYMMDDHHmmSSZ"。
///
/// 对应 Java: `new ASN1GeneralizedTime(new Date(), Locale.CHINA)`
pub(crate) fn generalized_time_now() -> String {
    chrono::Utc::now().format("%Y%m%d%H%M%SZ").to_string()
}

/// 当前本地时间 UTF-8 字节，用于 V1 timeInfo。
///
/// 对应 Java: `LocalDateTime.now().format("yyyy-MM-dd HH:mm:ss").getBytes(UTF_8)`
pub(crate) fn local_time_utf8_bytes() -> Vec<u8> {
    chrono::Local::now()
        .format("%Y-%m-%d %H:%M:%S")
        .to_string()
        .into_bytes()
}

// ── 签名节点类型 ────────────────────────────────────────────────────────

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

// ── 扩展签名容器接口 ────────────────────────────────────────────────────

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
    /// - `in_data`：待签名数据（Signature.xml 原文）
    /// - `property_info`：签章属性信息
    ///
    /// # 返回
    ///
    /// 签名结果字节（DER 编码或原始签名值）。
    fn sign(&self, in_data: &[u8], property_info: &str) -> Vec<u8>;

    /// 获取电子印章二进制编码。
    ///
    /// 当 [`sign_type()`] 返回 [`SigType::Sign`] 时返回 `None`。
    fn seal(&self) -> Option<Vec<u8>>;

    /// 获取签名节点类型。
    fn sign_type(&self) -> SigType;
}

// ── 待计算杂凑值的文件信息 ──────────────────────────────────────────────

/// 待计算杂凑值的文件信息。
///
/// 对应 Java: `org.ofdrw.sign.ToDigestFileInfo`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToDigestFileInfo {
    abs_path: String,
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

// ── 文件保护过滤器 ──────────────────────────────────────────────────────

/// 文件保护过滤器接口。
///
/// 对应 Java: `org.ofdrw.sign.ProtectFileFilter`
pub trait ProtectFileFilter: Send + Sync {
    /// 判断给定路径的文件是否应被保护。
    fn should_protect(&self, path: &str) -> bool;
}

// ── 签名清理器 ──────────────────────────────────────────────────────────

/// 签名清理器。
///
/// 对应 Java: `org.ofdrw.sign.SignCleaner`
#[derive(Debug, Clone)]
pub struct SignCleaner {
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

// ── 模块内测试 ──────────────────────────────────────────────────────────

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
    fn sm2_sign_with_sm3_produces_signature() {
        use sm2::elliptic_curve::Generate;
        let sk = sm2::SecretKey::generate();
        let sig = sm2_sign_with_sm3(&sk, b"test data");
        assert!(!sig.is_empty());
        assert_eq!(sig.len(), 64);
    }

    #[test]
    fn generalized_time_now_format() {
        let gt = generalized_time_now();
        assert!(gt.ends_with('Z'));
        assert_eq!(gt.len(), 15);
    }
}
