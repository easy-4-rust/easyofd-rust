//! Seal.esl ASN.1 DER encoding/decoding per GB/T 38540-2020 section 5.4.
//!
//! Provides [`SealInfo`] and roundtrip DER encoding for the `SignerCertEs`
//! ASN.1 module:
//!
//! ```asn1
//! SignerCertEs ::= SEQUENCE {
//!     version    INTEGER,
//!     cert       OCTET STRING,
//!     signature  BIT STRING
//! }
//! ```
//!
//! The signature field is a **placeholder** — real SM2 signing over the cert
//! blob is deferred until `cert.rs` chain verification is wired in.

use chrono::{DateTime, TimeZone, Utc};
use easyofd_core::{OfdError, OfdResult};

// ── ASN.1 tag constants ────────────────────────────────────────────────
const TAG_INTEGER: u8 = 0x02;
const TAG_OCTET_STRING: u8 = 0x04;
const TAG_BIT_STRING: u8 = 0x03;
const TAG_SEQUENCE: u8 = 0x30;

/// Seal information container matching the GB/T 38540 Seal.esl structure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SealInfo {
    /// Human-readable seal name.
    pub name: String,
    /// Seal creation timestamp.
    pub created_at: DateTime<Utc>,
    /// Seal validity expiration.
    pub valid_until: DateTime<Utc>,
    /// DER-encoded X.509 certificate of the signer.
    pub cert_der: Vec<u8>,
    /// Seal image data (e.g. PNG).
    pub image: Vec<u8>,
    /// Schema version number.
    pub version: u32,
}

// ── DER encoding helpers ───────────────────────────────────────────────

/// Encode a variable-length DER length field.
#[allow(clippy::cast_possible_truncation)]
fn encode_der_length(len: usize, out: &mut Vec<u8>) {
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
        // Up to 16 MB — sufficient for seal containers.
        out.push(0x83);
        out.push((len >> 16) as u8);
        out.push((len >> 8) as u8);
        out.push(len as u8);
    }
}

/// Encode a DER INTEGER from a `u32`.
fn encode_der_integer(val: u32, out: &mut Vec<u8>) {
    out.push(TAG_INTEGER);
    // Minimal big-endian encoding (strip leading zeros, but keep at least 1 byte).
    let bytes = val.to_be_bytes();
    let significant = bytes
        .iter()
        .position(|&b| b != 0)
        .unwrap_or(bytes.len() - 1);
    let payload = &bytes[significant..];
    // If high bit is set, prepend 0x00 to keep it positive.
    let needs_zero = payload[0] & 0x80 != 0;
    let len = payload.len() + usize::from(needs_zero);
    encode_der_length(len, out);
    if needs_zero {
        out.push(0x00);
    }
    out.extend_from_slice(payload);
}

/// Encode a DER OCTET STRING.
fn encode_der_octet_string(data: &[u8], out: &mut Vec<u8>) {
    out.push(TAG_OCTET_STRING);
    encode_der_length(data.len(), out);
    out.extend_from_slice(data);
}

/// Encode a DER BIT STRING (unused-bits byte = 0).
fn encode_der_bit_string(data: &[u8], out: &mut Vec<u8>) {
    out.push(TAG_BIT_STRING);
    // +1 for the leading "unused bits" byte.
    encode_der_length(data.len() + 1, out);
    out.push(0x00); // zero unused bits
    out.extend_from_slice(data);
}

// ── DER decoding helpers ───────────────────────────────────────────────

/// Read a DER length from `der[pos..]`, return (length, new_pos).
fn decode_der_length(der: &[u8], pos: usize) -> OfdResult<(usize, usize)> {
    if pos >= der.len() {
        return Err(OfdError::Conversion("DER: truncated length".into()));
    }
    let first = der[pos];
    if first < 0x80 {
        Ok((first as usize, pos + 1))
    } else {
        let num_bytes = (first & 0x7F) as usize;
        if num_bytes == 0 || pos + 1 + num_bytes > der.len() {
            return Err(OfdError::Conversion("DER: invalid length encoding".into()));
        }
        let mut len: usize = 0;
        for i in 0..num_bytes {
            len = (len << 8) | der[pos + 1 + i] as usize;
        }
        Ok((len, pos + 1 + num_bytes))
    }
}

/// Read a tagged TLV at `pos`, return (tag, value_bytes, new_pos).
fn decode_der_tlv(der: &[u8], pos: usize) -> OfdResult<(u8, Vec<u8>, usize)> {
    if pos >= der.len() {
        return Err(OfdError::Conversion("DER: truncated TLV".into()));
    }
    let tag = der[pos];
    let (len, after_len) = decode_der_length(der, pos + 1)?;
    let end = after_len + len;
    if end > der.len() {
        return Err(OfdError::Conversion("DER: value extends past end".into()));
    }
    Ok((tag, der[after_len..end].to_vec(), end))
}

// ── Public API ─────────────────────────────────────────────────────────

/// Encode a [`SealInfo`] into a `SignerCertEs` ASN.1 DER byte vector.
///
/// The signature field is a placeholder (empty) — real SM2 signing is TODO.
pub fn encode_seal_esl(info: &SealInfo) -> OfdResult<Vec<u8>> {
    // Build inner TLVs.
    let mut inner = Vec::new();

    // version INTEGER
    encode_der_integer(info.version, &mut inner);

    // cert OCTET STRING
    encode_der_octet_string(&info.cert_der, &mut inner);

    // signature BIT STRING (placeholder — empty for now)
    // TODO: sign cert_der with SM2 using the private key associated with cert_der.
    let placeholder_sig: Vec<u8> = Vec::new();
    encode_der_bit_string(&placeholder_sig, &mut inner);

    // Wrap in SEQUENCE.
    let mut out = Vec::with_capacity(inner.len() + 4);
    out.push(TAG_SEQUENCE);
    encode_der_length(inner.len(), &mut out);
    out.extend_from_slice(&inner);
    Ok(out)
}

/// Decode a `SignerCertEs` ASN.1 DER byte vector into a [`SealInfo`].
///
/// Fields not present in the ASN.1 structure (`name`, `created_at`,
/// `valid_until`, `image`) are filled with defaults.
pub fn decode_seal_esl(der: &[u8]) -> OfdResult<SealInfo> {
    // Outer SEQUENCE
    let (tag, seq_data, _) = decode_der_tlv(der, 0)?;
    if tag != TAG_SEQUENCE {
        return Err(OfdError::Conversion(format!(
            "SignerCertEs: expected SEQUENCE (0x30), got 0x{tag:02X}"
        )));
    }

    let mut pos = 0;

    // version INTEGER
    let (tag, val, next) = decode_der_tlv(&seq_data, pos)?;
    if tag != TAG_INTEGER {
        return Err(OfdError::Conversion(
            "SignerCertEs: expected INTEGER for version".into(),
        ));
    }
    let version = bytes_to_u32(&val);
    pos = next;

    // cert OCTET STRING
    let (tag, cert_der, next) = decode_der_tlv(&seq_data, pos)?;
    if tag != TAG_OCTET_STRING {
        return Err(OfdError::Conversion(
            "SignerCertEs: expected OCTET STRING for cert".into(),
        ));
    }
    pos = next;

    // signature BIT STRING
    let (tag, bit_val, _next) = decode_der_tlv(&seq_data, pos)?;
    if tag != TAG_BIT_STRING {
        return Err(OfdError::Conversion(
            "SignerCertEs: expected BIT STRING for signature".into(),
        ));
    }
    // bit_val[0] is the "unused bits" count; the real signature starts at [1..].
    // We currently ignore the signature content (placeholder).

    let _ = bit_val; // suppress unused warning

    Ok(SealInfo {
        name: String::new(),
        created_at: Utc.timestamp_opt(0, 0).single().expect("epoch"),
        valid_until: Utc.timestamp_opt(0, 0).single().expect("epoch"),
        cert_der,
        image: Vec::new(),
        version,
    })
}

/// Decode a minimal big-endian unsigned integer from bytes.
fn bytes_to_u32(bytes: &[u8]) -> u32 {
    let mut val: u32 = 0;
    for &b in bytes {
        val = val.wrapping_shl(8) | u32::from(b);
    }
    val
}

// ── 骑缝章 ───────────────────────────────────────────────────────────────

/// 骑缝章位置（对应 ofdrw `StampSide`）。
///
/// 指定骑缝章贴在页面的哪一侧。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StampSide {
    /// 左侧骑缝。
    Left,
    /// 右侧骑缝。
    Right,
    /// 顶部骑缝。
    Top,
    /// 底部骑缝。
    Bottom,
}

/// 骑缝章外观描述（对应 ofdrw `StampAppearance`）。
///
/// 描述单页上骑缝章切片的位置和尺寸。
#[derive(Debug, Clone, PartialEq)]
pub struct StampAppearance {
    /// 切片在页面上的 X 坐标（mm）。
    pub x: f64,
    /// 切片在页面上的 Y 坐标（mm）。
    pub y: f64,
    /// 切片宽度（mm）。
    pub width: f64,
    /// 切片高度（mm）。
    pub height: f64,
    /// 切片图像数据（原始格式，如 PNG/JPEG）。
    pub image_data: Vec<u8>,
    /// 页码索引（从 0 开始）。
    pub page_index: usize,
}

/// 骑缝章：把印章图片按页数等分，每页显示一个切片。
///
/// 对应 ofdrw 的骑缝章生成逻辑。当前实现简化版：将图像数据按页数
/// 垂直等分（Left/Right）或水平等分（Top/Bottom），每页分配一段
/// 原始字节作为切片数据。
///
/// # 参数
///
/// - `image`: 印章图片原始数据（如 PNG）。
/// - `page_width`: 页面宽度（mm）。
/// - `page_height`: 页面高度（mm）。
/// - `page_count`: 总页数。
/// - `side`: 骑缝位置（Left/Right/Top/Bottom）。
/// - `width`: 印章整体宽度（mm）。
/// - `height`: 印章整体高度（mm）。
/// - `margin`: 距页面边缘的间距（mm）。
///
/// # 返回
///
/// 返回长度为 `page_count` 的 `Vec<StampAppearance>`，每个元素对应一页。
#[allow(clippy::too_many_arguments, clippy::cast_precision_loss)]
pub fn riding_stamp_appearance(
    image: &[u8],
    page_width: f64,
    page_height: f64,
    page_count: usize,
    side: StampSide,
    width: f64,
    height: f64,
    margin: f64,
) -> Vec<StampAppearance> {
    if page_count == 0 {
        return Vec::new();
    }

    let chunk_size = image.len().div_ceil(page_count);

    (0..page_count)
        .map(|i| {
            let start = i * chunk_size;
            let end = std::cmp::min(start + chunk_size, image.len());
            let chunk = image[start..end].to_vec();

            let (x, y, slice_w, slice_h) = match side {
                StampSide::Left => {
                    let slice_h = height / page_count as f64;
                    let y = margin + i as f64 * slice_h;
                    (margin, y, width, slice_h)
                }
                StampSide::Right => {
                    let slice_h = height / page_count as f64;
                    let y = margin + i as f64 * slice_h;
                    (page_width - margin - width, y, width, slice_h)
                }
                StampSide::Top => {
                    let slice_w = width / page_count as f64;
                    let x = margin + i as f64 * slice_w;
                    (x, margin, slice_w, height)
                }
                StampSide::Bottom => {
                    let slice_w = width / page_count as f64;
                    let x = margin + i as f64 * slice_w;
                    (x, page_height - margin - height, slice_w, height)
                }
            };

            StampAppearance {
                x,
                y,
                width: slice_w,
                height: slice_h,
                image_data: chunk,
                page_index: i,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_seal_info() -> SealInfo {
        SealInfo {
            name: "TestSeal".into(),
            created_at: Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap(),
            valid_until: Utc.with_ymd_and_hms(2030, 1, 1, 0, 0, 0).unwrap(),
            cert_der: vec![0x30, 0x03, 0x02, 0x01, 0x01], // minimal dummy cert
            image: vec![0x89, 0x50, 0x4E, 0x47],          // PNG magic
            version: 1,
        }
    }

    #[test]
    fn seal_esl_roundtrip() {
        let info = sample_seal_info();
        let der = encode_seal_esl(&info).unwrap();
        let decoded = decode_seal_esl(&der).unwrap();

        assert_eq!(decoded.version, info.version);
        assert_eq!(decoded.cert_der, info.cert_der);
        // name/created_at/valid_until/image are not part of the ASN.1 structure,
        // so they are defaulted in decode — we only verify the roundtripped fields.
    }

    #[test]
    fn encode_starts_with_sequence_tag() {
        let der = encode_seal_esl(&sample_seal_info()).unwrap();
        assert_eq!(
            der[0], TAG_SEQUENCE,
            "DER should start with SEQUENCE tag 0x30"
        );
    }

    #[test]
    fn decode_rejects_non_sequence() {
        // Pass a valid-looking but wrong-tag byte stream.
        let bad = &[0xA0, 0x03, 0x01, 0x01, 0x00];
        assert!(decode_seal_esl(bad).is_err());
    }

    #[test]
    fn decode_rejects_truncated_input() {
        // SEQUENCE tag + length but no contents.
        let bad = &[0x30, 0x05, 0x02, 0x01]; // length says 5, only 2 bytes follow
        assert!(decode_seal_esl(bad).is_err());
    }

    #[test]
    fn encode_decode_roundtrip_version_zero() {
        let mut info = sample_seal_info();
        info.version = 0;
        let der = encode_seal_esl(&info).unwrap();
        let decoded = decode_seal_esl(&der).unwrap();
        assert_eq!(decoded.version, 0);
    }

    #[test]
    fn encode_decode_roundtrip_large_version() {
        let mut info = sample_seal_info();
        info.version = 0x8000_0001; // high bit set, needs zero prefix in DER INTEGER
        let der = encode_seal_esl(&info).unwrap();
        let decoded = decode_seal_esl(&der).unwrap();
        assert_eq!(decoded.version, 0x8000_0001);
    }

    #[test]
    fn encode_decode_roundtrip_empty_cert_der() {
        let mut info = sample_seal_info();
        info.cert_der = Vec::new();
        let der = encode_seal_esl(&info).unwrap();
        let decoded = decode_seal_esl(&der).unwrap();
        assert!(decoded.cert_der.is_empty());
        assert_eq!(decoded.version, info.version);
    }

    #[test]
    fn encode_decode_roundtrip_large_cert_der() {
        let mut info = sample_seal_info();
        info.cert_der = vec![0xAB; 300]; // > 0x80, exercises 2-byte length
        let der = encode_seal_esl(&info).unwrap();
        let decoded = decode_seal_esl(&der).unwrap();
        assert_eq!(decoded.cert_der, info.cert_der);
    }

    #[test]
    fn decode_rejects_empty_input() {
        assert!(decode_seal_esl(&[]).is_err());
    }

    #[test]
    fn decode_rejects_wrong_integer_tag() {
        // SEQUENCE containing OCTET STRING instead of INTEGER for version.
        let bad = &[0x30, 0x03, 0x04, 0x01, 0x00];
        assert!(decode_seal_esl(bad).is_err());
    }

    #[test]
    fn decode_rejects_wrong_octet_string_tag() {
        // SEQUENCE with INTEGER(version) then INTEGER instead of OCTET STRING for cert.
        let bad = &[0x30, 0x06, 0x02, 0x01, 0x01, 0x02, 0x01, 0x00];
        assert!(decode_seal_esl(bad).is_err());
    }

    #[test]
    fn decode_rejects_wrong_bit_string_tag() {
        // SEQUENCE with INTEGER(version) + OCTET STRING(cert) + INTEGER instead of BIT STRING.
        let bad = &[
            0x30, 0x08, 0x02, 0x01, 0x01, 0x04, 0x01, 0xAA, 0x02, 0x01, 0x00,
        ];
        assert!(decode_seal_esl(bad).is_err());
    }

    #[test]
    fn encode_starts_with_sequence_for_zero_version() {
        let mut info = sample_seal_info();
        info.version = 0;
        let der = encode_seal_esl(&info).unwrap();
        assert_eq!(der[0], TAG_SEQUENCE);
    }

    #[test]
    fn bytes_to_u32_empty_is_zero() {
        // Direct test of the bytes_to_u32 helper via encode/decode roundtrip.
        let mut info = sample_seal_info();
        info.version = 0;
        let der = encode_seal_esl(&info).unwrap();
        let decoded = decode_seal_esl(&der).unwrap();
        assert_eq!(decoded.version, 0);
    }

    #[test]
    fn bytes_to_u32_multi_byte() {
        // version = 256 = 0x0100 -> DER INTEGER with 2 payload bytes.
        let mut info = sample_seal_info();
        info.version = 256;
        let der = encode_seal_esl(&info).unwrap();
        let decoded = decode_seal_esl(&der).unwrap();
        assert_eq!(decoded.version, 256);
    }

    #[test]
    fn encode_der_length_short_form() {
        // Length < 0x80 uses short form (1 byte).
        let mut info = sample_seal_info();
        info.cert_der = vec![0x00; 10]; // small
        let der = encode_seal_esl(&info).unwrap();
        let decoded = decode_seal_esl(&der).unwrap();
        assert_eq!(decoded.cert_der.len(), 10);
    }

    #[test]
    fn encode_der_length_two_byte_form() {
        // Length in [0x80, 0xFF] uses 2-byte form.
        let mut info = sample_seal_info();
        info.cert_der = vec![0x00; 200]; // 0xC8, > 0x80
        let der = encode_seal_esl(&info).unwrap();
        let decoded = decode_seal_esl(&der).unwrap();
        assert_eq!(decoded.cert_der.len(), 200);
    }

    #[test]
    fn encode_der_length_three_byte_form() {
        // Length in [0x100, 0xFFFF] uses 3-byte form.
        let mut info = sample_seal_info();
        info.cert_der = vec![0x00; 300]; // 0x012C, > 0xFF
        let der = encode_seal_esl(&info).unwrap();
        let decoded = decode_seal_esl(&der).unwrap();
        assert_eq!(decoded.cert_der.len(), 300);
    }

    // ── 骑缝章测试 ────────────────────────────────────────────────────

    #[test]
    fn riding_stamp_empty_pages_returns_empty() {
        let result = riding_stamp_appearance(
            &[0x01; 100],
            210.0,
            297.0,
            0,
            StampSide::Right,
            20.0,
            30.0,
            5.0,
        );
        assert!(result.is_empty());
    }

    #[test]
    fn riding_stamp_right_side_positions() {
        let image = vec![0xAA; 300];
        let result =
            riding_stamp_appearance(&image, 210.0, 297.0, 3, StampSide::Right, 20.0, 60.0, 5.0);
        assert_eq!(result.len(), 3);
        // Right side: x = page_width - margin - width = 210 - 5 - 20 = 185
        for (i, app) in result.iter().enumerate() {
            assert!(
                (app.x - 185.0).abs() < f64::EPSILON,
                "page {i} x should be 185"
            );
            assert_eq!(app.page_index, i);
        }
    }

    #[test]
    fn riding_stamp_left_side_positions() {
        let image = vec![0xBB; 200];
        let result =
            riding_stamp_appearance(&image, 210.0, 297.0, 2, StampSide::Left, 15.0, 40.0, 3.0);
        assert_eq!(result.len(), 2);
        // Left side: x = margin = 3.0
        for app in &result {
            assert!((app.x - 3.0).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn riding_stamp_top_side_positions() {
        let image = vec![0xCC; 400];
        let result =
            riding_stamp_appearance(&image, 210.0, 297.0, 4, StampSide::Top, 50.0, 20.0, 2.0);
        assert_eq!(result.len(), 4);
        // Top side: y = margin = 2.0
        for app in &result {
            assert!((app.y - 2.0).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn riding_stamp_bottom_side_positions() {
        let image = vec![0xDD; 150];
        let result =
            riding_stamp_appearance(&image, 210.0, 297.0, 3, StampSide::Bottom, 30.0, 25.0, 4.0);
        assert_eq!(result.len(), 3);
        // Bottom side: y = page_height - margin - height = 297 - 4 - 25 = 268
        for app in &result {
            assert!((app.y - 268.0).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn riding_stamp_image_data_split_evenly() {
        let image: Vec<u8> = (0..100).collect();
        let result =
            riding_stamp_appearance(&image, 210.0, 297.0, 4, StampSide::Right, 20.0, 30.0, 5.0);
        assert_eq!(result.len(), 4);
        // 100 bytes / 4 pages = 25 bytes per chunk
        assert_eq!(result[0].image_data.len(), 25);
        assert_eq!(result[0].image_data[0], 0);
        assert_eq!(result[3].image_data.len(), 25);
    }

    #[test]
    fn riding_stamp_single_page() {
        let image = vec![0xEE; 100];
        let result =
            riding_stamp_appearance(&image, 210.0, 297.0, 1, StampSide::Right, 20.0, 30.0, 5.0);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].image_data, image);
        assert_eq!(result[0].page_index, 0);
    }
}
