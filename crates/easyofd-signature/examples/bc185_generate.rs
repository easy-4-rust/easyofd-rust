//! 通过真实Rust容器生成Java反向验签产物，使用公开上游测试身份。
use easyofd_gm::pkcs12_tools::Pkcs12Tools;
#[path = "common/public_test_pfx.rs"]
mod public_test_pfx;
use easyofd_signature::{ExtendSignatureContainer, SesV1Container, SesV4Container, SesV5Container};
use sm2::dsa::signature::Signer;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = std::env::args()
        .nth(1)
        .ok_or("usage: bc185_generate OUTPUT")?;
    let out = std::path::Path::new(&out);
    std::fs::create_dir_all(out)?;
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/bc185");
    let read = |name: &str| std::fs::read(root.join(name));
    let secret = Pkcs12Tools::read_private_key(&read("public-test.p12")?, "public-test")
        .ok_or("PFX decode failed")?;
    let key = sm2::SecretKey::from_slice(&secret)?;
    let cert = read("cert.der")?;
    let message = read("message.bin")?;
    for name in ["cert.der", "message.bin", "public.sec1"] {
        std::fs::write(out.join(name), read(name)?)?;
    }
    std::fs::write(
        out.join("public-test.p12"),
        public_test_pfx::encode(&secret, &cert, "public-test"),
    )?;
    let signer = sm2::dsa::SigningKey::new("1234567812345678", &key)?;
    let sig: sm2::dsa::Signature = signer.sign(&message);
    std::fs::write(out.join("sm2.der"), sig.to_der().as_bytes())?;
    let v1 = SesV1Container::new(key.clone(), read("seal-v1.der")?, cert.clone())
        .sign(&message, "interop")?;
    std::fs::write(out.join("ses-v1.der"), v1)?;
    let v4 = SesV4Container::new(key.clone(), read("seal-v4.der")?, cert.clone())
        .sign(&message, "interop")?;
    std::fs::write(out.join("ses-v4.der"), v4)?;
    let v5 = SesV5Container::new(key, read("seal-v5.der")?, cert).sign(&message, "interop")?;
    std::fs::write(out.join("ses-v5.der"), v5)?;
    Ok(())
}
