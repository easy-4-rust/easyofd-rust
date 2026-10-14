//! 外部密码学输入的生产安全回归。
use easyofd_gm::ses::{v1, v4, v5};

#[test]
fn historical_fuzz_length_overflow_is_rejected_without_panic() {
    // 来自GitHub Fuzz crash-2f95a473751accbbfb7f2ffe9e21706e0c81d781。
    let malformed = [
        0x30, 0x0d, 0x06, 0x89, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x00,
    ];
    assert!(v1::SESSignature::decode_der(&malformed).is_err());
    assert!(v1::SESeal::decode_der(&malformed).is_err());
    assert!(v4::SESSignature::decode_der(&malformed).is_err());
    assert!(v4::SESeal::decode_der(&malformed).is_err());
    assert!(v5::SESSignature::decode_der(&malformed).is_err());
    assert!(v5::SESeal::decode_der(&malformed).is_err());
}
