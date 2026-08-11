//! 厂商自定义扩展数据。
//!
//! 对应 Java: `org.ofdrw.gm.ses.v1.ExtData`
//!
//! ASN.1 结构：
//! ```asn1
//! ExtData ::= SEQUENCE {
//!     extnID      OBJECT IDENTIFIER,
//!     critical    BOOLEAN DEFAULT FALSE,
//!     extnValue   OCTET STRING
//! }
//! ```

use super::{
    DerResult, TAG_BOOLEAN, TAG_OBJECT_IDENTIFIER, TAG_OCTET_STRING, decode_oid, encode_boolean,
    encode_octet_string, encode_oid, encode_sequence, expect_tlv,
};

/// 厂商自定义扩展数据。
///
/// 对应 Java: `org.ofdrw.gm.ses.v1.ExtData` / `org.ofdrw.gm.ses.v5.ExtData`
///
/// 用于在印章或签章结构中携带厂商自定义的扩展信息。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtData {
    /// 扩展字段标识 OID。
    pub extn_id: Vec<u32>,
    /// 是否为关键扩展（默认 `false`）。
    pub critical: bool,
    /// 扩展字段数据值。
    pub extn_value: Vec<u8>,
}

impl ExtData {
    /// 创建扩展数据。
    #[must_use]
    pub fn new(extn_id: Vec<u32>, critical: bool, extn_value: Vec<u8>) -> Self {
        Self {
            extn_id,
            critical,
            extn_value,
        }
    }

    /// 创建非关键扩展数据。
    #[must_use]
    pub fn non_critical(extn_id: Vec<u32>, extn_value: Vec<u8>) -> Self {
        Self::new(extn_id, false, extn_value)
    }

    /// DER 编码。
    pub fn encode_der(&self) -> Vec<u8> {
        let mut inner = Vec::new();
        encode_oid(&self.extn_id, &mut inner);
        if self.critical {
            encode_boolean(true, &mut inner);
        }
        encode_octet_string(&self.extn_value, &mut inner);
        let mut out = Vec::new();
        encode_sequence(&inner, &mut out);
        out
    }

    /// DER 解码。
    pub fn decode_der(der: &[u8]) -> DerResult<Self> {
        let (val, _) = super::decode_sequence(der, 0)?;
        let mut pos = 0;

        // extnID: OBJECT IDENTIFIER
        let (oid_val, next) = expect_tlv(&val, pos, TAG_OBJECT_IDENTIFIER)?;
        let extn_id = decode_oid(&oid_val)?;
        pos = next;

        // critical: BOOLEAN (可选，默认 FALSE)
        let mut critical = false;
        if pos < val.len() && val[pos] == TAG_BOOLEAN {
            let (bool_val, next) = expect_tlv(&val, pos, TAG_BOOLEAN)?;
            critical = !bool_val.is_empty() && bool_val[0] != 0;
            pos = next;
        }

        // extnValue: OCTET STRING
        let (extn_value, _) = expect_tlv(&val, pos, TAG_OCTET_STRING)?;

        Ok(Self {
            extn_id,
            critical,
            extn_value,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sm2_struct::oids::SM3;

    #[test]
    fn encode_decode_roundtrip_non_critical() {
        let ext = ExtData::non_critical(
            super::super::super::sm2_struct::oids::parse_oid(SM3),
            vec![0x01, 0x02, 0x03],
        );
        let der = ext.encode_der();
        let decoded = ExtData::decode_der(&der).unwrap();
        assert_eq!(decoded, ext);
        assert!(!decoded.critical);
    }

    #[test]
    fn encode_decode_roundtrip_critical() {
        let ext = ExtData::new(
            super::super::super::sm2_struct::oids::parse_oid(SM3),
            true,
            vec![0xAB, 0xCD],
        );
        let der = ext.encode_der();
        let decoded = ExtData::decode_der(&der).unwrap();
        assert_eq!(decoded, ext);
        assert!(decoded.critical);
    }

    #[test]
    fn empty_value_roundtrip() {
        let ext = ExtData::non_critical(vec![1, 2, 840, 113_549], Vec::new());
        let der = ext.encode_der();
        let decoded = ExtData::decode_der(&der).unwrap();
        assert_eq!(decoded, ext);
    }
}
