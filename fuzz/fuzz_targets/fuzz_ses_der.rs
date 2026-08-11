//! 模糊测试目标：GM/SES DER 解码
//!
//! 覆盖函数：
//! - `easyofd_gm::ses::v1::SESSignature::decode_der`
//! - `easyofd_gm::ses::v1::SESeal::decode_der`
//! - `easyofd_gm::ses::v4::SESSignature::decode_der`
//! - `easyofd_gm::ses::v4::SESeal::decode_der`
//! - `easyofd_gm::ses::v5::SESSignature::decode_der`
//! - `easyofd_gm::ses::v5::SESeal::decode_der`
//! - `easyofd_gm::sm2_struct::ContentInfo::from_der`
//! - `easyofd_gm::sm2_struct::SignedData::from_der`
//! - `easyofd_gm::sm2_struct::Sm2Cipher::from_der`
//! - `easyofd_gm::sm2_struct::IssuerAndSerialNumber::from_der`
//!
//! 目标：任意 DER 字节解码不 panic。
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // ---- SES v1 ----
    let _ = easyofd_gm::ses::v1::SESSignature::decode_der(data);
    let _ = easyofd_gm::ses::v1::SESeal::decode_der(data);

    // ---- SES v4 ----
    let _ = easyofd_gm::ses::v4::SESSignature::decode_der(data);
    let _ = easyofd_gm::ses::v4::SESeal::decode_der(data);

    // ---- SES v5 ----
    let _ = easyofd_gm::ses::v5::SESSignature::decode_der(data);
    let _ = easyofd_gm::ses::v5::SESeal::decode_der(data);

    // ---- sm2_struct ----
    let _ = easyofd_gm::sm2_struct::ContentInfo::from_der(data);
    let _ = easyofd_gm::sm2_struct::SignedData::from_der(data);
    let _ = easyofd_gm::sm2_struct::Sm2Cipher::from_der(data);
    let _ = easyofd_gm::sm2_struct::IssuerAndSerialNumber::from_der(data);
});
