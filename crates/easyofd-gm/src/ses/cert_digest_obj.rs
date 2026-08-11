//! 签章者证书杂凑值对象。
//!
//! 对应 Java: `org.ofdrw.gm.ses.v4.CertDigestObj` / `org.ofdrw.gm.ses.v5.CertDigestObj`
//!
//! ASN.1 结构：
//! ```asn1
//! CertDigestObj ::= SEQUENCE {
//!     type   PrintableString,
//!     value  OCTET STRING
//! }
//! ```
//!
//! 注意：此类型不同于 [`super::v4::CertDigest`]（三字段：cert + digestAlg + digestValue）。
//! `CertDigestObj` 是更简单的二字段结构，用于 `CertDigestList` 中。

use super::{
    DerResult, TAG_OCTET_STRING, encode_octet_string, encode_printable_string, encode_sequence,
    expect_tlv,
};

/// 签章者证书杂凑值对象。
///
/// 对应 Java: `org.ofdrw.gm.ses.v4.CertDigestObj` / `org.ofdrw.gm.ses.v5.CertDigestObj`
///
/// 简单的类型-值对，用于 [`super::CertDigestList`] 中描述证书摘要。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CertDigestObj {
    /// 自定义类型标识。
    pub digest_type: String,
    /// 证书杂凑值。
    pub value: Vec<u8>,
}

impl CertDigestObj {
    /// 创建证书杂凑值对象。
    #[must_use]
    pub fn new(digest_type: impl Into<String>, value: Vec<u8>) -> Self {
        Self {
            digest_type: digest_type.into(),
            value,
        }
    }

    /// DER 编码。
    pub fn encode_der(&self) -> Vec<u8> {
        let mut inner = Vec::new();
        encode_printable_string(&self.digest_type, &mut inner);
        encode_octet_string(&self.value, &mut inner);
        let mut out = Vec::new();
        encode_sequence(&inner, &mut out);
        out
    }

    /// DER 解码。
    pub fn decode_der(der: &[u8]) -> DerResult<Self> {
        let (val, _) = super::decode_sequence(der, 0)?;
        let mut pos = 0;

        // type: PrintableString
        let (tag, type_val, next) = super::decode_tlv(&val, pos)?;
        if tag != super::TAG_PRINTABLE_STRING {
            return Err(super::DerError(
                "期望 CertDigestObj.type 为 PrintableString",
            ));
        }
        let digest_type = String::from_utf8_lossy(&type_val).into_owned();
        pos = next;

        // value: OCTET STRING
        let (value, _) = expect_tlv(&val, pos, TAG_OCTET_STRING)?;

        Ok(Self { digest_type, value })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_decode_roundtrip() {
        let obj = CertDigestObj::new("SM3", vec![0xAB; 32]);
        let der = obj.encode_der();
        let decoded = CertDigestObj::decode_der(&der).unwrap();
        assert_eq!(decoded, obj);
    }

    #[test]
    fn field_values_preserved() {
        let obj = CertDigestObj::new("SHA256", vec![0x01, 0x02, 0x03]);
        let der = obj.encode_der();
        let decoded = CertDigestObj::decode_der(&der).unwrap();
        assert_eq!(decoded.digest_type, "SHA256");
        assert_eq!(decoded.value, vec![0x01, 0x02, 0x03]);
    }

    #[test]
    fn empty_value_roundtrip() {
        let obj = CertDigestObj::new("SM3", Vec::new());
        let der = obj.encode_der();
        let decoded = CertDigestObj::decode_der(&der).unwrap();
        assert_eq!(decoded, obj);
    }
}
