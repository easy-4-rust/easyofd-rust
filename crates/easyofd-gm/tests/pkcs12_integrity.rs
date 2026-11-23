//! 外部密码学输入的生产安全回归。
use easyofd_gm::pkcs12_tools::Pkcs12Tools;
const PFX: &[u8] = include_bytes!("../../../tests/fixtures/bc185/rust-public-test.p12");

#[test]
fn unencrypted_keybag_with_mac_rejects_wrong_password() {
    assert_eq!(
        Pkcs12Tools::read_private_key(PFX, "public-test")
            .unwrap()
            .len(),
        32
    );
    assert!(Pkcs12Tools::read_private_key(PFX, "wrong-password").is_none());
}
#[test]
fn tampered_mac_cannot_release_private_key() {
    let mut pfx = PFX.to_vec();
    // 盐串位于MacData中；盐变更必须使完整性校验失败，KeyBag仍可解析。
    let salt = b"public-test-salt!";
    let offset = pfx.windows(salt.len()).position(|w| w == salt).unwrap();
    pfx[offset] ^= 1;
    assert!(Pkcs12Tools::read_private_key(&pfx, "public-test").is_none());
}

fn fixture(name: &str) -> Vec<u8> {
    std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/pkcs12_integrity")
            .join(name),
    )
    .unwrap()
}
#[test]
fn strict_import_requires_mac_and_accepts_all_rfc_hash_variants() {
    for name in [
        "sha1",
        "sha224",
        "sha256",
        "sha384",
        "sha512",
        "sha512_224",
        "sha512_256",
    ] {
        let data = fixture(&format!("{name}.p12"));
        assert_eq!(
            Pkcs12Tools::read_private_key_verified(&data, "public-test")
                .unwrap()
                .len(),
            32,
            "{name}"
        );
        assert!(
            Pkcs12Tools::read_private_key_verified(&data, "wrong").is_none(),
            "{name}"
        );
    }
    let no_mac = fixture("no_mac.p12");
    assert!(Pkcs12Tools::read_private_key_verified(&no_mac, "public-test").is_none());
    assert!(Pkcs12Tools::read_private_key(&no_mac, "public-test").is_some());
}
#[test]
fn invalid_mac_iterations_are_rejected_within_budget() {
    let start = std::time::Instant::now();
    for file in ["negative.p12", "zero.p12", "over_budget.p12"] {
        assert!(
            Pkcs12Tools::read_private_key_verified(&fixture(file), "public-test").is_none(),
            "{file}"
        );
    }
    assert!(start.elapsed().as_secs() < 2, "恶意迭代次数必须快速拒绝");
}
#[test]
fn mutation_of_authenticated_content_is_rejected() {
    let mut data = fixture("sha256.p12");
    let label = b"public-test-key";
    let offset = data.windows(label.len()).position(|w| w == label).unwrap();
    data[offset] ^= 1;
    assert!(Pkcs12Tools::read_private_key_verified(&data, "public-test").is_none());
}
#[test]
fn authenticated_non_sm2_or_invalid_scalar_is_rejected() {
    for file in ["wrong_curve.p12", "zero_scalar.p12", "invalid_scalar.p12"] {
        assert!(
            Pkcs12Tools::read_private_key_verified(&fixture(file), "public-test").is_none(),
            "{file}"
        );
    }
}
#[test]
fn authenticated_malicious_key_lengths_reject_without_allocation() {
    let start = std::time::Instant::now();
    for name in ["negative", "zero", "wrong_size", "huge"] {
        assert!(
            Pkcs12Tools::read_private_key_verified(
                &fixture(&format!("kdf_{name}.p12")),
                "public-test"
            )
            .is_none(),
            "{name}"
        );
    }
    assert!(start.elapsed().as_secs() < 2);
}
