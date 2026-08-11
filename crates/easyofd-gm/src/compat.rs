//! ofdrw Java 类名兼容别名。
//!
//! 将 ofdrw-gm Java 项目中的类名映射到 easyofd-gm 中已有的等价类型。
//! 这些 `pub type` 别名不引入新逻辑，仅用于降低从 Java 迁移时的认知负担。

// ── SES 电子印章 ──────────────────────────────────────────────────────────

/// 签章者证书信息列表。
///
/// 对应 Java: org.ofdrw.gm.ses.v5.SES_CertList
///
/// 等价于 [`crate::ses::SESCertList`]。
pub use crate::ses::SESCertList as SES_CertList;

// ── SM2 签名数据结构 ──────────────────────────────────────────────────────

/// SM2 加密结果。
///
/// 对应 Java: org.ofdrw.gm.sm2strut.SM2Cipher
///
/// 等价于 [`crate::sm2_struct::Sm2Cipher`]。
pub use crate::sm2_struct::Sm2Cipher as SM2Cipher;

// ── OID 常量 ──────────────────────────────────────────────────────────────

/// GB/T 35275 国密 OID 常量。
///
/// 对应 Java: org.ofdrw.gm.sm2strut.OIDs
///
/// 等价于 [`crate::sm2_struct::oids`] 模块。
pub use crate::sm2_struct::oids as OIDs;

// ── 密钥派生 ──────────────────────────────────────────────────────────────

/// 密钥派生函数（KDF）。
///
/// 对应 Java: org.ofdrw.gm.support.KDF
///
/// 等价于 [`crate::support::Kdf`]。
pub use crate::support::Kdf as KDF;

// ── 证书工具 ──────────────────────────────────────────────────────────────

/// 证书生成工具。
///
/// 对应 Java: org.ofdrw.gm.cert.PKCGenerate
///
/// 等价于 [`crate::cert::PkcGenerate`]。
pub use crate::cert::PkcGenerate as PKCGenerate;

/// PKCS#12 工具。
///
/// 对应 Java: org.ofdrw.gm.cert.PKCS12Tools
///
/// 等价于 [`crate::cert::Pkcs12Tools`]。
pub use crate::cert::Pkcs12Tools as PKCS12Tools;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ses_cert_list_alias() {
        let _ = SES_CertList::full_certs(crate::ses::CertInfoList::from_certs(vec![vec![0x01]]));
    }

    #[test]
    fn sm2_cipher_alias() {
        let c = SM2Cipher::from_u64(1, 2, vec![], vec![]);
        assert!(!c.x_coordinate.is_empty());
    }

    #[test]
    fn oids_alias() {
        assert_eq!(OIDs::SM3, "1.2.156.10197.1.401");
    }

    #[test]
    fn kdf_alias() {
        let key = KDF::generate_key(&[1, 2, 3], 16);
        assert_eq!(key.len(), 16);
    }

    #[test]
    fn pkc_generate_alias() {
        let cert = PKCGenerate::generate_self_signed("CN=Test");
        assert!(!cert.is_empty());
    }

    #[test]
    fn pkcs12_tools_alias() {
        assert!(PKCS12Tools::read_private_key(&[0; 100], "pw").is_none());
    }
}
