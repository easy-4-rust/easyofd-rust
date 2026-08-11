use easyofd_core::OfdResult;
use std::io::{Cursor, Read};

use crate::algorithm::SignatureAlgorithm;
use crate::internal_helpers::{compute_sm3, hex};
use crate::verification_result::VerificationResult;
use crate::xml;

/// 从已签名的 OFD 文件中读取签名信息。
/// 读取签名后 OFD 的完整签章信息（含 SignedInfo 字节、References、SM2 公钥/签名值）。
///
/// 返回的 [`VerificationResult`] 中 `digest` 是 **SignedInfo.xml 的 SM3**（即 SM2
/// 实际签名的消息摘要）；`signature_value` 是 **SM2 签名值 hex**（来自 SignedValue.dat）。
/// 本函数不再解析整个 OFD bytes 的摘要——按 GB/T 38540，签名对象不是整个文件。
///
/// # 错误
///
/// 文件无法读取、签名格式不完整或 SignedInfo 缺失时返回错误。
pub fn read_signature(ofd_path: impl AsRef<std::path::Path>) -> OfdResult<VerificationResult> {
    let ofd_path = ofd_path.as_ref();
    let bytes = std::fs::read(ofd_path).map_err(easyofd_core::OfdError::Io)?;

    // 一次性把所有 entry 的字节抽到 HashMap，避免 ZipFile 多重借用。
    // 签名后 OFD 通常只有几十个 entry，这个开销可忽略。
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

    // 1. 顶层 Signature.xml：含 SignedInfo/SignedValue 引用 + 公钥 + 签名值 hex。
    let sig_xml = entry_bytes
        .get("Doc_0/Signs/Signature.xml")
        .ok_or_else(|| easyofd_core::OfdError::Conversion("签名文件不存在".into()))?;
    let sig_xml = std::str::from_utf8(sig_xml)
        .map_err(|e| easyofd_core::OfdError::Conversion(format!("Signature.xml 非 UTF-8: {e}")))?
        .to_string();

    // 2. 用 SAX 解析 Signature.xml 提取顶级字段。
    let sig_top = xml::parse_signature_top(&sig_xml)?;

    // SignedInfo.xml 字节 = SM2 实际签名的消息。
    let signed_info_path = sig_top
        .signed_info_ref
        .unwrap_or_else(|| "Doc_0/Signs/SignedInfo.xml".to_string());
    let signed_info_bytes = entry_bytes
        .get(&signed_info_path)
        .ok_or_else(|| easyofd_core::OfdError::Conversion("SignedInfo 缺失".into()))?
        .clone();
    let signed_info_digest = hex(&compute_sm3(&signed_info_bytes));

    // 3. SignedValue.dat = SM2 签名值二进制（raw 64 字节 r||s）。
    let signed_value_path = sig_top
        .signed_value
        .unwrap_or_else(|| "Doc_0/Signs/SignedValue.dat".to_string());
    let signed_value_bytes = entry_bytes
        .get(&signed_value_path)
        .ok_or_else(|| easyofd_core::OfdError::Conversion("SignedValue 缺失".into()))?
        .clone();
    let signature_value_hex = hex(&signed_value_bytes);

    // 4. 公钥 + 算法。
    let public_key = sig_top.public_key.unwrap_or_default();
    let algorithm = if sig_top.method.as_deref() == Some("SM2WithSM3") {
        SignatureAlgorithm::Sm2WithSm3
    } else {
        SignatureAlgorithm::Sha256WithRsa
    };

    // 5. References/FileRef 完整性：对每个 FileRef 重算同名 entry 的 SM3。
    let mut reference_failures: Vec<String> = Vec::new();
    let si_str = String::from_utf8_lossy(&signed_info_bytes);
    let file_refs = xml::parse_signed_info(&si_str)?;
    for entry in &file_refs {
        let actual = entry_bytes
            .get(&*entry.name)
            .map(|data| hex(&compute_sm3(data)));
        if actual.as_deref() != Some(entry.check_value.as_str()) {
            reference_failures.push(entry.name.clone());
        }
    }

    Ok(VerificationResult {
        valid: false,
        digest: signed_info_digest,
        signature_value: signature_value_hex,
        public_key,
        algorithm,
        reference_failures,
    })
}
