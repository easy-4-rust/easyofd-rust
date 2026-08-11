//! 签章者证书杂凑值列表。
//!
//! 对应 Java: `org.ofdrw.gm.ses.v4.CertDigestList` / `org.ofdrw.gm.ses.v5.CertDigestList`
//!
//! ASN.1 结构：
//! ```asn1
//! CertDigestList ::= SEQUENCE OF CertDigestObj
//! ```

use super::cert_digest_obj::CertDigestObj;
use super::{DerResult, encode_sequence};

/// 签章者证书杂凑值列表。
///
/// 对应 Java: `org.ofdrw.gm.ses.v4.CertDigestList` / `org.ofdrw.gm.ses.v5.CertDigestList`
///
/// 包含零或多个 [`CertDigestObj`]。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CertDigestList {
    /// 杂凑值列表。
    items: Vec<CertDigestObj>,
}

impl CertDigestList {
    /// 创建空列表。
    #[must_use]
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    /// 从已有列表创建。
    #[must_use]
    pub fn from_items(items: Vec<CertDigestObj>) -> Self {
        Self { items }
    }

    /// 追加一个杂凑值对象。
    pub fn push(&mut self, item: CertDigestObj) {
        self.items.push(item);
    }

    /// 获取列表引用。
    #[must_use]
    pub fn items(&self) -> &[CertDigestObj] {
        &self.items
    }

    /// 是否为空。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// 数量。
    #[must_use]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// DER 编码。
    pub fn encode_der(&self) -> Vec<u8> {
        let mut inner = Vec::new();
        for item in &self.items {
            inner.extend_from_slice(&item.encode_der());
        }
        let mut out = Vec::new();
        encode_sequence(&inner, &mut out);
        out
    }

    /// DER 解码。
    pub fn decode_der(der: &[u8]) -> DerResult<Self> {
        let (val, _) = super::decode_sequence(der, 0)?;
        let mut items = Vec::new();
        let mut pos = 0;
        while pos < val.len() {
            let (tag, seq_val, next) = super::decode_tlv(&val, pos)?;
            if tag != super::TAG_SEQUENCE {
                return Err(super::DerError("期望 CertDigestObj 为 SEQUENCE"));
            }
            let mut full = Vec::with_capacity(seq_val.len() + 4);
            encode_sequence(&seq_val, &mut full);
            items.push(CertDigestObj::decode_der(&full)?);
            pos = next;
        }
        Ok(Self { items })
    }
}

impl From<Vec<CertDigestObj>> for CertDigestList {
    fn from(items: Vec<CertDigestObj>) -> Self {
        Self::from_items(items)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_roundtrip() {
        let list = CertDigestList::new();
        assert!(list.is_empty());
        let der = list.encode_der();
        let decoded = CertDigestList::decode_der(&der).unwrap();
        assert!(decoded.is_empty());
    }

    #[test]
    fn single_item_roundtrip() {
        let mut list = CertDigestList::new();
        list.push(CertDigestObj::new("SM3", vec![0xAB; 32]));
        let der = list.encode_der();
        let decoded = CertDigestList::decode_der(&der).unwrap();
        assert_eq!(decoded.len(), 1);
        assert_eq!(decoded.items()[0].digest_type, "SM3");
    }

    #[test]
    fn multiple_items_roundtrip() {
        let list = CertDigestList::from_items(vec![
            CertDigestObj::new("SM3", vec![0xAB; 32]),
            CertDigestObj::new("SHA256", vec![0xCD; 32]),
        ]);
        let der = list.encode_der();
        let decoded = CertDigestList::decode_der(&der).unwrap();
        assert_eq!(decoded.len(), 2);
        assert_eq!(decoded.items()[0].digest_type, "SM3");
        assert_eq!(decoded.items()[1].digest_type, "SHA256");
    }
}
