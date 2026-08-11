//! RFC 3161 TimeStamp stub for OFD signatures.
//!
//! Provides a local [`TimeStamp`] structure and minimal ASN.1 DER
//! encoding/decoding for a simplified timestamp token. This is a
//! **stub** — no real TSA (Time Stamp Authority) service is contacted.
//!
//! # TODO(gbt38540)
//!
//! A production implementation should:
//! - Send a `TimeStampReq` to a real TSA via HTTP POST (RFC 3161 §2.4.1)
//! - Parse the `TimeStampResp` / `TimeStampToken` CMS ContentInfo
//! - Validate the TSA's certificate chain

use chrono::{DateTime, TimeZone, Utc};
use easyofd_core::{OfdError, OfdResult};

// ── ASN.1 tag constants ────────────────────────────────────────────────
const TAG_SEQUENCE: u8 = 0x30;
const TAG_GENERALIZED_TIME: u8 = 0x18;
const TAG_UTF8_STRING: u8 = 0x0C;
const TAG_OID: u8 = 0x06;

/// Stub timestamp token.
///
/// Represents the minimal fields of an RFC 3161 timestamp token without
/// the full CMS `ContentInfo` wrapper.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimeStamp {
    /// Timestamp generation time.
    pub gen_time: DateTime<Utc>,
    /// TSA (Time Stamp Authority) name.
    pub tsa_name: String,
    /// Signature algorithm OID (e.g. `"1.2.3.4.5"` for the stub).
    pub signature_oid: String,
}

/// Create a local [`TimeStamp`] with the given generation time and TSA name.
///
/// The signature OID is set to the stub value `"1.2.3.4.5"`.
/// No real TSA is contacted.
pub fn create_timestamp(gen_time: DateTime<Utc>, tsa_name: impl Into<String>) -> TimeStamp {
    TimeStamp {
        gen_time,
        tsa_name: tsa_name.into(),
        signature_oid: "1.2.3.4.5".to_string(),
    }
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
        out.push(0x83);
        out.push((len >> 16) as u8);
        out.push((len >> 8) as u8);
        out.push(len as u8);
    }
}

/// Encode a DER GeneralizedTime (YYYYMMDDHHMMSSZ format).
fn encode_generalized_time(dt: DateTime<Utc>, out: &mut Vec<u8>) {
    let s = dt.format("%Y%m%d%H%M%SZ").to_string();
    out.push(TAG_GENERALIZED_TIME);
    encode_der_length(s.len(), out);
    out.extend_from_slice(s.as_bytes());
}

/// Encode a DER UTF8String.
fn encode_utf8_string(s: &str, out: &mut Vec<u8>) {
    let bytes = s.as_bytes();
    out.push(TAG_UTF8_STRING);
    encode_der_length(bytes.len(), out);
    out.extend_from_slice(bytes);
}

/// Encode a dotted-decimal OID string as a DER OID value.
///
/// Does NOT include the tag (0x06) or length — just the value bytes.
#[allow(clippy::cast_possible_truncation)]
fn encode_oid_value(oid_str: &str) -> Vec<u8> {
    let components: Vec<u32> = oid_str.split('.').map(|s| s.parse().unwrap_or(0)).collect();
    if components.len() < 2 {
        return vec![0, 0]; // invalid but safe fallback
    }
    let mut value = Vec::new();
    // First two components are encoded as 40 * a + b.
    value.push((components[0] * 40 + components[1]) as u8);
    // Remaining components use base-128 encoding.
    for &comp in &components[2..] {
        if comp < 0x80 {
            value.push(comp as u8);
        } else if comp < 0x4000 {
            value.push((0x80 | (comp >> 7)) as u8);
            value.push((comp & 0x7F) as u8);
        } else if comp < 0x20_0000 {
            value.push((0x80 | (comp >> 14)) as u8);
            value.push((0x80 | ((comp >> 7) & 0x7F)) as u8);
            value.push((comp & 0x7F) as u8);
        } else {
            value.push((0x80 | (comp >> 21)) as u8);
            value.push((0x80 | ((comp >> 14) & 0x7F)) as u8);
            value.push((0x80 | ((comp >> 7) & 0x7F)) as u8);
            value.push((comp & 0x7F) as u8);
        }
    }
    value
}

/// Encode a DER OID (tag + length + value).
fn encode_oid(oid_str: &str, out: &mut Vec<u8>) {
    let value = encode_oid_value(oid_str);
    out.push(TAG_OID);
    encode_der_length(value.len(), out);
    out.extend_from_slice(&value);
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

/// Decode a GeneralizedTime value (DER value bytes only, no tag/length).
fn decode_generalized_time(bytes: &[u8]) -> OfdResult<DateTime<Utc>> {
    let s = std::str::from_utf8(bytes)
        .map_err(|_| OfdError::Conversion("GeneralizedTime: non-UTF-8".into()))?;
    // Expected format: YYYYMMDDHHMMSSZ
    if s.len() < 15 || !s.ends_with('Z') {
        return Err(OfdError::Conversion(format!(
            "GeneralizedTime: unexpected format: {s}"
        )));
    }
    let year: i32 = s[0..4]
        .parse()
        .map_err(|_| OfdError::Conversion("GeneralizedTime: bad year".into()))?;
    let month: u32 = s[4..6]
        .parse()
        .map_err(|_| OfdError::Conversion("GeneralizedTime: bad month".into()))?;
    let day: u32 = s[6..8]
        .parse()
        .map_err(|_| OfdError::Conversion("GeneralizedTime: bad day".into()))?;
    let hour: u32 = s[8..10]
        .parse()
        .map_err(|_| OfdError::Conversion("GeneralizedTime: bad hour".into()))?;
    let min: u32 = s[10..12]
        .parse()
        .map_err(|_| OfdError::Conversion("GeneralizedTime: bad minute".into()))?;
    let sec: u32 = s[12..14]
        .parse()
        .map_err(|_| OfdError::Conversion("GeneralizedTime: bad second".into()))?;
    Utc.with_ymd_and_hms(year, month, day, hour, min, sec)
        .single()
        .ok_or_else(|| OfdError::Conversion("GeneralizedTime: invalid date".into()))
}

/// Decode an OID from DER value bytes (no tag/length) to dotted-decimal string.
fn decode_oid_value(bytes: &[u8]) -> OfdResult<String> {
    if bytes.is_empty() {
        return Err(OfdError::Conversion("OID: empty value".into()));
    }
    let mut result = Vec::new();
    // First byte encodes first two components.
    let first = bytes[0];
    result.push(u32::from(first / 40));
    result.push(u32::from(first % 40));
    // Remaining components use base-128 encoding.
    let mut i = 1;
    while i < bytes.len() {
        let mut val: u32 = 0;
        loop {
            if i >= bytes.len() {
                return Err(OfdError::Conversion("OID: truncated base-128".into()));
            }
            let b = bytes[i];
            i += 1;
            val = (val << 7) | u32::from(b & 0x7F);
            if b & 0x80 == 0 {
                break;
            }
        }
        result.push(val);
    }
    Ok(result
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("."))
}

// ── Public API ─────────────────────────────────────────────────────────

/// Encode a [`TimeStamp`] into a simplified ASN.1 DER timestamp token.
///
/// Structure:
/// ```asn1
/// TimeStamp ::= SEQUENCE {
///     genTime    GeneralizedTime,
///     tsaName    UTF8String,
///     sigOID     OBJECT IDENTIFIER
/// }
/// ```
pub fn encode_der(ts: &TimeStamp) -> OfdResult<Vec<u8>> {
    let mut inner = Vec::new();

    encode_generalized_time(ts.gen_time, &mut inner);
    encode_utf8_string(&ts.tsa_name, &mut inner);
    encode_oid(&ts.signature_oid, &mut inner);

    let mut out = Vec::with_capacity(inner.len() + 4);
    out.push(TAG_SEQUENCE);
    encode_der_length(inner.len(), &mut out);
    out.extend_from_slice(&inner);
    Ok(out)
}

/// Decode a simplified ASN.1 DER timestamp token into a [`TimeStamp`].
pub fn decode_der(bytes: &[u8]) -> OfdResult<TimeStamp> {
    let (tag, seq_data, _) = decode_der_tlv(bytes, 0)?;
    if tag != TAG_SEQUENCE {
        return Err(OfdError::Conversion(format!(
            "TimeStamp: expected SEQUENCE (0x30), got 0x{tag:02X}"
        )));
    }

    let mut pos = 0;

    // genTime GeneralizedTime
    let (tag, time_bytes, next) = decode_der_tlv(&seq_data, pos)?;
    if tag != TAG_GENERALIZED_TIME {
        return Err(OfdError::Conversion(
            "TimeStamp: expected GeneralizedTime".into(),
        ));
    }
    let gen_time = decode_generalized_time(&time_bytes)?;
    pos = next;

    // tsaName UTF8String
    let (tag, name_bytes, next) = decode_der_tlv(&seq_data, pos)?;
    if tag != TAG_UTF8_STRING {
        return Err(OfdError::Conversion(
            "TimeStamp: expected UTF8String for tsaName".into(),
        ));
    }
    let tsa_name = String::from_utf8(name_bytes)
        .map_err(|e| OfdError::Conversion(format!("TimeStamp: tsaName not UTF-8: {e}")))?;
    pos = next;

    // sigOID OID
    let (tag, oid_bytes, _next) = decode_der_tlv(&seq_data, pos)?;
    if tag != TAG_OID {
        return Err(OfdError::Conversion(
            "TimeStamp: expected OID for sigOID".into(),
        ));
    }
    let signature_oid = decode_oid_value(&oid_bytes)?;

    Ok(TimeStamp {
        gen_time,
        tsa_name,
        signature_oid,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timestamp_roundtrip() {
        let ts = create_timestamp(
            Utc.with_ymd_and_hms(2026, 8, 10, 12, 30, 0).unwrap(),
            "TestTSA",
        );
        let der = encode_der(&ts).unwrap();
        let decoded = decode_der(&der).unwrap();
        assert_eq!(decoded.gen_time, ts.gen_time);
        assert_eq!(decoded.tsa_name, "TestTSA");
        assert_eq!(decoded.signature_oid, "1.2.3.4.5");
    }

    #[test]
    fn encode_starts_with_sequence() {
        let ts = create_timestamp(Utc::now(), "TSA");
        let der = encode_der(&ts).unwrap();
        assert_eq!(der[0], TAG_SEQUENCE);
    }

    #[test]
    fn decode_rejects_empty_input() {
        assert!(decode_der(&[]).is_err());
    }

    #[test]
    fn decode_rejects_non_sequence() {
        assert!(decode_der(&[0xA0, 0x03, 0x01, 0x01, 0x00]).is_err());
    }

    #[test]
    fn decode_rejects_truncated() {
        // SEQUENCE tag + length but not enough content.
        assert!(decode_der(&[0x30, 0x10, 0x18, 0x0F]).is_err());
    }

    #[test]
    fn create_timestamp_sets_stub_oid() {
        let ts = create_timestamp(Utc::now(), "MyTSA");
        assert_eq!(ts.signature_oid, "1.2.3.4.5");
    }

    #[test]
    fn timestamp_roundtrip_various_names() {
        let ts = create_timestamp(
            Utc.with_ymd_and_hms(2030, 1, 1, 0, 0, 0).unwrap(),
            "Long TSA Name With Spaces",
        );
        let der = encode_der(&ts).unwrap();
        let decoded = decode_der(&der).unwrap();
        assert_eq!(decoded.tsa_name, "Long TSA Name With Spaces");
    }
}
