//! GB/T 35275 签名数据验证。
//!
//! 对应 Java: org.ofdrw.gm.sm2strut.GBT35275Validate
//!
//! 提供 GB/T 35275 SM2 签名数据结构的基本验证功能。
//! Java 版验证 SignedData / ContentInfo / SignerInfo 结构的完整性；
//! Rust 版提供简化验证逻辑。

use crate::sm2_struct::content_info::ContentInfo;
use crate::sm2_struct::signed_data::SignedData;
use crate::sm2_struct::verify_info::VerifyInfo;

/// GB/T 35275 签名数据验证器。
///
/// 对应 Java: org.ofdrw.gm.sm2strut.GBT35275Validate
///
/// 提供对 SignedData、ContentInfo 等结构的基本完整性验证。
pub struct Gbt35275Validate;

impl Gbt35275Validate {
    /// 验证 SignedData 结构的基本完整性。
    ///
    /// 检查：
    /// - 版本号必须为 1
    /// - 至少有一个摘要算法
    /// - 至少有一个签名者信息
    /// - contentInfo 不为空
    ///
    /// 对应 Java: GBT35275Validate#validate(SignedData)
    #[must_use]
    pub fn validate_signed_data(sd: &SignedData) -> VerifyInfo {
        if sd.version != 1 {
            return VerifyInfo::err(format!("版本号不合法: {}", sd.version));
        }
        if sd.digest_algorithms.is_empty() {
            return VerifyInfo::err("摘要算法列表为空".to_string());
        }
        if sd.signer_infos.is_empty() {
            return VerifyInfo::err("签名者信息列表为空".to_string());
        }
        if sd.content_info.content.is_empty() {
            return VerifyInfo::err("ContentInfo 内容为空".to_string());
        }
        VerifyInfo::ok()
    }

    /// 验证 ContentInfo 结构的基本完整性。
    ///
    /// 对应 Java: GBT35275Validate#validate(ContentInfo)
    #[must_use]
    pub fn validate_content_info(ci: &ContentInfo) -> VerifyInfo {
        if ci.content_type.is_empty() {
            return VerifyInfo::err("内容类型 OID 为空".to_string());
        }
        if ci.content.is_empty() {
            return VerifyInfo::err("内容数据为空".to_string());
        }
        VerifyInfo::ok()
    }

    /// 从 DER 字节验证 SignedData。
    ///
    /// 先尝试解码，再验证结构完整性。
    ///
    /// 对应 Java: GBT35275Validate#validate(byte[])
    #[must_use]
    pub fn validate_der(der: &[u8]) -> VerifyInfo {
        match SignedData::from_der(der) {
            Ok(sd) => Self::validate_signed_data(&sd),
            Err(e) => VerifyInfo::err(format!("DER 解码失败: {e}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sm2_struct::IssuerAndSerialNumber;
    use crate::sm2_struct::oids::{SM2_SIGN, SM3, parse_oid};

    fn make_valid_signed_data() -> SignedData {
        let ci = ContentInfo::new(parse_oid(SM3), vec![0x30, 0x00]);
        let si = crate::sm2_struct::SignerInfo::new(
            IssuerAndSerialNumber::from_serial(1),
            parse_oid(SM3),
            parse_oid(SM2_SIGN),
            vec![0x01],
        );
        SignedData::new(vec![parse_oid(SM3)], ci, vec![si])
    }

    #[test]
    fn test_valid_signed_data() {
        let sd = make_valid_signed_data();
        let result = Gbt35275Validate::validate_signed_data(&sd);
        assert!(result.is_ok(), "expected ok, got: {result}");
    }

    #[test]
    fn test_bad_version() {
        let mut sd = make_valid_signed_data();
        sd.version = 99;
        let result = Gbt35275Validate::validate_signed_data(&sd);
        assert!(result.is_err());
        assert!(result.hit.contains("版本号"));
    }

    #[test]
    fn test_empty_digest_algorithms() {
        let mut sd = make_valid_signed_data();
        sd.digest_algorithms.clear();
        let result = Gbt35275Validate::validate_signed_data(&sd);
        assert!(result.is_err());
        assert!(result.hit.contains("摘要算法"));
    }

    #[test]
    fn test_empty_signer_infos() {
        let mut sd = make_valid_signed_data();
        sd.signer_infos.clear();
        let result = Gbt35275Validate::validate_signed_data(&sd);
        assert!(result.is_err());
        assert!(result.hit.contains("签名者"));
    }

    #[test]
    fn test_empty_content() {
        let mut sd = make_valid_signed_data();
        sd.content_info.content.clear();
        let result = Gbt35275Validate::validate_signed_data(&sd);
        assert!(result.is_err());
        assert!(result.hit.contains("ContentInfo"));
    }

    #[test]
    fn test_validate_content_info_ok() {
        let ci = ContentInfo::new(parse_oid(SM3), vec![0x30, 0x00]);
        assert!(Gbt35275Validate::validate_content_info(&ci).is_ok());
    }

    #[test]
    fn test_validate_content_info_empty_type() {
        let ci = ContentInfo::new(vec![], vec![0x30, 0x00]);
        let r = Gbt35275Validate::validate_content_info(&ci);
        assert!(r.is_err());
        assert!(r.hit.contains("OID"));
    }

    #[test]
    fn test_validate_der_ok() {
        let sd = make_valid_signed_data();
        let der = sd.to_der().unwrap();
        assert!(Gbt35275Validate::validate_der(&der).is_ok());
    }

    #[test]
    fn test_validate_der_bad_input() {
        let r = Gbt35275Validate::validate_der(&[0x00, 0x01]);
        assert!(r.is_err());
        assert!(r.hit.contains("解码"));
    }
}
