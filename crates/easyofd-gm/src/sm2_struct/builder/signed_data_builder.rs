//! SignedData 构建器。
//!
//! 对应 Java: org.ofdrw.gm.sm2strut.builder.SignedDataBuilder
//!
//! 流式构建 GB/T 35275 SignedData 结构，替代 Java 版的 builder 模式。
//! Java 版通过链式调用 setDigestAlgorithm / setContentInfo / addSignerInfo 等
//! 方法构建 SignedData；Rust 版提供等价的 builder 接口。

use crate::sm2_struct::content_info::ContentInfo;
use crate::sm2_struct::oids::{SM3, parse_oid};
use crate::sm2_struct::signed_data::SignedData;
use crate::sm2_struct::signer_info::SignerInfo;

/// SignedData 构建器。
///
/// 对应 Java: org.ofdrw.gm.sm2strut.builder.SignedDataBuilder
///
/// 流式构建 [`SignedData`]，默认使用 SM3 摘要和 SM2 签名算法。
///
/// # 示例
///
/// ```rust
/// use easyofd_gm::sm2_struct::builder::SignedDataBuilder;
/// use easyofd_gm::sm2_struct::ContentInfo;
/// use easyofd_gm::sm2_struct::IssuerAndSerialNumber;
/// use easyofd_gm::sm2_struct::SignerInfo;
///
/// let signer = SignerInfo::new(
///     IssuerAndSerialNumber::from_serial(1),
///     easyofd_gm::sm2_struct::oids::parse_oid(easyofd_gm::sm2_struct::oids::SM3),
///     easyofd_gm::sm2_struct::oids::parse_oid(easyofd_gm::sm2_struct::oids::SM2_SIGN),
///     vec![0x01],
/// );
/// let ci = ContentInfo::new(
///     easyofd_gm::sm2_struct::oids::parse_oid(easyofd_gm::sm2_struct::oids::SM3),
///     vec![0x30, 0x00],
/// );
/// let sd = SignedDataBuilder::new()
///     .content_info(ci)
///     .add_signer(signer)
///     .build();
/// assert_eq!(sd.signer_infos.len(), 1);
/// ```
#[derive(Debug, Clone)]
pub struct SignedDataBuilder {
    digest_algorithms: Vec<Vec<u32>>,
    content_info: Option<ContentInfo>,
    certificates: Vec<Vec<u8>>,
    crls: Vec<Vec<u8>>,
    signer_infos: Vec<SignerInfo>,
}

impl SignedDataBuilder {
    /// 创建新的构建器（默认 SM3 摘要算法）。
    #[must_use]
    pub fn new() -> Self {
        Self {
            digest_algorithms: vec![parse_oid(SM3)],
            content_info: None,
            certificates: Vec::new(),
            crls: Vec::new(),
            signer_infos: Vec::new(),
        }
    }

    /// 设置摘要算法 OID。
    #[must_use]
    pub fn digest_algorithm(mut self, oid: Vec<u32>) -> Self {
        self.digest_algorithms = vec![oid];
        self
    }

    /// 添加摘要算法 OID。
    #[must_use]
    pub fn add_digest_algorithm(mut self, oid: Vec<u32>) -> Self {
        self.digest_algorithms.push(oid);
        self
    }

    /// 设置内容信息。
    #[must_use]
    pub fn content_info(mut self, ci: ContentInfo) -> Self {
        self.content_info = Some(ci);
        self
    }

    /// 添加证书（DER 字节）。
    #[must_use]
    pub fn add_certificate(mut self, cert_der: Vec<u8>) -> Self {
        self.certificates.push(cert_der);
        self
    }

    /// 添加 CRL（DER 字节）。
    #[must_use]
    pub fn add_crl(mut self, crl_der: Vec<u8>) -> Self {
        self.crls.push(crl_der);
        self
    }

    /// 添加签名者信息。
    #[must_use]
    pub fn add_signer(mut self, signer: SignerInfo) -> Self {
        self.signer_infos.push(signer);
        self
    }

    /// 构建 SignedData。
    ///
    /// # Panics
    ///
    /// 如果未设置 `content_info`，会 panic。
    #[must_use]
    pub fn build(self) -> SignedData {
        let content_info = self
            .content_info
            .expect("SignedDataBuilder: content_info is required");
        SignedData {
            version: 1,
            digest_algorithms: self.digest_algorithms,
            content_info,
            certificates: self.certificates,
            crls: self.crls,
            signer_infos: self.signer_infos,
        }
    }

    /// 构建 SignedData（不 panic，返回 None 如果缺少 content_info）。
    #[must_use]
    pub fn try_build(self) -> Option<SignedData> {
        self.content_info.map(|ci| SignedData {
            version: 1,
            digest_algorithms: self.digest_algorithms,
            content_info: ci,
            certificates: self.certificates,
            crls: self.crls,
            signer_infos: self.signer_infos,
        })
    }
}

impl Default for SignedDataBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sm2_struct::IssuerAndSerialNumber;
    use crate::sm2_struct::oids::{SM2_SIGN, SM3, parse_oid};

    #[test]
    fn test_new_has_sm3_default() {
        let b = SignedDataBuilder::new();
        assert_eq!(b.digest_algorithms.len(), 1);
        assert_eq!(b.digest_algorithms[0], parse_oid(SM3));
    }

    #[test]
    fn test_build_minimal() {
        let ci = ContentInfo::new(parse_oid(SM3), vec![0x30, 0x00]);
        let sd = SignedDataBuilder::new().content_info(ci).build();
        assert_eq!(sd.version, 1);
        assert!(sd.signer_infos.is_empty());
    }

    #[test]
    fn test_build_full() {
        let ci = ContentInfo::new(parse_oid(SM3), vec![0x30, 0x00]);
        let signer = SignerInfo::new(
            IssuerAndSerialNumber::from_serial(42),
            parse_oid(SM3),
            parse_oid(SM2_SIGN),
            vec![0xAB],
        );
        let sd = SignedDataBuilder::new()
            .content_info(ci)
            .add_certificate(vec![0x30, 0x02])
            .add_crl(vec![0x30, 0x01])
            .add_signer(signer)
            .build();
        assert_eq!(sd.certificates.len(), 1);
        assert_eq!(sd.crls.len(), 1);
        assert_eq!(sd.signer_infos.len(), 1);
    }

    #[test]
    fn test_try_build_missing_ci() {
        let result = SignedDataBuilder::new().try_build();
        assert!(result.is_none());
    }

    #[test]
    fn test_try_build_with_ci() {
        let ci = ContentInfo::new(parse_oid(SM3), vec![]);
        let result = SignedDataBuilder::new().content_info(ci).try_build();
        assert!(result.is_some());
    }

    #[test]
    #[should_panic(expected = "content_info is required")]
    fn test_build_panics_without_ci() {
        let _ = SignedDataBuilder::new().build();
    }
}
