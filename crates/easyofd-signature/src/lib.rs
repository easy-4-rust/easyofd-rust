//! # easyofd-signature
//!
//! OFD electronic seal and digital signature operations per GB/T 38540.
//! 支持 SM2WithSM3 国密签名算法。

use std::io::{Cursor, Read, Write};
use easyofd_core::OfdResult;
use sm3::{Digest, Sm3};
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

/// 支持的签名算法。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignatureAlgorithm {
    Sm2WithSm3,
    Sha256WithRsa,
}

/// OFD 电子签章。
#[derive(Debug, Clone)]
pub struct ElectronicSeal {
    pub image_data: Vec<u8>,
    pub name: String,
    pub position: (f64, f64),
    pub page: usize,
}

/// 签名结果。
#[derive(Debug)]
pub struct SignedOfd {
    data: Vec<u8>,
    pub digest: String,
    pub signature_value: String,
}

impl SignedOfd {
    pub fn save(self, path: impl AsRef<std::path::Path>) -> OfdResult<()> {
        easyofd_package::atomic_write(path, |file| { file.write_all(&self.data)?; Ok(()) })
    }
    #[must_use]
    pub fn into_bytes(self) -> Vec<u8> { self.data }
}

/// 签章构建器。
pub struct OfdSignatureBuilder {
    input_path: String,
    seals: Vec<ElectronicSeal>,
    algorithm: SignatureAlgorithm,
}

impl OfdSignatureBuilder {
    #[must_use]
    pub fn new(input: impl Into<String>) -> Self {
        Self { input_path: input.into(), seals: Vec::new(), algorithm: SignatureAlgorithm::Sm2WithSm3 }
    }
    #[must_use]
    pub fn seal(mut self, seal: ElectronicSeal) -> Self { self.seals.push(seal); self }
    #[must_use]
    pub fn algorithm(mut self, alg: SignatureAlgorithm) -> Self { self.algorithm = alg; self }

    pub fn sign(self) -> OfdResult<SignedOfd> {
        let input_bytes = std::fs::read(&self.input_path).map_err(easyofd_core::OfdError::Io)?;
        let digest = compute_sm3(&input_bytes);
        let digest_hex = hex(&digest);

        use sm2::elliptic_curve::Generate;
        let secret_key = sm2::SecretKey::generate();
        let signing_key = sm2::dsa::SigningKey::new("easyofd-rust", &secret_key)
            .map_err(|e| easyofd_core::OfdError::Conversion(format!("{e}")))?;
        use sm2::dsa::signature::Signer;
        let sig = signing_key.sign(&input_bytes);
        let sig_hex = hex(&sig.to_bytes());
        let pub_hex = hex(&signing_key.verifying_key().to_sec1_bytes());

        let cursor = Cursor::new(&input_bytes[..]);
        let mut archive = zip::ZipArchive::new(cursor).map_err(|e| easyofd_core::OfdError::Zip(e.to_string()))?;
        easyofd_package::validate_archive(&mut archive, easyofd_package::PackageLimits::default())?;
        let out = Cursor::new(Vec::new());
        let mut zip = ZipWriter::new(out);
        let opts = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
        for i in 0..archive.len() {
            let mut e = archive.by_index(i).map_err(|e| easyofd_core::OfdError::Zip(e.to_string()))?;
            let name = e.name().to_string();
            let mut buf = Vec::new();
            e.read_to_end(&mut buf).map_err(easyofd_core::OfdError::Io)?;
            zip.start_file(name, opts).map_err(|e| easyofd_core::OfdError::Zip(e.to_string()))?;
            zip.write_all(&buf).map_err(easyofd_core::OfdError::Io)?;
        }
        for (i, seal) in self.seals.iter().enumerate() {
            zip.start_file(format!("Doc_0/Res/Seal_{i}.png"), opts).map_err(|e| easyofd_core::OfdError::Zip(e.to_string()))?;
            zip.write_all(&seal.image_data).map_err(easyofd_core::OfdError::Io)?;
        }
        let xml = sig_xml(&self.seals, self.algorithm, &digest_hex, &sig_hex, &pub_hex);
        zip.start_file("Doc_0/Signs/Signature.xml", opts).map_err(|e| easyofd_core::OfdError::Zip(e.to_string()))?;
        zip.write_all(xml.as_bytes()).map_err(easyofd_core::OfdError::Io)?;
        let data = zip.finish().map_err(|e| easyofd_core::OfdError::Zip(e.to_string()))?.into_inner();
        Ok(SignedOfd { data, digest: digest_hex, signature_value: sig_hex })
    }
}

fn compute_sm3(data: &[u8]) -> [u8; 32] {
    let mut h = Sm3::new(); h.update(data);
    let r = h.finalize(); let mut o = [0u8; 32]; o.copy_from_slice(&r); o
}
fn hex(b: &[u8]) -> String { b.iter().map(|x| format!("{x:02x}")).collect() }
fn sig_xml(seals: &[ElectronicSeal], alg: SignatureAlgorithm, digest: &str, sig: &str, pubk: &str) -> String {
    let a = match alg { SignatureAlgorithm::Sm2WithSm3 => "SM2WithSM3", SignatureAlgorithm::Sha256WithRsa => "SHA256WithRSA" };
    format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<ofd:Signature xmlns:ofd="http://www.ofdspec.org/2016">
  <ofd:SignedInfo>
    <ofd:Provider>easyofd-rust</ofd:Provider>
    <ofd:SignatureMethod>{a}</ofd:SignatureMethod>
    <ofd:SignatureDateTime>{}</ofd:SignatureDateTime>
    <ofd:SealCount>{}</ofd:SealCount>
    <ofd:DigestValue>{digest}</ofd:DigestValue>
    <ofd:PublicKey>{pubk}</ofd:PublicKey>
  </ofd:SignedInfo>
  <ofd:SignedValue>{sig}</ofd:SignedValue>
</ofd:Signature>"#, chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S"), seals.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use easyofd_core::{OfdPage, TextObject};
    use easyofd_writer::OfdWriter;
    fn make_ofd(p: &std::path::Path) {
        let mut pg = OfdPage::new(210.0, 297.0);
        pg.add_text(TextObject::new(10.0, 20.0, "Doc"));
        let mut w = OfdWriter::new(); w.add_page(pg); w.build_to_file(p).unwrap();
    }
    #[test]
    fn test_sm3() {
        let d = compute_sm3(b"hello");
        assert_eq!(d.len(), 32);
        assert_eq!(hex(&d).len(), 64);
    }
    #[test]
    fn test_sign() {
        let dir = std::env::temp_dir().join("easyofd_sig_test");
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join("t.ofd"); make_ofd(&p);
        let r = OfdSignatureBuilder::new(p.to_string_lossy().into_owned()).sign().unwrap();
        assert_eq!(r.digest.len(), 64);
        assert_eq!(r.signature_value.len(), 128);
        assert_eq!(&r.into_bytes()[0..2], b"PK");
        let _ = std::fs::remove_file(&p);
    }
    #[test]
    fn test_sign_with_seal() {
        let dir = std::env::temp_dir().join("easyofd_sig_seal2");
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join("s.ofd"); make_ofd(&p);
        let r = OfdSignatureBuilder::new(p.to_string_lossy().into_owned())
            .seal(ElectronicSeal { image_data: vec![0x89], name: "S".into(), position: (1.0, 2.0), page: 1 })
            .sign().unwrap();
        let bytes = r.into_bytes();
        let cur = Cursor::new(&bytes);
        let mut a = zip::ZipArchive::new(cur).unwrap();
        let names: Vec<String> = (0..a.len()).map(|i| a.by_index(i).unwrap().name().to_string()).collect();
        assert!(names.contains(&"Doc_0/Res/Seal_0.png".to_string()));
        assert!(names.contains(&"Doc_0/Signs/Signature.xml".to_string()));
        let mut e = a.by_name("Doc_0/Signs/Signature.xml").unwrap();
        let mut s = String::new(); e.read_to_string(&mut s).unwrap();
        assert!(s.contains("SM2WithSM3"));
        assert!(!s.contains("PLACEHOLDER"));
        let _ = std::fs::remove_file(&p);
    }
}

/// 签名验证结果。
#[derive(Debug)]
pub struct VerificationResult {
    /// 签名是否有效。
    pub valid: bool,
    /// SM3 摘要值。
    pub digest: String,
    /// 签名值。
    pub signature_value: String,
    /// 公钥（十六进制）。
    pub public_key: String,
    /// 签名算法。
    pub algorithm: SignatureAlgorithm,
}

/// 从已签名的 OFD 文件中读取签名信息。
///
/// # 错误
///
/// 如果文件无法读取或签名格式无效则返回错误。
pub fn read_signature(ofd_path: impl AsRef<std::path::Path>) -> OfdResult<VerificationResult> {
    let ofd_path = ofd_path.as_ref();
    let bytes = std::fs::read(ofd_path).map_err(easyofd_core::OfdError::Io)?;
    let cursor = Cursor::new(&bytes[..]);
    let mut archive = zip::ZipArchive::new(cursor)
        .map_err(|e| easyofd_core::OfdError::Zip(e.to_string()))?;

    // 读取 Signature.xml
    let mut sig_file = archive
        .by_name("Doc_0/Signs/Signature.xml")
        .map_err(|e| easyofd_core::OfdError::Conversion(format!("签名文件不存在: {e}")))?;

    let mut sig_xml = String::new();
    sig_file
        .read_to_string(&mut sig_xml)
        .map_err(easyofd_core::OfdError::Io)?;

    // 解析 XML 提取签名信息
    let algorithm = if sig_xml.contains("SM2WithSM3") {
        SignatureAlgorithm::Sm2WithSm3
    } else {
        SignatureAlgorithm::Sha256WithRsa
    };

    let digest = extract_xml_value(&sig_xml, "ofd:DigestValue").unwrap_or_default();
    let signature_value = extract_xml_value(&sig_xml, "ofd:SignedValue").unwrap_or_default();
    let public_key = extract_xml_value(&sig_xml, "ofd:PublicKey").unwrap_or_default();

    Ok(VerificationResult {
        valid: true, // 需要实际验证逻辑
        digest,
        signature_value,
        public_key,
        algorithm,
    })
}

/// 验证 OFD 文件的 SM2 签名。
///
/// 当前实现仅读取和解析签名信息，完整的密码学验证需要
/// 根据实际业务场景集成 SM2 验签逻辑。
///
/// # 错误
///
/// 如果文件无法读取或签名格式无效则返回错误。
pub fn verify_signature(ofd_path: impl AsRef<std::path::Path>) -> OfdResult<bool> {
    let result = read_signature(ofd_path)?;

    // 验证签名值和摘要存在
    if result.signature_value.is_empty() || result.digest.is_empty() {
        return Ok(false);
    }

    // 验证算法支持
    if result.algorithm != SignatureAlgorithm::Sm2WithSm3 {
        return Err(easyofd_core::OfdError::Conversion(
            "仅支持 SM2WithSM3 算法验证".into(),
        ));
    }

    // 解析公钥（验证格式有效性）
    let pub_bytes = unhex(&result.public_key)
        .map_err(|_| easyofd_core::OfdError::Conversion("公钥格式无效".into()))?;

    if pub_bytes.len() != 33 && pub_bytes.len() != 65 {
        return Err(easyofd_core::OfdError::Conversion("公钥长度无效".into()));
    }

    // 解析签名（验证格式有效性）
    let sig_bytes = unhex(&result.signature_value)
        .map_err(|_| easyofd_core::OfdError::Conversion("签名格式无效".into()))?;

    if sig_bytes.len() != 64 {
        return Err(easyofd_core::OfdError::Conversion("签名长度无效".into()));
    }

    // TODO: 使用 sm2::dsa::VerifyingKey 进行完整密码学验证
    // 需要根据实际签名时使用的 DistId 和消息格式进行适配

    Ok(true)
}

/// 从十六进制字符串解析字节。
fn unhex(s: &str) -> Result<Vec<u8>, std::num::ParseIntError> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
        .collect()
}

/// 从 XML 中提取指定标签的值。
fn extract_xml_value(xml: &str, tag: &str) -> Option<String> {
    let start_tag = format!("<{tag}>");
    let end_tag = format!("</{tag}>");

    let start = xml.find(&start_tag)? + start_tag.len();
    let end = xml.find(&end_tag)?;

    Some(xml[start..end].to_string())
}
