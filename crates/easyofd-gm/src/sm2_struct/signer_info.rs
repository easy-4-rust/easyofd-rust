//! SignerInfo 结构。
//!
//! 对应 Java: org.ofdrw.gm.sm2strut.SignerInfo

use crate::ses::{DerResult, TAG_INTEGER, TAG_OCTET_STRING, decode_sequence, expect_tlv};
use crate::ses::{encode_integer, encode_octet_string, encode_oid, encode_sequence};

use super::issuer_and_serial_number::IssuerAndSerialNumber;

/// 签名者信息（PKCS#7 SignerInfo）。
///
/// 对应 Java: ofdrw SignerInfo。
/// DER 布局：
/// ```asn1
/// SignerInfo ::= SEQUENCE {
///     version                      INTEGER,
///     sid                          IssuerAndSerialNumber,
///     digestAlgorithm              AlgorithmIdentifier,
///     authenticatedAttributes      [0] IMPLICIT SET OPTIONAL,
///     digestEncryptionAlgorithm    AlgorithmIdentifier,
///     encryptedDigest              OCTET STRING,
///     unauthenticatedAttributes    [1] IMPLICIT SET OPTIONAL
/// }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct SignerInfo {
    /// 版本（默认 1）。
    pub version: u64,
    /// 签发者与序列号。
    pub issuer_serial: IssuerAndSerialNumber,
    /// 摘要算法 OID 弧段（如 SM3）。
    pub digest_algorithm: Vec<u32>,
    /// 认证属性（[0] IMPLICIT SET 的原始 DER 字节，可为空）。
    pub authenticated_attributes: Vec<u8>,
    /// 摘要加密算法 OID 弧段（如 SM2 签名）。
    pub digest_encryption_algorithm: Vec<u32>,
    /// 加密后的摘要（签名值）。
    pub encrypted_digest: Vec<u8>,
    /// 非认证属性（[1] IMPLICIT SET 的原始 DER 字节，可为空）。
    pub unauthenticated_attributes: Vec<u8>,
}

impl SignerInfo {
    /// 创建新的签名者信息。
    #[must_use]
    pub fn new(
        issuer_serial: IssuerAndSerialNumber,
        digest_algorithm: Vec<u32>,
        digest_encryption_algorithm: Vec<u32>,
        encrypted_digest: Vec<u8>,
    ) -> Self {
        Self {
            version: 1,
            issuer_serial,
            digest_algorithm,
            authenticated_attributes: Vec::new(),
            digest_encryption_algorithm,
            encrypted_digest,
            unauthenticated_attributes: Vec::new(),
        }
    }

    /// 编码为 DER 字节。
    ///
    /// # 错误
    ///
    /// DER 编码不失败，此签名保留以对齐 ofdrw API。
    pub fn to_der(&self) -> DerResult<Vec<u8>> {
        let mut inner = Vec::new();
        encode_integer(self.version, &mut inner);
        inner.extend_from_slice(&self.issuer_serial.to_der()?);
        encode_oid(&self.digest_algorithm, &mut inner);
        if !self.authenticated_attributes.is_empty() {
            crate::ses::encode_context_explicit(0, &self.authenticated_attributes, &mut inner);
        }
        encode_oid(&self.digest_encryption_algorithm, &mut inner);
        encode_octet_string(&self.encrypted_digest, &mut inner);
        if !self.unauthenticated_attributes.is_empty() {
            crate::ses::encode_context_explicit(1, &self.unauthenticated_attributes, &mut inner);
        }
        let mut out = Vec::new();
        encode_sequence(&inner, &mut out);
        Ok(out)
    }

    /// 从 DER 字节解码。
    ///
    /// # 错误
    ///
    /// 输入不是合法的 SignerInfo DER 序列时返回错误。
    pub fn from_der(der: &[u8]) -> DerResult<Self> {
        let (seq, _) = decode_sequence(der, 0)?;
        let mut pos = 0;
        let (version_val, next) = expect_tlv(&seq, pos, TAG_INTEGER)?;
        let version = crate::ses::decode_uint(&version_val);
        pos = next;

        // sid: 内嵌 SEQUENCE
        let (iasn_seq, next) = crate::ses::expect_tlv(&seq, pos, 0x30)?;
        let mut full_iasn = vec![0x30];
        let mut len_buf = Vec::new();
        crate::ses::encode_length(iasn_seq.len(), &mut len_buf);
        full_iasn.extend_from_slice(&len_buf);
        full_iasn.extend_from_slice(&iasn_seq);
        let issuer_serial = IssuerAndSerialNumber::from_der(&full_iasn)?;
        pos = next;

        let (digest_alg_val, next) = expect_tlv(&seq, pos, 0x06)?;
        let digest_algorithm = crate::ses::decode_oid(&digest_alg_val)?;
        pos = next;

        let mut authenticated_attributes = Vec::new();
        if seq.get(pos).copied() == Some(0xA0) {
            let (_tag, val, next) = crate::ses::decode_tlv(&seq, pos)?;
            authenticated_attributes = val;
            pos = next;
        }

        let (enc_alg_val, next) = expect_tlv(&seq, pos, 0x06)?;
        let digest_encryption_algorithm = crate::ses::decode_oid(&enc_alg_val)?;
        pos = next;

        let (digest_val, next) = expect_tlv(&seq, pos, TAG_OCTET_STRING)?;
        let encrypted_digest = digest_val;
        pos = next;

        let mut unauthenticated_attributes = Vec::new();
        if seq.get(pos).copied() == Some(0xA1) {
            let (_tag, val, _next) = crate::ses::decode_tlv(&seq, pos)?;
            unauthenticated_attributes = val;
        }

        Ok(Self {
            version,
            issuer_serial,
            digest_algorithm,
            authenticated_attributes,
            digest_encryption_algorithm,
            encrypted_digest,
            unauthenticated_attributes,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sm2_struct::oids::{SM2_SIGN, SM3};

    #[test]
    fn test_roundtrip() {
        let iasn = IssuerAndSerialNumber::from_serial(0x1A2B);
        let info = SignerInfo::new(
            iasn,
            crate::sm2_struct::oids::parse_oid(SM3),
            crate::sm2_struct::oids::parse_oid(SM2_SIGN),
            vec![0x01, 0x02, 0x03],
        );
        let der = info.to_der().unwrap();
        let decoded = SignerInfo::from_der(&der).unwrap();
        assert_eq!(decoded.version, 1);
        assert_eq!(decoded.encrypted_digest, vec![0x01, 0x02, 0x03]);
        assert_eq!(decoded.digest_algorithm, info.digest_algorithm);
        assert_eq!(
            decoded.issuer_serial.cert_serial_number,
            info.issuer_serial.cert_serial_number
        );
    }

    #[test]
    fn test_with_authenticated_attributes() {
        let iasn = IssuerAndSerialNumber::from_serial(7);
        let mut info = SignerInfo::new(
            iasn,
            crate::sm2_struct::oids::parse_oid(SM3),
            crate::sm2_struct::oids::parse_oid(SM2_SIGN),
            vec![0xAA],
        );
        info.authenticated_attributes = vec![0x31, 0x02, 0x01, 0x01];
        let der = info.to_der().unwrap();
        let decoded = SignerInfo::from_der(&der).unwrap();
        assert_eq!(
            decoded.authenticated_attributes,
            vec![0x31, 0x02, 0x01, 0x01]
        );
    }
}
