//! Java BC1.85的真实签名及篡改反例。
use easyofd_signature::{
    SesV1ValidateContainer, SesV4ValidateContainer, SesV5ValidateContainer,
    SignedDataValidateContainer,
};
fn fixture(name: &str) -> Vec<u8> {
    let root = std::env::var_os("BC185_FIXTURES").map_or_else(
        || std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/bc185"),
        std::path::PathBuf::from,
    );
    std::fs::read(root.join(name)).unwrap()
}
#[test]
fn ses_all_versions_verify_and_reject_tampering() {
    let message = fixture("message.bin");
    let verifiers: Vec<Box<dyn SignedDataValidateContainer>> = vec![
        Box::new(SesV1ValidateContainer),
        Box::new(SesV4ValidateContainer),
        Box::new(SesV5ValidateContainer),
    ];
    for (version, v) in [1, 4, 5].into_iter().zip(verifiers) {
        let value = fixture(&format!("ses-v{version}.der"));
        v.validate(&message, &value).unwrap();
        let mut bad = message.clone();
        bad[0] ^= 1;
        assert!(
            v.validate(&bad, &value).is_err(),
            "V{version} accepted tampered content"
        );
        let mut bad = value.clone();
        let last = bad.len() - 1;
        bad[last] ^= 1;
        assert!(
            v.validate(&message, &bad).is_err(),
            "V{version} accepted tampered signature"
        );
        assert!(v.validate(&message, b"bad DER").is_err());
    }
}
#[test]
fn raw_sm2_signature_uses_der_and_rejects_wrong_identity() {
    use sm2::dsa::signature::Verifier;
    let key = sm2::dsa::VerifyingKey::from_sec1_bytes("1234567812345678", &fixture("public.sec1"))
        .unwrap();
    let sig = sm2::dsa::Signature::from_der(&fixture("sm2.der")).unwrap();
    key.verify(&fixture("message.bin"), &sig).unwrap();
    let wrong =
        sm2::dsa::VerifyingKey::from_sec1_bytes("wrong-id", &fixture("public.sec1")).unwrap();
    assert!(wrong.verify(&fixture("message.bin"), &sig).is_err());
}

fn replace_field(value: &[u8], index: usize, replacement: &[u8]) -> Vec<u8> {
    use der::{Decode, Encode, Reader};
    let root = der::asn1::AnyRef::from_der(value).unwrap();
    let mut reader = der::SliceReader::new(root.value()).unwrap();
    let mut body = Vec::new();
    let mut i = 0;
    while !reader.is_finished() {
        let field = der::asn1::AnyRef::decode(&mut reader).unwrap();
        body.extend(if i == index {
            replacement.to_vec()
        } else {
            field.to_der().unwrap()
        });
        i += 1;
    }
    let mut out = Vec::new();
    easyofd_gm::ses::encode_sequence(&body, &mut out);
    out
}
fn first_field(value: &[u8]) -> Vec<u8> {
    use der::{Decode, Encode};
    let root = der::asn1::AnyRef::from_der(value).unwrap();
    let mut r = der::SliceReader::new(root.value()).unwrap();
    der::asn1::AnyRef::decode(&mut r).unwrap().to_der().unwrap()
}
#[test]
fn rejects_wrong_certificate_algorithm_and_version_but_accepts_legacy_raw() {
    use der::{Decode, Encode, Reader};
    let message = fixture("message.bin");
    let wrong_cert =
        easyofd_gm::pkc_generate::PkcGenerate::generate_self_signed("CN=Wrong Public Test");
    let certificate = der::asn1::OctetString::new(wrong_cert)
        .unwrap()
        .to_der()
        .unwrap();
    let unknown: der::asn1::ObjectIdentifier = "1.2.3.4".parse().unwrap();
    let unknown = unknown.to_der().unwrap();
    let vs: Vec<Box<dyn SignedDataValidateContainer>> = vec![
        Box::new(SesV1ValidateContainer),
        Box::new(SesV4ValidateContainer),
        Box::new(SesV5ValidateContainer),
    ];
    for (version, v) in [1, 4, 5].into_iter().zip(vs) {
        let value = fixture(&format!("ses-v{version}.der"));
        let wrong = if version == 1 {
            replace_field(
                &value,
                0,
                &replace_field(&first_field(&value), 5, &certificate),
            )
        } else {
            replace_field(&value, 1, &certificate)
        };
        assert!(
            v.validate(&message, &wrong).is_err(),
            "V{version} accepted wrong certificate"
        );
        let bad_alg = if version == 1 {
            replace_field(&value, 0, &replace_field(&first_field(&value), 6, &unknown))
        } else {
            replace_field(&value, 2, &unknown)
        };
        assert!(v.validate(&message, &bad_alg).is_err());
        let wrong_version = replace_field(
            &value,
            0,
            &replace_field(&first_field(&value), 0, &9u64.to_der().unwrap()),
        );
        assert!(v.validate(&message, &wrong_version).is_err());
        let root = der::asn1::AnyRef::from_der(&value).unwrap();
        let mut reader = der::SliceReader::new(root.value()).unwrap();
        let mut fields = Vec::new();
        while !reader.is_finished() {
            fields.push(der::asn1::AnyRef::decode(&mut reader).unwrap());
        }
        let i = if version == 1 { 1 } else { 3 };
        let signature = sm2::dsa::Signature::from_der(&fields[i].value()[1..]).unwrap();
        let raw = der::asn1::BitString::new(0, signature.to_bytes().to_vec())
            .unwrap()
            .to_der()
            .unwrap();
        v.validate(&message, &replace_field(&value, i, &raw))
            .unwrap();
    }
}
