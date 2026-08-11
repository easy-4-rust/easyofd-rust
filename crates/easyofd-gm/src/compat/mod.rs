//! ofdrw Java 类名兼容别名。
//!
//! 将 ofdrw-gm Java 项目中的类名映射到 easyofd-gm 中已有的等价类型。
//! 这些 `pub use` 别名不引入新逻辑，仅用于降低从 Java 迁移时的认知负担。

mod kdf;
mod oids;
mod pkc_generate;
mod pkcs12_tools;
mod ses_cert_list;
mod sm2_cipher;

pub use kdf::*;
pub use oids::*;
pub use pkc_generate::*;
pub use pkcs12_tools::*;
pub use ses_cert_list::*;
pub use sm2_cipher::*;

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
