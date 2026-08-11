//! 签章者证书列表。
//!
//! 对应 Java: `org.ofdrw.gm.ses.v4.CertInfoList` / `org.ofdrw.gm.ses.v5.CertInfoList`
//!
//! ASN.1 结构：
//! ```asn1
//! CertInfoList ::= SEQUENCE OF OCTET STRING
//! ```
//!
//! 每个元素为一个 DER 编码的 X.509 证书。

use super::{DerResult, TAG_OCTET_STRING, encode_octet_string, encode_sequence, expect_tlv};

/// 签章者证书列表。
///
/// 对应 Java: `org.ofdrw.gm.ses.v4.CertInfoList` / `org.ofdrw.gm.ses.v5.CertInfoList`
///
/// 每个元素为 DER 编码的 X.509 证书字节。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CertInfoList {
    /// 证书列表（每个元素为 DER 编码的 X.509 证书）。
    certs: Vec<Vec<u8>>,
}

impl CertInfoList {
    /// 创建空列表。
    #[must_use]
    pub fn new() -> Self {
        Self { certs: Vec::new() }
    }

    /// 从已有列表创建。
    #[must_use]
    pub fn from_certs(certs: Vec<Vec<u8>>) -> Self {
        Self { certs }
    }

    /// 追加证书。
    pub fn push(&mut self, cert: Vec<u8>) {
        self.certs.push(cert);
    }

    /// 获取证书列表引用。
    #[must_use]
    pub fn certs(&self) -> &[Vec<u8>] {
        &self.certs
    }

    /// 是否为空。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.certs.is_empty()
    }

    /// 证书数量。
    #[must_use]
    pub fn len(&self) -> usize {
        self.certs.len()
    }

    /// DER 编码。
    pub fn encode_der(&self) -> Vec<u8> {
        let mut inner = Vec::new();
        for cert in &self.certs {
            encode_octet_string(cert, &mut inner);
        }
        let mut out = Vec::new();
        encode_sequence(&inner, &mut out);
        out
    }

    /// DER 解码。
    pub fn decode_der(der: &[u8]) -> DerResult<Self> {
        let (val, _) = super::decode_sequence(der, 0)?;
        let mut certs = Vec::new();
        let mut pos = 0;
        while pos < val.len() {
            let (cert_val, next) = expect_tlv(&val, pos, TAG_OCTET_STRING)?;
            certs.push(cert_val);
            pos = next;
        }
        Ok(Self { certs })
    }
}

impl From<Vec<Vec<u8>>> for CertInfoList {
    fn from(certs: Vec<Vec<u8>>) -> Self {
        Self::from_certs(certs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_roundtrip() {
        let list = CertInfoList::new();
        assert!(list.is_empty());
        let der = list.encode_der();
        let decoded = CertInfoList::decode_der(&der).unwrap();
        assert!(decoded.is_empty());
    }

    #[test]
    fn single_cert_roundtrip() {
        let mut list = CertInfoList::new();
        list.push(vec![0x30, 0x03, 0x02, 0x01, 0x01]);
        let der = list.encode_der();
        let decoded = CertInfoList::decode_der(&der).unwrap();
        assert_eq!(decoded.len(), 1);
        assert_eq!(decoded.certs()[0], vec![0x30, 0x03, 0x02, 0x01, 0x01]);
    }

    #[test]
    fn multiple_certs_roundtrip() {
        let list = CertInfoList::from_certs(vec![
            vec![0x30, 0x03, 0x02, 0x01, 0x01],
            vec![0x30, 0x05, 0x02, 0x01, 0x02, 0x02, 0x01],
        ]);
        let der = list.encode_der();
        let decoded = CertInfoList::decode_der(&der).unwrap();
        assert_eq!(decoded.len(), 2);
    }
}
