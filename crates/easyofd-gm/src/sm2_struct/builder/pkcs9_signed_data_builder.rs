//! PKCS#9 SignedData 构建器。
//!
//! 对应 Java: org.ofdrw.gm.sm2strut.builder.PKCS9SignedDataBuilder
//!
//! 流式构建符合 PKCS#9 规范的 SignedData。相比 [`SignedDataBuilder`]，
//! 本构建器额外支持 PKCS#9 属性（签名时间、消息摘要等认证属性）。
//!
//! Java 版通过 `addSigner(CertSigHolder)` 批量添加签名者；
//! Rust 版提供等价接口。

use crate::ses::encode_sequence;

use super::cert_sig_holder::CertSigHolder;
use super::signed_data_builder::SignedDataBuilder;
use crate::sm2_struct::content_info::ContentInfo;
use crate::sm2_struct::oids::{SM2_SIGN, SM3, parse_oid};
use crate::sm2_struct::signed_data::SignedData;
use crate::sm2_struct::signer_info::SignerInfo;

/// PKCS#9 SignedData 构建器。
///
/// 对应 Java: org.ofdrw.gm.sm2strut.builder.PKCS9SignedDataBuilder
///
/// 在 [`SignedDataBuilder`] 基础上支持 PKCS#9 认证属性
/// （如签名时间 `signingTime`、消息摘要 `messageDigest`）。
#[derive(Debug, Clone)]
pub struct Pkcs9SignedDataBuilder {
    inner: SignedDataBuilder,
    /// 签名时间（GeneralizedTime 格式，如 "20250101120000Z"）。
    signing_time: Option<String>,
}

impl Pkcs9SignedDataBuilder {
    /// 创建新的 PKCS#9 构建器。
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: SignedDataBuilder::new(),
            signing_time: None,
        }
    }

    /// 设置内容信息。
    #[must_use]
    pub fn content_info(self, ci: ContentInfo) -> Self {
        Self {
            inner: self.inner.content_info(ci),
            ..self
        }
    }

    /// 添加证书。
    #[must_use]
    pub fn add_certificate(self, cert_der: Vec<u8>) -> Self {
        Self {
            inner: self.inner.add_certificate(cert_der),
            ..self
        }
    }

    /// 设置签名时间（GeneralizedTime，如 "20250101120000Z"）。
    #[must_use]
    pub fn signing_time(mut self, time: impl Into<String>) -> Self {
        self.signing_time = Some(time.into());
        self
    }

    /// 从证书签名持有者添加签名者。
    ///
    /// 自动构建 SignerInfo，使用默认 SM3 摘要和 SM2 签名算法。
    /// 如果设置了签名时间，会将其编码为 PKCS#9 认证属性。
    #[must_use]
    pub fn add_signer_from_holder(
        self,
        holder: CertSigHolder,
        issuer_serial: crate::sm2_struct::IssuerAndSerialNumber,
    ) -> Self {
        let mut signer = SignerInfo::new(
            issuer_serial,
            parse_oid(SM3),
            parse_oid(SM2_SIGN),
            holder.signature,
        );
        // 如果设置了签名时间，编码为 PKCS#9 认证属性
        if let Some(ref time) = self.signing_time {
            signer.authenticated_attributes = encode_signing_time_attr(time);
        }
        Self {
            inner: self
                .inner
                .add_certificate(holder.cert_der)
                .add_signer(signer),
            ..self
        }
    }

    /// 构建 SignedData。
    #[must_use]
    pub fn build(self) -> SignedData {
        self.inner.build()
    }

    /// 尝试构建 SignedData（缺少 content_info 时返回 None）。
    #[must_use]
    pub fn try_build(self) -> Option<SignedData> {
        self.inner.try_build()
    }
}

impl Default for Pkcs9SignedDataBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// 编码 PKCS#9 signingTime 认证属性（SET OF Attribute 的 DER 字节）。
///
/// 简化实现：只编码 signingTime 单个属性。
fn encode_signing_time_attr(time: &str) -> Vec<u8> {
    // signingTime OID: 1.2.840.113549.1.9.5
    let oid_arcs: [u32; 7] = [1, 2, 840, 113_549, 1, 9, 5];
    let mut attr_value = Vec::new();
    // AttributeValue: GeneralizedTime
    let mut time_der = Vec::new();
    time_der.push(0x18); // GeneralizedTime tag
    crate::ses::encode_length(time.len(), &mut time_der);
    time_der.extend_from_slice(time.as_bytes());
    // Wrap in SET OF
    attr_value.push(0x31); // SET tag
    crate::ses::encode_length(time_der.len(), &mut attr_value);
    attr_value.extend_from_slice(&time_der);

    let mut attr = Vec::new();
    crate::ses::encode_oid(&oid_arcs, &mut attr);
    attr.extend_from_slice(&attr_value);

    let mut out = Vec::new();
    encode_sequence(&attr, &mut out);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sm2_struct::IssuerAndSerialNumber;
    use crate::sm2_struct::oids::{SM3, parse_oid};

    #[test]
    fn test_new() {
        let b = Pkcs9SignedDataBuilder::new();
        assert!(b.signing_time.is_none());
    }

    #[test]
    fn test_build_minimal() {
        let ci = ContentInfo::new(parse_oid(SM3), vec![0x30, 0x00]);
        let sd = Pkcs9SignedDataBuilder::new().content_info(ci).build();
        assert_eq!(sd.version, 1);
    }

    #[test]
    fn test_add_signer_from_holder() {
        let ci = ContentInfo::new(parse_oid(SM3), vec![0x30, 0x00]);
        let holder = CertSigHolder::new(vec![0x30, 0x02, 0x01, 0x01], vec![0xAA, 0xBB]);
        let iasn = IssuerAndSerialNumber::from_serial(1);
        let sd = Pkcs9SignedDataBuilder::new()
            .content_info(ci)
            .signing_time("20250101120000Z")
            .add_signer_from_holder(holder, iasn)
            .build();
        assert_eq!(sd.certificates.len(), 1);
        assert_eq!(sd.signer_infos.len(), 1);
        // 认证属性应已设置
        assert!(!sd.signer_infos[0].authenticated_attributes.is_empty());
    }

    #[test]
    fn test_try_build_none() {
        assert!(Pkcs9SignedDataBuilder::new().try_build().is_none());
    }

    #[test]
    fn test_try_build_some() {
        let ci = ContentInfo::new(parse_oid(SM3), vec![]);
        assert!(
            Pkcs9SignedDataBuilder::new()
                .content_info(ci)
                .try_build()
                .is_some()
        );
    }
}
