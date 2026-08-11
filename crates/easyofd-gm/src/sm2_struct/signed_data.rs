//! SignedData 结构。
//!
//! 对应 Java: org.ofdrw.gm.sm2strut.SignedData

use crate::ses::{DerResult, TAG_INTEGER, decode_sequence, expect_tlv};
use crate::ses::{encode_integer, encode_oid, encode_sequence};

use super::content_info::ContentInfo;
use super::signer_info::SignerInfo;

/// 签名数据（GB/T 35275 SignedData，PKCS#7 风格）。
///
/// 对应 Java: ofdrw SignedData。
/// DER 布局：
/// ```asn1
/// SignedData ::= SEQUENCE {
///     version            INTEGER,
///     digestAlgorithms   SET OF AlgorithmIdentifier,
///     contentInfo        ContentInfo,
///     certificates       [0] IMPLICIT SET OF Certificate OPTIONAL,
///     crls               [1] IMPLICIT SET OF CertificateList OPTIONAL,
///     signerInfos        SET OF SignerInfo
/// }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct SignedData {
    /// 版本（默认 1）。
    pub version: u64,
    /// 摘要算法 OID 列表。
    pub digest_algorithms: Vec<Vec<u32>>,
    /// 内容信息。
    pub content_info: ContentInfo,
    /// 证书集合（[0] IMPLICIT，DER 字节列表）。
    pub certificates: Vec<Vec<u8>>,
    /// CRL 集合（[1] IMPLICIT，DER 字节列表）。
    pub crls: Vec<Vec<u8>>,
    /// 签名者信息列表。
    pub signer_infos: Vec<SignerInfo>,
}

impl SignedData {
    /// 创建新的签名数据。
    #[must_use]
    pub fn new(
        digest_algorithms: Vec<Vec<u32>>,
        content_info: ContentInfo,
        signer_infos: Vec<SignerInfo>,
    ) -> Self {
        Self {
            version: 1,
            digest_algorithms,
            content_info,
            certificates: Vec::new(),
            crls: Vec::new(),
            signer_infos,
        }
    }

    /// 编码为 DER 字节。
    ///
    /// # 错误
    ///
    /// 子结构 DER 编码失败时返回错误。
    pub fn to_der(&self) -> DerResult<Vec<u8>> {
        let mut inner = Vec::new();
        encode_integer(self.version, &mut inner);
        // digestAlgorithms: SET OF AlgorithmIdentifier（OID）
        let mut alg_set = Vec::new();
        for alg in &self.digest_algorithms {
            let mut alg_der = Vec::new();
            encode_oid(alg, &mut alg_der);
            crate::ses::encode_sequence(&alg_der, &mut alg_set);
        }
        inner.push(0x31); // SET tag
        crate::ses::encode_length(alg_set.len(), &mut inner);
        inner.extend_from_slice(&alg_set);
        // contentInfo
        inner.extend_from_slice(&self.content_info.to_der()?);
        // certificates [0] IMPLICIT SET
        if !self.certificates.is_empty() {
            let mut cert_set = Vec::new();
            for cert in &self.certificates {
                cert_set.extend_from_slice(cert);
            }
            crate::ses::encode_context_explicit(0, &cert_set, &mut inner);
        }
        // crls [1] IMPLICIT SET
        if !self.crls.is_empty() {
            let mut crl_set = Vec::new();
            for crl in &self.crls {
                crl_set.extend_from_slice(crl);
            }
            crate::ses::encode_context_explicit(1, &crl_set, &mut inner);
        }
        // signerInfos: SET OF SignerInfo
        let mut si_set = Vec::new();
        for si in &self.signer_infos {
            si_set.extend_from_slice(&si.to_der()?);
        }
        inner.push(0x31); // SET tag
        crate::ses::encode_length(si_set.len(), &mut inner);
        inner.extend_from_slice(&si_set);
        let mut out = Vec::new();
        encode_sequence(&inner, &mut out);
        Ok(out)
    }

    /// 从 DER 字节解码。
    ///
    /// # 错误
    ///
    /// 输入不是合法的 SignedData DER 序列时返回错误。
    pub fn from_der(der: &[u8]) -> DerResult<Self> {
        let (seq, _) = decode_sequence(der, 0)?;
        let mut pos = 0;

        let (version_val, next) = expect_tlv(&seq, pos, TAG_INTEGER)?;
        let version = crate::ses::decode_uint(&version_val);
        pos = next;

        // digestAlgorithms: SET OF AlgorithmIdentifier
        let mut digest_algorithms = Vec::new();
        if seq.get(pos).copied() == Some(0x31) {
            let (_tag, val, next) = crate::ses::decode_tlv(&seq, pos)?;
            let mut alg_pos = 0;
            while alg_pos < val.len() {
                let (alg_seq, after) = crate::ses::expect_tlv(&val, alg_pos, 0x30)?;
                let (oid_val, _) = expect_tlv(&alg_seq, 0, 0x06)?;
                digest_algorithms.push(crate::ses::decode_oid(&oid_val)?);
                alg_pos = after;
            }
            pos = next;
        }

        // contentInfo
        let ci_der_end = {
            let (_tag, _val, next) = crate::ses::decode_tlv(&seq, pos)?;
            next
        };
        let content_info = ContentInfo::from_der(&seq[pos..ci_der_end])?;
        pos = ci_der_end;

        // certificates [0] IMPLICIT
        let mut certificates = Vec::new();
        if seq.get(pos).copied() == Some(0xA0) {
            let (_tag, val, next) = crate::ses::decode_tlv(&seq, pos)?;
            let mut c_pos = 0;
            while c_pos < val.len() {
                let (_t, _v, after) = crate::ses::decode_tlv(&val, c_pos)?;
                certificates.push(val[c_pos..after].to_vec());
                c_pos = after;
            }
            pos = next;
        }

        // crls [1] IMPLICIT
        let mut crls = Vec::new();
        if seq.get(pos).copied() == Some(0xA1) {
            let (_tag, val, next) = crate::ses::decode_tlv(&seq, pos)?;
            let mut c_pos = 0;
            while c_pos < val.len() {
                let (_t, _v, after) = crate::ses::decode_tlv(&val, c_pos)?;
                crls.push(val[c_pos..after].to_vec());
                c_pos = after;
            }
            pos = next;
        }

        // signerInfos: SET OF SignerInfo
        let mut signer_infos = Vec::new();
        if seq.get(pos).copied() == Some(0x31) {
            let (_tag, val, _next) = crate::ses::decode_tlv(&seq, pos)?;
            let mut si_pos = 0;
            while si_pos < val.len() {
                let (_t, _v, after) = crate::ses::decode_tlv(&val, si_pos)?;
                signer_infos.push(SignerInfo::from_der(&val[si_pos..after])?);
                si_pos = after;
            }
        }

        Ok(Self {
            version,
            digest_algorithms,
            content_info,
            certificates,
            crls,
            signer_infos,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sm2_struct::issuer_and_serial_number::IssuerAndSerialNumber;
    use crate::sm2_struct::oids::{SM2_SIGN, SM3};

    #[test]
    fn test_roundtrip_signed_data() {
        let iasn = IssuerAndSerialNumber::from_serial(42);
        let si = SignerInfo::new(
            iasn,
            crate::sm2_struct::oids::parse_oid(SM3),
            crate::sm2_struct::oids::parse_oid(SM2_SIGN),
            vec![0xDE, 0xAD],
        );
        let empty_ci = ContentInfo::new(crate::sm2_struct::oids::parse_oid(SM3), vec![0x30, 0x00]);
        let sd = SignedData::new(
            vec![crate::sm2_struct::oids::parse_oid(SM3)],
            empty_ci,
            vec![si],
        );
        let der = sd.to_der().unwrap();
        let decoded = SignedData::from_der(&der).unwrap();
        assert_eq!(decoded.version, 1);
        assert_eq!(decoded.digest_algorithms.len(), 1);
        assert_eq!(decoded.signer_infos.len(), 1);
        assert_eq!(decoded.signer_infos[0].encrypted_digest, vec![0xDE, 0xAD]);
    }

    #[test]
    fn test_roundtrip_with_certificates() {
        let iasn = IssuerAndSerialNumber::from_serial(1);
        let si = SignerInfo::new(
            iasn,
            crate::sm2_struct::oids::parse_oid(SM3),
            crate::sm2_struct::oids::parse_oid(SM2_SIGN),
            vec![0x01],
        );
        let mut sd = SignedData::new(
            vec![crate::sm2_struct::oids::parse_oid(SM3)],
            ContentInfo::new(crate::sm2_struct::oids::parse_oid(SM3), vec![0x30, 0x00]),
            vec![si],
        );
        sd.certificates.push(vec![0x30, 0x02, 0x01, 0x01]);
        let der = sd.to_der().unwrap();
        let decoded = SignedData::from_der(&der).unwrap();
        assert_eq!(decoded.certificates.len(), 1);
        assert_eq!(decoded.certificates[0], vec![0x30, 0x02, 0x01, 0x01]);
    }
}
