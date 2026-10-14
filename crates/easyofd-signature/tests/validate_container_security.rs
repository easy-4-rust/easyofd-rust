//! 公共签名容器必须真实验签，不能把非空配置当作签名有效。
use easyofd_signature::verify_containers::{
    DigitalValidateContainer, Gbt35275ValidateContainer, SignedDataValidateContainer,
};
fn fixture(name: &str) -> Vec<u8> {
    std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/bc185")
            .join(name),
    )
    .unwrap()
}
#[test]
fn digital_container_rejects_forgery_and_tampering() {
    let container = DigitalValidateContainer::new(fixture("public.sec1"));
    let msg = fixture("message.bin");
    assert!(container.validate(&msg, &fixture("sm2.der")).is_ok());
    assert!(container.validate(&msg, b"garbage signature").is_err());
    assert!(
        container
            .validate(b"tampered", &fixture("sm2.der"))
            .is_err()
    );
}
#[test]
fn gbt_container_rejects_garbage_with_nonempty_certificate_list() {
    let container = Gbt35275ValidateContainer::new(vec![fixture("cert.der")]);
    assert!(container.validate(b"message", b"not SignedData").is_err());
}
#[test]
fn odd_and_unicode_hex_signatures_are_rejected_without_panic() {
    let container = DigitalValidateContainer::new(fixture("public.sec1"));
    for value in [b"0".as_slice(), "汉字".as_bytes()] {
        assert!(container.validate(b"message", value).is_err());
    }
}
fn gbt_fixture(name: &str) -> Vec<u8> {
    std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/gbt35275")
            .join(name),
    )
    .unwrap()
}
#[test]
fn java_gbt_and_cms_verify_and_reject_tampered_message() {
    let container = Gbt35275ValidateContainer::new(vec![fixture("cert.der")]);
    let msg = fixture("message.bin");
    for name in ["direct.der", "pkcs9.der", "cms.der"] {
        let value = gbt_fixture(name);
        container
            .validate(&msg, &value)
            .unwrap_or_else(|e| panic!("{name}: {e}"));
        assert!(container.validate(b"tampered", &value).is_err(), "{name}");
        let mut bad = value;
        let n = bad.len();
        bad[n - 1] ^= 1;
        assert!(container.validate(&msg, &bad).is_err(), "{name}");
        assert!(
            Gbt35275ValidateContainer::new(vec![vec![48, 0]])
                .validate(&msg, &gbt_fixture(name))
                .is_err()
        );
    }
}
#[test]
fn existing_rust_gbt_writers_have_real_verification() {
    use easyofd_gm::pkcs12_tools::Pkcs12Tools;
    use easyofd_signature::{
        ExtendSignatureContainer, Gbt35275DsContainer, Gbt35275Pkcs9DsContainer,
    };
    let secret = sm2::SecretKey::from_slice(
        &Pkcs12Tools::read_private_key_verified(&fixture("rust-public-test.p12"), "public-test")
            .unwrap(),
    )
    .unwrap();
    let cert = fixture("cert.der");
    let msg = fixture("message.bin");
    let verifier = Gbt35275ValidateContainer::new(vec![cert.clone()]);
    let direct = Gbt35275DsContainer::new(secret.clone(), cert.clone())
        .sign(&msg, "")
        .unwrap();
    let attrs = Gbt35275Pkcs9DsContainer::new(secret, cert)
        .sign(&msg, "")
        .unwrap();
    for signature in [direct, attrs] {
        verifier.validate(&msg, &signature).unwrap();
        assert!(verifier.validate(b"tampered", &signature).is_err());
    }
}

#[test]
fn legacy_gb_signature_cannot_be_repackaged_as_cms() {
    use easyofd_gm::{
        pkcs12_tools::Pkcs12Tools,
        sm2_struct::{ContentInfo, oids::parse_oid},
    };
    use easyofd_signature::{ExtendSignatureContainer, Gbt35275Pkcs9DsContainer};
    let secret = sm2::SecretKey::from_slice(
        &Pkcs12Tools::read_private_key_verified(&fixture("rust-public-test.p12"), "public-test")
            .unwrap(),
    )
    .unwrap();
    let msg = fixture("message.bin");
    let cert = fixture("cert.der");
    let original = Gbt35275Pkcs9DsContainer::new(secret, cert.clone())
        .sign(&msg, "")
        .unwrap();
    let mut outer = ContentInfo::from_der(&original).unwrap();
    let mut signed = outer.get_signed_data().unwrap();
    outer.content_type = parse_oid("1.2.840.113549.1.7.2");
    signed.content_info.content_type = parse_oid("1.2.840.113549.1.7.1");
    let mut payload = Vec::new();
    easyofd_gm::ses::encode_octet_string(&msg, &mut payload);
    signed.content_info.content = payload;
    outer.content = signed.to_der().unwrap();
    let verifier = Gbt35275ValidateContainer::new(vec![cert]);
    verifier.validate(&msg, &original).unwrap();
    assert!(verifier.validate(&msg, &outer.to_der().unwrap()).is_err());
}

#[test]
fn duplicate_authenticated_attributes_rejected_even_with_valid_signature() {
    use der::{Decode, Tagged};
    use easyofd_gm::{pkcs12_tools::Pkcs12Tools, sm2_struct::ContentInfo};
    use easyofd_signature::{ExtendSignatureContainer, Gbt35275Pkcs9DsContainer};
    use sm2::dsa::signature::Signer;
    let secret = sm2::SecretKey::from_slice(
        &Pkcs12Tools::read_private_key_verified(&fixture("rust-public-test.p12"), "public-test")
            .unwrap(),
    )
    .unwrap();
    let msg = fixture("message.bin");
    let cert = fixture("cert.der");
    let original = Gbt35275Pkcs9DsContainer::new(secret.clone(), cert.clone())
        .sign(&msg, "")
        .unwrap();
    let mut outer = ContentInfo::from_der(&original).unwrap();
    let mut signed = outer.get_signed_data().unwrap();
    let record = &mut signed.signer_infos[0];
    let attrs = der::asn1::AnyRef::from_der(&record.authenticated_attributes).unwrap();
    assert_eq!(attrs.tag(), der::Tag::Set);
    let mut duplicate = attrs.value().to_vec();
    duplicate.extend_from_slice(attrs.value());
    let mut encoded = vec![0x31];
    easyofd_gm::ses::encode_length(duplicate.len(), &mut encoded);
    encoded.extend_from_slice(&duplicate);
    record.authenticated_attributes = encoded.clone();
    let key = sm2::dsa::SigningKey::new("1234567812345678", &secret).unwrap();
    let signature: sm2::dsa::Signature = key.sign(&encoded);
    record.encrypted_digest = signature.to_bytes().to_vec();
    outer.content = signed.to_der().unwrap();
    assert!(
        Gbt35275ValidateContainer::new(vec![cert])
            .validate(&msg, &outer.to_der().unwrap())
            .is_err()
    );
}
