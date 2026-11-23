//! BC1.85公开测试身份的PKCS#12兼容性。
use easyofd_gm::pkcs12_tools::Pkcs12Tools;
#[test]
fn reads_correct_password_rejects_wrong_password() {
    let root = std::env::var_os("BC185_FIXTURES").map_or_else(
        || std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/bc185"),
        std::path::PathBuf::from,
    );
    let data = std::fs::read(root.join("public-test.p12")).unwrap();
    let key = Pkcs12Tools::read_private_key(&data, "public-test").expect("BC1.85 PFX private key");
    assert_eq!(key.len(), 32);
    assert!(Pkcs12Tools::read_private_key(&data, "wrong-password").is_none());
    assert!(Pkcs12Tools::read_private_key(b"bad DER", "public-test").is_none());
}
