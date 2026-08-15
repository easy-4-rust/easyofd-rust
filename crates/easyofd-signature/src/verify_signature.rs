use easyofd_core::OfdResult;
use std::io::{Cursor, Read};

use crate::algorithm::SignatureAlgorithm;
use crate::internal_helpers::{compute_sm3, hex, unhex};
use crate::read_signature::read_signature;
use crate::xml;

/// GB/T 38540 端到端签名验证：
/// 1. 读取 Signature.xml/SignedInfo.xml/SignedValue.dat；
/// 2. 校验 References/FileRef 列出的每个受保护 entry SM3 一致；
/// 3. 若 Signature.xml 含 `<ofd:Seal>` 节点，检查印章匹配（对应 Java:
///    `OFDValidator#checkSealMatch`）；
/// 4. 用嵌入公钥对 SignedInfo 字节做 SM2 验签（DistId 固定为
///    `"1234567812345678"` —— 与签章端一致）。
///
/// 返回 `true` 当且仅当 References 完整性 + 印章匹配（如适用） + SM2 密码学全部通过。
///
/// # 错误
///
/// 文件无法读取、签名结构无效或 SM2 验签失败时返回错误。
pub fn verify_signature(ofd_path: impl AsRef<std::path::Path>) -> OfdResult<bool> {
    let ofd_path = ofd_path.as_ref();
    let result = read_signature(ofd_path)?;

    // 1. References 完整性：任何 FileRef 重算摘要不一致即视为失败。
    if !result.reference_failures.is_empty() {
        return Ok(false);
    }
    if result.algorithm != SignatureAlgorithm::Sm2WithSm3 {
        return Err(easyofd_core::OfdError::Conversion(
            "仅支持 SM2WithSM3 算法验证".into(),
        ));
    }
    if result.public_key.is_empty() || result.signature_value.is_empty() {
        return Ok(false);
    }

    // 2. 一次性把 SignedInfo 字节与 SM2 签名值 hex 解出；
    //    验证不需要重新打开 archive——`read_signature` 已完成结构解析，
    //    `result.signature_value` 就是 SignedValue.dat 的 hex（64 字节）。
    let sig_bytes = unhex(&result.signature_value)
        .map_err(|_| easyofd_core::OfdError::Conversion("签名格式无效".into()))?;
    if sig_bytes.len() != 64 {
        return Err(easyofd_core::OfdError::Conversion("签名长度无效".into()));
    }
    let pub_bytes = unhex(&result.public_key)
        .map_err(|_| easyofd_core::OfdError::Conversion("公钥格式无效".into()))?;
    if pub_bytes.len() != 33 && pub_bytes.len() != 65 {
        return Err(easyofd_core::OfdError::Conversion("公钥长度无效".into()));
    }

    // 3. 一次性读取 Signature.xml / SignedInfo / Seal.esl / SignedValue 字节，
    //    做印章匹配检查，然后释放 archive。
    let signed_info_bytes = {
        let bytes = std::fs::read(ofd_path).map_err(easyofd_core::OfdError::Io)?;
        let cursor = Cursor::new(&bytes[..]);
        let mut archive =
            zip::ZipArchive::new(cursor).map_err(|e| easyofd_core::OfdError::Zip(e.to_string()))?;

        // 读取 Signature.xml 并解析路径。
        let sig_xml = {
            let mut f = archive
                .by_name("Doc_0/Signs/Signature.xml")
                .map_err(|e| easyofd_core::OfdError::Zip(e.to_string()))?;
            let mut s = String::new();
            f.read_to_string(&mut s)
                .map_err(easyofd_core::OfdError::Io)?;
            s
        };
        let sig_top = xml::parse_signature_top(&sig_xml)?;
        let si_path = sig_top
            .signed_info_ref
            .unwrap_or_else(|| "Doc_0/Signs/SignedInfo.xml".to_string());

        // 印章匹配检查（对应 Java: OFDValidator#checkSealMatch）。
        // 仅当 Signature.xml 中存在 <ofd:Seal> 节点时执行。
        if let Some(ref seal_path) = sig_top.seal_path {
            let signed_value_path = sig_top
                .signed_value
                .unwrap_or_else(|| "Doc_0/Signs/SignedValue.dat".to_string());
            let seal_bytes = {
                let mut f = archive
                    .by_name(seal_path)
                    .map_err(|e| easyofd_core::OfdError::Zip(e.to_string()))?;
                let mut buf = Vec::new();
                f.read_to_end(&mut buf)
                    .map_err(easyofd_core::OfdError::Io)?;
                buf
            };
            let sv_bytes = {
                let mut f = archive
                    .by_name(&signed_value_path)
                    .map_err(|e| easyofd_core::OfdError::Zip(e.to_string()))?;
                let mut buf = Vec::new();
                f.read_to_end(&mut buf)
                    .map_err(easyofd_core::OfdError::Io)?;
                buf
            };
            let seal_ok =
                crate::check_seal_match::check_seal_match(&seal_bytes, &sv_bytes).unwrap_or(false);
            if !seal_ok {
                return Ok(false);
            }
        }

        // 读取 SignedInfo 字节。
        let mut signed_info_file = archive
            .by_name(&si_path)
            .map_err(|e| easyofd_core::OfdError::Zip(e.to_string()))?;
        let mut buf = Vec::new();
        signed_info_file
            .read_to_end(&mut buf)
            .map_err(easyofd_core::OfdError::Io)?;
        buf
    };

    // 4. SM2 验签：使用 GB/T 38540 标准 DistId "1234567812345678"。
    let vkey = sm2::dsa::VerifyingKey::from_sec1_bytes("1234567812345678", &pub_bytes)
        .map_err(|e| easyofd_core::OfdError::Conversion(format!("公钥解析失败: {e}")))?;
    let signature = sm2::dsa::Signature::from_slice(&sig_bytes)
        .map_err(|e| easyofd_core::OfdError::Conversion(format!("签名解析失败: {e}")))?;

    use sm2::dsa::signature::Verifier;
    Ok(vkey.verify(&signed_info_bytes, &signature).is_ok())
}

/// Multi-signer verification result for a single signature.
#[derive(Debug)]
pub struct SignatureVerificationResult {
    /// Name of the signature entry (e.g. `"Signature_0.xml"`).
    pub name: String,
    /// Whether the signature is valid (References + SM2 verification).
    pub valid: bool,
    /// SM3 digest of the SignedInfo.xml for this signature.
    pub signed_info_digest: String,
}

/// GB/T 38540 multi-signer verification: parse the OFD.xml
/// `<ofd:Signatures>` list and verify each signature independently.
///
/// For each `<ofd:SignatureRef>` found in the OFD.xml, reads the
/// corresponding Signature XML, checks References/FileRef integrity,
/// and performs SM2-with-SM3 verification.
///
/// Returns one [`SignatureVerificationResult`] per signature.
/// If no `<ofd:Signatures>` element exists, returns an empty vector.
///
/// # Errors
///
/// Returns an error if the OFD file cannot be read or the OFD.xml is
/// malformed. Individual signature verification failures are reported
/// via `valid: false` in the result rather than as errors.
pub fn verify_signature_multi(
    ofd_path: impl AsRef<std::path::Path>,
) -> OfdResult<Vec<SignatureVerificationResult>> {
    let ofd_path = ofd_path.as_ref();
    let bytes = std::fs::read(ofd_path).map_err(easyofd_core::OfdError::Io)?;

    // 1. Extract all ZIP entries into a HashMap.
    let mut entry_bytes: std::collections::HashMap<String, Vec<u8>> =
        std::collections::HashMap::new();
    {
        let cursor = Cursor::new(&bytes[..]);
        let mut archive =
            zip::ZipArchive::new(cursor).map_err(|e| easyofd_core::OfdError::Zip(e.to_string()))?;
        for i in 0..archive.len() {
            let mut e = archive
                .by_index(i)
                .map_err(|e| easyofd_core::OfdError::Zip(e.to_string()))?;
            let name = e.name().to_string();
            let mut buf = Vec::new();
            e.read_to_end(&mut buf)
                .map_err(easyofd_core::OfdError::Io)?;
            entry_bytes.insert(name, buf);
        }
    }

    // 2. Find and parse OFD.xml to get signature references.
    let ofd_xml_bytes = entry_bytes
        .values()
        .zip(entry_bytes.keys())
        .find(|(_, name)| name.ends_with("OFD.xml"))
        .map(|(data, _)| data.clone());

    let ofd_xml = match ofd_xml_bytes {
        Some(data) => String::from_utf8_lossy(&data).to_string(),
        None => return Ok(Vec::new()),
    };

    let ofd_root = xml::parse_ofd_root(&ofd_xml)?;
    if ofd_root.signatures.is_empty() {
        return Ok(Vec::new());
    }

    // 3. Verify each signature independently.
    let mut results = Vec::with_capacity(ofd_root.signatures.len());
    for sig_ref in &ofd_root.signatures {
        let result = verify_single_from_entries(&entry_bytes, &sig_ref.path);
        results.push(result);
    }

    Ok(results)
}

/// Verify a single signature from pre-loaded ZIP entries.
///
/// This is an internal helper shared by `verify_signature_multi`.
#[allow(clippy::too_many_lines)]
fn verify_single_from_entries(
    entry_bytes: &std::collections::HashMap<String, Vec<u8>>,
    sig_xml_path: &str,
) -> SignatureVerificationResult {
    /// Helper to produce an early-exit invalid result.
    fn invalid(name: String, digest: String) -> SignatureVerificationResult {
        SignatureVerificationResult {
            name,
            valid: false,
            signed_info_digest: digest,
        }
    }

    // Extract the signature name from the path (e.g. "Signature_0.xml").
    let name = sig_xml_path
        .rsplit('/')
        .next()
        .unwrap_or(sig_xml_path)
        .to_string();

    // Try to read and parse the signature XML.
    let Some(sig_xml_bytes) = entry_bytes.get(sig_xml_path) else {
        return invalid(name, String::new());
    };
    let Ok(sig_xml) = std::str::from_utf8(sig_xml_bytes) else {
        return invalid(name, String::new());
    };
    let Ok(sig_top) = xml::parse_signature_top(sig_xml) else {
        return invalid(name, String::new());
    };

    // Read SignedInfo bytes and compute digest.
    let si_path = sig_top
        .signed_info_ref
        .unwrap_or_else(|| "Doc_0/Signs/SignedInfo.xml".to_string());
    let Some(signed_info_bytes) = entry_bytes.get(&si_path).cloned() else {
        return invalid(name, String::new());
    };
    let signed_info_digest = hex(&compute_sm3(&signed_info_bytes));

    // Read SignedValue bytes.
    let sv_path = sig_top
        .signed_value
        .unwrap_or_else(|| "Doc_0/Signs/SignedValue.dat".to_string());
    let Some(signed_value_bytes) = entry_bytes.get(&sv_path) else {
        return invalid(name, signed_info_digest);
    };

    // 1. References integrity check.
    let si_str = String::from_utf8_lossy(&signed_info_bytes);
    let Ok(file_refs) = xml::parse_signed_info(&si_str) else {
        return invalid(name, signed_info_digest);
    };
    for entry in &file_refs {
        let actual = entry_bytes
            .get(&*entry.name)
            .map(|data| hex(&compute_sm3(data)));
        if actual.as_deref() != Some(entry.check_value.as_str()) {
            return invalid(name, signed_info_digest);
        }
    }

    // 2. 印章匹配检查（Seal Match Check）。
    //    对应 Java: org.ofdrw.sign.verify.OFDValidator#checkSealMatch
    //    仅当 Signature.xml 中存在 <ofd:Seal> 节点时执行。
    if let Some(ref seal_path) = sig_top.seal_path {
        if let Some(seal_bytes) = entry_bytes.get(seal_path) {
            let seal_ok = crate::check_seal_match::check_seal_match(seal_bytes, signed_value_bytes)
                .unwrap_or(false);
            if !seal_ok {
                return invalid(name, signed_info_digest);
            }
        } else {
            // Seal.esl 文件缺失视为不匹配。
            return invalid(name, signed_info_digest);
        }
    }

    // 3. SM2 verification.
    let public_key = sig_top.public_key.unwrap_or_default();
    let sig_hex = hex(signed_value_bytes);
    if public_key.is_empty() || sig_hex.is_empty() {
        return invalid(name, signed_info_digest);
    }

    let Ok(sig_bytes) = unhex(&sig_hex) else {
        return invalid(name, signed_info_digest);
    };
    if sig_bytes.len() != 64 {
        return invalid(name, signed_info_digest);
    }
    let Ok(pub_bytes) = unhex(&public_key) else {
        return invalid(name, signed_info_digest);
    };
    if pub_bytes.len() != 33 && pub_bytes.len() != 65 {
        return invalid(name, signed_info_digest);
    }

    let valid = sm2::dsa::VerifyingKey::from_sec1_bytes("1234567812345678", &pub_bytes)
        .and_then(|vkey| {
            let signature = sm2::dsa::Signature::from_slice(&sig_bytes)?;
            use sm2::dsa::signature::Verifier;
            vkey.verify(&signed_info_bytes, &signature)
        })
        .is_ok();

    SignatureVerificationResult {
        name,
        valid,
        signed_info_digest,
    }
}
