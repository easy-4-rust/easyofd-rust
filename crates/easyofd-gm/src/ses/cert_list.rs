//! 签章者证书信息列表（CHOICE 类型）。
//!
//! 对应 Java: `org.ofdrw.gm.ses.v4.SES_CertList` / `org.ofdrw.gm.ses.v5.SES_CertList`
//!
//! `SES_CertList` 是一个 CHOICE 类型，支持两种表示：
//! - 完整证书列表（`CertInfoList`，type=1）
//! - 证书摘要列表（`CertDigestList`，type=2）

use super::cert_digest_list::CertDigestList;
use super::cert_info_list::CertInfoList;

/// 签章者证书信息列表。
///
/// 对应 Java: `org.ofdrw.gm.ses.v4.SES_CertList` / `org.ofdrw.gm.ses.v5.SES_CertList`
///
/// CHOICE 类型：
/// - [`CertInfoList`]：完整证书列表
/// - [`CertDigestList`]：证书摘要列表
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SESCertList {
    /// 完整证书列表（type=1）。
    FullCerts(CertInfoList),
    /// 证书摘要列表（type=2）。
    Digests(CertDigestList),
}

impl SESCertList {
    /// 创建完整证书列表。
    #[must_use]
    pub fn full_certs(certs: CertInfoList) -> Self {
        Self::FullCerts(certs)
    }

    /// 创建证书摘要列表。
    #[must_use]
    pub fn digests(digests: CertDigestList) -> Self {
        Self::Digests(digests)
    }

    /// 获取类型标识（1=完整证书，2=摘要）。
    #[must_use]
    pub fn type_id(&self) -> u32 {
        match self {
            Self::FullCerts(_) => 1,
            Self::Digests(_) => 2,
        }
    }

    /// 尝试获取完整证书列表引用。
    #[must_use]
    pub fn as_full_certs(&self) -> Option<&CertInfoList> {
        match self {
            Self::FullCerts(list) => Some(list),
            Self::Digests(_) => None,
        }
    }

    /// 尝试获取证书摘要列表引用。
    #[must_use]
    pub fn as_digests(&self) -> Option<&CertDigestList> {
        match self {
            Self::Digests(list) => Some(list),
            Self::FullCerts(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_certs_variant() {
        let list = CertInfoList::from_certs(vec![vec![0x01, 0x02]]);
        let cert_list = SESCertList::full_certs(list);
        assert_eq!(cert_list.type_id(), 1);
        assert!(cert_list.as_full_certs().is_some());
        assert!(cert_list.as_digests().is_none());
    }

    #[test]
    fn digests_variant() {
        let list = CertDigestList::from_items(vec![]);
        let cert_list = SESCertList::digests(list);
        assert_eq!(cert_list.type_id(), 2);
        assert!(cert_list.as_digests().is_some());
        assert!(cert_list.as_full_certs().is_none());
    }

    #[test]
    fn roundtrip_full_certs() {
        let inner = CertInfoList::from_certs(vec![vec![0x30, 0x03, 0x02, 0x01, 0x01]]);
        let cert_list = SESCertList::full_certs(inner);
        let inner_ref = cert_list.as_full_certs().unwrap();
        let der = inner_ref.encode_der();
        let decoded = CertInfoList::decode_der(&der).unwrap();
        assert_eq!(decoded.certs()[0], vec![0x30, 0x03, 0x02, 0x01, 0x01]);
    }

    #[test]
    fn roundtrip_digests() {
        use super::super::cert_digest_obj::CertDigestObj;
        let inner = CertDigestList::from_items(vec![CertDigestObj::new("SM3", vec![0xAB; 32])]);
        let cert_list = SESCertList::digests(inner);
        let inner_ref = cert_list.as_digests().unwrap();
        let der = inner_ref.encode_der();
        let decoded = CertDigestList::decode_der(&der).unwrap();
        assert_eq!(decoded.len(), 1);
    }
}
