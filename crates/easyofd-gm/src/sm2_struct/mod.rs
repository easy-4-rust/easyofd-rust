//! GB/T 35275 SM2 签名数据结构（ASN.1 DER）。
//!
//! 对应 Java 版 [`org.ofdrw.gm.sm2strut`](https://github.com/ofdrw/ofdrw) 包。
//! 这些结构用于 GB/T 38540-2020 数字签名的 SignedInfo 与签名值编码。

pub mod content_info;
pub mod issuer_and_serial_number;
pub mod oids;
pub mod signed_data;
pub mod signer_info;
pub mod sm2_cipher;
pub mod verify_info;

/// Builder 和验证辅助类型（对应 Java: org.ofdrw.gm.sm2strut.builder）。
pub mod builder {
    pub mod cert_sig_holder;
    pub mod gbt35275_validate;
    pub mod pkcs9_signed_data_builder;
    pub mod signed_data_builder;

    pub use cert_sig_holder::CertSigHolder;
    pub use gbt35275_validate::Gbt35275Validate;
    pub use pkcs9_signed_data_builder::Pkcs9SignedDataBuilder;
    pub use signed_data_builder::SignedDataBuilder;
}

pub use content_info::ContentInfo;
pub use issuer_and_serial_number::IssuerAndSerialNumber;
pub use oids::{format_oid, parse_oid};
pub use signed_data::SignedData;
pub use signer_info::SignerInfo;
pub use sm2_cipher::Sm2Cipher;
pub use verify_info::VerifyInfo;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_exports() {
        // 模块公开类型可实例化。
        let _ = oids::parse_oid(oids::SM2_SIGN);
        let _ = IssuerAndSerialNumber::from_serial(1);
        let _ = Sm2Cipher::from_u64(1, 2, vec![], vec![]);
    }
}
