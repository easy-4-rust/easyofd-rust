//! SES（Secure Electronic Seal）电子印章 ASN.1 结构定义。
//!
//! 对应 Java 版 [`ofdrw-gm`](https://github.com/ofdrw/ofdrw) 中的
//! `org.ofdrw.gm.ses` 包，实现 GB/T 38540-2020 标准规定的
//! SESeal / SES_Signature ASN.1 DER 编码与解码。
//!
//! # 版本说明
//!
//! - [`v1`] — SES V1 结构（GB/T 38540-2020 第一版），UTCTime 时间格式。
//! - [`v4`] — SES V4 结构，展平 cert/alg/sig 到顶层，GeneralizedTime，
//!   CertList 支持 CHOICE（full cert 或 digest）。
//! - [`v5`] — SES V5 结构，等同 V4 + 可选 timeStamp。

mod der_error;
pub mod v1;
pub mod v4;
pub mod v5;

pub use der_error::{DerError, DerResult};

// ── 公共 DER 编码/解码工具 ─────────────────────────────────────────────

/// ASN.1 tag 常量。
pub(crate) const TAG_INTEGER: u8 = 0x02;
pub(crate) const TAG_BIT_STRING: u8 = 0x03;
pub(crate) const TAG_OCTET_STRING: u8 = 0x04;
pub(crate) const TAG_OBJECT_IDENTIFIER: u8 = 0x06;
#[allow(dead_code)]
pub(crate) const TAG_UTF8_STRING: u8 = 0x0C;
pub(crate) const TAG_PRINTABLE_STRING: u8 = 0x13;
pub(crate) const TAG_IA5_STRING: u8 = 0x16;
pub(crate) const TAG_GENERALIZED_TIME: u8 = 0x18;
pub(crate) const TAG_SEQUENCE: u8 = 0x30;
#[allow(dead_code)]
pub(crate) const TAG_SET: u8 = 0x31;

/// 上下文特定显式标签基址（Constructed, Context-specific）。
const CONTEXT_SPECIFIC: u8 = 0xA0;

// ── 编码辅助函数 ──────────────────────────────────────────────────────

/// 编码 DER 长度字段（variable-length）。
#[allow(clippy::cast_possible_truncation)]
pub(crate) fn encode_length(len: usize, out: &mut Vec<u8>) {
    if len < 0x80 {
        out.push(len as u8);
    } else if len <= 0xFF {
        out.push(0x81);
        out.push(len as u8);
    } else if len <= 0xFFFF {
        out.push(0x82);
        out.push((len >> 8) as u8);
        out.push(len as u8);
    } else {
        out.push(0x83);
        out.push((len >> 16) as u8);
        out.push((len >> 8) as u8);
        out.push(len as u8);
    }
}

/// 编码 DER INTEGER（无符号）。
pub(crate) fn encode_integer(val: u64, out: &mut Vec<u8>) {
    out.push(TAG_INTEGER);
    let bytes = val.to_be_bytes();
    let significant = bytes
        .iter()
        .position(|&b| b != 0)
        .unwrap_or(bytes.len() - 1);
    let payload = &bytes[significant..];
    let needs_zero = !payload.is_empty() && (payload[0] & 0x80 != 0);
    let len = payload.len() + usize::from(needs_zero);
    encode_length(len, out);
    if needs_zero {
        out.push(0x00);
    }
    out.extend_from_slice(payload);
}

/// 编码 DER OCTET STRING。
pub(crate) fn encode_octet_string(data: &[u8], out: &mut Vec<u8>) {
    out.push(TAG_OCTET_STRING);
    encode_length(data.len(), out);
    out.extend_from_slice(data);
}

/// 编码 DER BIT STRING（unused-bits byte = 0）。
pub(crate) fn encode_bit_string(data: &[u8], out: &mut Vec<u8>) {
    out.push(TAG_BIT_STRING);
    encode_length(data.len() + 1, out);
    out.push(0x00);
    out.extend_from_slice(data);
}

/// 编码 DER OBJECT IDENTIFIER。
///
/// `arcs` 是 OID 的各个弧段，例如 `[1, 2, 840, 113549, 1, 1, 11]`。
#[allow(clippy::cast_possible_truncation)]
pub(crate) fn encode_oid(arcs: &[u32], out: &mut Vec<u8>) {
    debug_assert!(arcs.len() >= 2, "OID must have at least 2 arcs");
    let mut body = Vec::new();
    // 前两个弧段合并为 `40 * first + second`。
    body.push((arcs[0] * 40 + arcs[1]) as u8);
    for &arc in &arcs[2..] {
        encode_oid_arc(arc, &mut body);
    }
    out.push(TAG_OBJECT_IDENTIFIER);
    encode_length(body.len(), out);
    out.extend_from_slice(&body);
}

/// 编码单个 OID 弧段（base-128 变长编码，高位标记）。
#[allow(clippy::cast_possible_truncation)]
fn encode_oid_arc(mut arc: u32, out: &mut Vec<u8>) {
    if arc < 0x80 {
        out.push(arc as u8);
        return;
    }
    let mut temp = Vec::new();
    temp.push((arc & 0x7F) as u8);
    arc >>= 7;
    while arc > 0 {
        temp.push((arc & 0x7F) as u8 | 0x80);
        arc >>= 7;
    }
    temp.reverse();
    out.extend_from_slice(&temp);
}

/// 编码 DER IA5String。
pub(crate) fn encode_ia5_string(s: &str, out: &mut Vec<u8>) {
    out.push(TAG_IA5_STRING);
    encode_length(s.len(), out);
    out.extend_from_slice(s.as_bytes());
}

/// 编码 DER UTF8String。
#[allow(dead_code)]
pub(crate) fn encode_utf8_string(s: &str, out: &mut Vec<u8>) {
    out.push(TAG_UTF8_STRING);
    encode_length(s.len(), out);
    out.extend_from_slice(s.as_bytes());
}

/// 编码 DER PrintableString。
pub(crate) fn encode_printable_string(s: &str, out: &mut Vec<u8>) {
    out.push(TAG_PRINTABLE_STRING);
    encode_length(s.len(), out);
    out.extend_from_slice(s.as_bytes());
}

/// 编码 SEQUENCE 头并追加内部字节到 `out`。
pub(crate) fn encode_sequence(inner: &[u8], out: &mut Vec<u8>) {
    out.push(TAG_SEQUENCE);
    encode_length(inner.len(), out);
    out.extend_from_slice(inner);
}

/// 编码 DER UTCTime（YYMMDDHHmmSSZ 格式，13 字节）。
pub(crate) fn encode_utc_time(s: &str, out: &mut Vec<u8>) {
    assert!(
        s.len() == 13 && s.ends_with('Z'),
        "UTCTime must be YYMMDDHHmmSSZ"
    );
    out.push(0x17); // UTCTime tag
    encode_length(s.len(), out);
    out.extend_from_slice(s.as_bytes());
}

/// 编码 DER GeneralizedTime（YYYYMMDDHHmmSSZ 格式，15 字节）。
pub(crate) fn encode_generalized_time(s: &str, out: &mut Vec<u8>) {
    assert!(
        s.len() == 15 && s.ends_with('Z'),
        "GeneralizedTime must be YYYYMMDDHHmmSSZ"
    );
    out.push(TAG_GENERALIZED_TIME);
    encode_length(s.len(), out);
    out.extend_from_slice(s.as_bytes());
}

/// 编码上下文特定显式标签 `[n]`（Constructed）。
pub(crate) fn encode_context_explicit(n: u8, inner: &[u8], out: &mut Vec<u8>) {
    out.push(CONTEXT_SPECIFIC | n);
    encode_length(inner.len(), out);
    out.extend_from_slice(inner);
}

// ── 解码辅助函数 ──────────────────────────────────────────────────────

/// 解码 DER 长度，返回 `(length, new_pos)`。
pub(crate) fn decode_length(der: &[u8], pos: usize) -> DerResult<(usize, usize)> {
    if pos >= der.len() {
        return Err(DerError("truncated length"));
    }
    let first = der[pos];
    if first < 0x80 {
        Ok((first as usize, pos + 1))
    } else {
        let num_bytes = (first & 0x7F) as usize;
        if num_bytes == 0 || pos + 1 + num_bytes > der.len() {
            return Err(DerError("invalid length encoding"));
        }
        let mut len: usize = 0;
        for i in 0..num_bytes {
            len = (len << 8) | der[pos + 1 + i] as usize;
        }
        Ok((len, pos + 1 + num_bytes))
    }
}

/// 解码 DER TLV，返回 `(tag, value_bytes, new_pos)`。
pub(crate) fn decode_tlv(der: &[u8], pos: usize) -> DerResult<(u8, Vec<u8>, usize)> {
    if pos >= der.len() {
        return Err(DerError("truncated TLV"));
    }
    let tag = der[pos];
    let (len, after_len) = decode_length(der, pos + 1)?;
    let end = after_len + len;
    if end > der.len() {
        return Err(DerError("value extends past end"));
    }
    Ok((tag, der[after_len..end].to_vec(), end))
}

/// 解码无符号整数（DER INTEGER 的 value 部分）。
pub(crate) fn decode_uint(bytes: &[u8]) -> u64 {
    let mut val: u64 = 0;
    for &b in bytes {
        val = val.wrapping_shl(8) | u64::from(b);
    }
    val
}

/// 解码 OID（DER OBJECT IDENTIFIER 的 value 部分），返回弧段数组。
pub(crate) fn decode_oid(bytes: &[u8]) -> DerResult<Vec<u32>> {
    if bytes.is_empty() {
        return Err(DerError("empty OID"));
    }
    let mut arcs = Vec::new();
    let first = bytes[0];
    arcs.push(u32::from(first / 40));
    arcs.push(u32::from(first % 40));

    let mut pos = 1;
    while pos < bytes.len() {
        let mut arc: u32 = 0;
        loop {
            if pos >= bytes.len() {
                return Err(DerError("truncated OID arc"));
            }
            let b = bytes[pos];
            arc = arc.checked_shl(7).ok_or(DerError("OID arc overflow"))? | u32::from(b & 0x7F);
            pos += 1;
            if b & 0x80 == 0 {
                break;
            }
        }
        arcs.push(arc);
    }
    Ok(arcs)
}

/// 期望并解码指定 tag 的 TLV，返回 value 字节和新位置。
pub(crate) fn expect_tlv(der: &[u8], pos: usize, expected_tag: u8) -> DerResult<(Vec<u8>, usize)> {
    let (tag, val, next) = decode_tlv(der, pos)?;
    if tag != expected_tag {
        return Err(DerError("unexpected tag"));
    }
    Ok((val, next))
}

/// 解码 SEQUENCE 的 value 部分（跳过 SEQUENCE 头）。
pub(crate) fn decode_sequence(der: &[u8], pos: usize) -> DerResult<(Vec<u8>, usize)> {
    let (tag, val, next) = decode_tlv(der, pos)?;
    if tag != TAG_SEQUENCE {
        return Err(DerError("expected SEQUENCE"));
    }
    Ok((val, next))
}

/// 解码上下文特定显式标签 `[n]`，返回内部 value 和新位置。
/// 如果当前位置的 tag 不匹配 `n`，返回 `Ok(None, pos)`。
pub(crate) fn decode_context_explicit_optional(
    der: &[u8],
    pos: usize,
    n: u8,
) -> DerResult<(Option<Vec<u8>>, usize)> {
    if pos >= der.len() {
        return Ok((None, pos));
    }
    let expected_tag = CONTEXT_SPECIFIC | n;
    if der[pos] != expected_tag {
        return Ok((None, pos));
    }
    let (_tag, val, next) = decode_tlv(der, pos)?;
    // val 已经是内部值（decode_tlv 提取了 value 部分）
    Ok((Some(val), next))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_length_short_form() {
        let mut out = Vec::new();
        encode_length(0x7F, &mut out);
        assert_eq!(out, vec![0x7F]);
    }

    #[test]
    fn test_encode_length_two_byte_form() {
        let mut out = Vec::new();
        encode_length(0x80, &mut out);
        assert_eq!(out, vec![0x81, 0x80]);
    }

    #[test]
    fn test_encode_decode_oid() {
        // RSA SHA-256: 1.2.840.113549.1.1.11
        let arcs = [1, 2, 840, 113_549, 1, 1, 11];
        let mut out = Vec::new();
        encode_oid(&arcs, &mut out);
        assert_eq!(out[0], TAG_OBJECT_IDENTIFIER);
        // Decode the value part
        let decoded = decode_oid(&out[2..]).unwrap();
        assert_eq!(decoded, arcs);
    }

    #[test]
    fn test_encode_decode_oid_sm2() {
        // SM2 with SM3: 1.2.156.10197.1.501
        let arcs = [1, 2, 156, 10_197, 1, 501];
        let mut out = Vec::new();
        encode_oid(&arcs, &mut out);
        let decoded = decode_oid(&out[2..]).unwrap();
        assert_eq!(decoded, arcs);
    }

    #[test]
    fn test_decode_uint() {
        assert_eq!(decode_uint(&[0x00]), 0);
        assert_eq!(decode_uint(&[0x01]), 1);
        assert_eq!(decode_uint(&[0x01, 0x00]), 256);
        assert_eq!(decode_uint(&[0x80, 0x00, 0x00, 0x01]), 0x8000_0001);
    }
}
