//! Certificate Revocation List (CRL) parsing and OCSP stub.
//!
//! Provides [`CrlInfo`] as a flat representation of a CRL, a revocation-check
//! helper, and a placeholder OCSP responder stub.
//!
//! These APIs are **not** wired into the signature verification path yet.

use chrono::{DateTime, TimeZone, Utc};
use der::{Decode, Encode};
use easyofd_core::{OfdError, OfdResult};
use std::io::{Read, Write as _};
use std::net::TcpStream;
use std::time::Duration;
use x509_cert::Certificate;
use x509_cert::certificate::Rfc5280;
use x509_cert::crl::CertificateList;

/// Flat representation of CRL fields relevant to OFD certificate validation.
#[derive(Debug, Clone)]
pub struct CrlInfo {
    /// Issuer distinguished name (RFC 4514 string).
    pub issuer: String,
    /// Serial numbers of revoked certificates (big-endian, leading zeros stripped).
    pub revoked_serials: Vec<Vec<u8>>,
    /// This-update timestamp.
    pub this_update: DateTime<Utc>,
    /// Next-update timestamp.
    pub next_update: DateTime<Utc>,
}

/// Convert a `der::DateTime` to `chrono::DateTime<Utc>`.
fn der_datetime_to_chrono(dt: der::DateTime) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(
        i32::from(dt.year()),
        u32::from(dt.month()),
        u32::from(dt.day()),
        u32::from(dt.hour()),
        u32::from(dt.minutes()),
        u32::from(dt.seconds()),
    )
    .single()
    .unwrap_or_else(|| Utc.timestamp_opt(0, 0).single().expect("epoch"))
}

/// Convert an `x509_cert::time::Time` to `chrono::DateTime<Utc>`.
fn x509_time_to_chrono(t: x509_cert::time::Time) -> DateTime<Utc> {
    der_datetime_to_chrono(t.to_date_time())
}

/// Parse a CRL from DER-encoded bytes.
///
/// Extracts issuer, revoked serial numbers, and update timestamps.
#[allow(dead_code)]
pub fn parse_crl_der(der: &[u8]) -> OfdResult<CrlInfo> {
    let crl: CertificateList<Rfc5280> = CertificateList::from_der(der)
        .map_err(|e| OfdError::Conversion(format!("CRL DER parse error: {e}")))?;

    let tbs = &crl.tbs_cert_list;
    let issuer = tbs.issuer.to_string();

    let this_update = x509_time_to_chrono(tbs.this_update);
    let next_update = tbs
        .next_update
        .map_or(DateTime::<Utc>::MAX_UTC, x509_time_to_chrono);

    let revoked_serials: Vec<Vec<u8>> = tbs
        .revoked_certificates
        .as_ref()
        .map(|revoked| {
            revoked
                .iter()
                .map(|rc| rc.serial_number.as_bytes().to_vec())
                .collect()
        })
        .unwrap_or_default();

    Ok(CrlInfo {
        issuer,
        revoked_serials,
        this_update,
        next_update,
    })
}

/// Check whether a certificate serial number appears in the CRL's revoked list.
#[allow(dead_code)]
pub fn check_revoked(crl: &CrlInfo, serial: &[u8]) -> bool {
    crl.revoked_serials.iter().any(|s| s.as_slice() == serial)
}

/// OCSP certificate status check — currently a placeholder.
///
/// Always returns `Ok(false)` (certificate not revoked).
///
/// # TODO(gbt38540)
///
/// Real OCSP protocol (RFC 6960) is not yet implemented. When an OCSP
/// responder URL is available, this function should construct an OCSP
/// request, send it via HTTP POST, and parse the `BasicOCSPResponse`.
/// GB/T 38540 does not mandate OCSP but it is common in production PKI
/// deployments alongside CRL checking.
#[allow(dead_code)]
pub fn ocsp_check(_serial: &[u8]) -> OfdResult<bool> {
    // Placeholder: assume certificate is not revoked.
    // Future: send OCSP request to responder URL, parse BasicOCSPResponse.
    Ok(false)
}

// ═══════════════════════════════════════════════════════════════════════
// OCSP client implementation (RFC 6960)
// ═══════════════════════════════════════════════════════════════════════

/// Minimal SHA-1 implementation (RFC 3174) for OCSP CertID hashing.
#[allow(clippy::many_single_char_names)]
fn sha1(data: &[u8]) -> [u8; 20] {
    let mut h: [u32; 5] = [
        0x6745_2301,
        0xEFCD_AB89,
        0x98BA_DCFE,
        0x1032_5476,
        0xC3D2_E1F0,
    ];
    let mut msg = data.to_vec();
    let bit_len = u64::try_from(msg.len()).unwrap_or(0) * 8;
    msg.push(0x80);
    while msg.len() % 64 != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&bit_len.to_be_bytes());
    for chunk in msg.chunks(64) {
        let mut w = [0_u32; 80];
        for i in 0..16 {
            w[i] = u32::from_be_bytes(chunk[i * 4..(i + 1) * 4].try_into().expect("sha1 chunk"));
        }
        for i in 16..80 {
            w[i] = (w[i - 3] ^ w[i - 8] ^ w[i - 14] ^ w[i - 16]).rotate_left(1);
        }
        let [mut a, mut b, mut c, mut d, mut e] = h;
        #[allow(clippy::needless_range_loop)]
        for i in 0..80 {
            let (f, k) = match i {
                0..=19 => ((b & c) | (!b & d), 0x5A82_7999),
                20..=39 => (b ^ c ^ d, 0x6ED9_EBA1),
                40..=59 => ((b & c) | (b & d) | (c & d), 0x8F1B_BCDC),
                _ => (b ^ c ^ d, 0xCA62_C1D6),
            };
            let temp = a
                .rotate_left(5)
                .wrapping_add(f)
                .wrapping_add(e)
                .wrapping_add(k)
                .wrapping_add(w[i]);
            e = d;
            d = c;
            c = b.rotate_left(30);
            b = a;
            a = temp;
        }
        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
    }
    let mut out = [0_u8; 20];
    for (i, word) in h.iter().enumerate() {
        out[i * 4..(i + 1) * 4].copy_from_slice(&word.to_be_bytes());
    }
    out
}

/// Encode a DER TLV (Tag-Length-Value) wrapper.
#[allow(clippy::cast_possible_truncation)]
fn der_wrap(tag: u8, value: &[u8]) -> Vec<u8> {
    let len = value.len();
    let mut out = Vec::with_capacity(4 + len);
    out.push(tag);
    if len < 0x80 {
        out.push(len as u8);
    } else if len < 0x100 {
        out.extend_from_slice(&[0x81, len as u8]);
    } else {
        out.extend_from_slice(&[0x82, (len >> 8) as u8, (len & 0xFF) as u8]);
    }
    out.extend_from_slice(value);
    out
}

/// Wrap value bytes in a DER SEQUENCE.
fn der_sequence(value: &[u8]) -> Vec<u8> {
    der_wrap(0x30, value)
}

/// Wrap value bytes in a DER OCTET STRING.
fn der_octet_string(value: &[u8]) -> Vec<u8> {
    der_wrap(0x04, value)
}

/// Encode bytes as a positive DER INTEGER (strips leading zeros, prepends
/// `0x00` if the high bit is set).
fn der_integer_positive(bytes: &[u8]) -> Vec<u8> {
    if bytes.is_empty() {
        return vec![0x02, 0x01, 0x00];
    }
    let start = bytes
        .iter()
        .position(|&b| b != 0)
        .unwrap_or(bytes.len() - 1);
    let stripped = &bytes[start..];
    if stripped.is_empty() {
        return vec![0x02, 0x01, 0x00];
    }
    if stripped[0] & 0x80 != 0 {
        let mut val = Vec::with_capacity(1 + stripped.len());
        val.push(0x00);
        val.extend_from_slice(stripped);
        der_wrap(0x02, &val)
    } else {
        der_wrap(0x02, stripped)
    }
}

/// Build an OCSP request DER per RFC 6960.
///
/// Uses SHA-1 for the CertID hash (standard). `issuer_name_der` is the
/// DER-encoded issuer DN; `issuer_spki_der` is the DER-encoded
/// SubjectPublicKeyInfo.
fn build_ocsp_request(serial: &[u8], issuer_name_der: &[u8], issuer_spki_der: &[u8]) -> Vec<u8> {
    let name_hash = sha1(issuer_name_der);
    let key_hash = sha1(issuer_spki_der);

    // SHA-1 AlgorithmIdentifier: SEQUENCE { OID 1.3.14.3.2.26, NULL }
    let sha1_oid: &[u8] = &[0x06, 0x05, 0x2B, 0x0E, 0x03, 0x02, 0x1A];
    let null_bytes: &[u8] = &[0x05, 0x00];
    let mut alg_val = Vec::with_capacity(sha1_oid.len() + null_bytes.len());
    alg_val.extend_from_slice(sha1_oid);
    alg_val.extend_from_slice(null_bytes);
    let alg_id = der_sequence(&alg_val);

    // CertID: SEQUENCE { alg_id, nameHash, keyHash, serial }
    let mut cid_val = Vec::new();
    cid_val.extend_from_slice(&alg_id);
    cid_val.extend_from_slice(&der_octet_string(&name_hash));
    cid_val.extend_from_slice(&der_octet_string(&key_hash));
    cid_val.extend_from_slice(&der_integer_positive(serial));
    let cert_id = der_sequence(&cid_val);

    // Request -> requestList -> TBSRequest -> OCSPRequest
    let request = der_sequence(&cert_id);
    let request_list = der_sequence(&request);
    let tbs_request = der_sequence(&request_list);
    der_sequence(&tbs_request)
}

/// Read a DER Tag-Length header, advancing `pos` past it.
fn read_tl(der: &[u8], pos: &mut usize) -> OfdResult<(u8, usize)> {
    if *pos >= der.len() {
        return Err(OfdError::Conversion("DER: unexpected end".into()));
    }
    let tag = der[*pos];
    *pos += 1;
    if *pos >= der.len() {
        return Err(OfdError::Conversion("DER: missing length".into()));
    }
    let first = der[*pos];
    *pos += 1;
    let length = if first & 0x80 == 0 {
        usize::from(first)
    } else {
        let n = usize::from(first & 0x7F);
        if n == 0 || *pos + n > der.len() {
            return Err(OfdError::Conversion("DER: bad length".into()));
        }
        let mut len = 0_usize;
        for i in 0..n {
            len = (len << 8) | usize::from(der[*pos + i]);
        }
        *pos += n;
        len
    };
    Ok((tag, length))
}

/// Parse an OCSP response and return `Ok(true)` if the certificate is revoked.
///
/// Only the `certStatus` field is inspected; signature verification of the
/// response is **not** performed (would require the responder certificate).
fn parse_ocsp_response(response: &[u8]) -> OfdResult<bool> {
    let mut p = 0;

    // OCSPResponse SEQUENCE
    let (tag, _) = read_tl(response, &mut p)?;
    if tag != 0x30 {
        return Err(OfdError::Conversion("OCSP: not SEQUENCE".into()));
    }

    // responseStatus ENUMERATED
    let (tag, len) = read_tl(response, &mut p)?;
    if tag != 0x0A || len != 1 {
        return Err(OfdError::Conversion("OCSP: bad responseStatus".into()));
    }
    let status = response[p];
    p += 1;
    if status != 0 {
        return Ok(false); // not successful -> conservative: not revoked
    }

    // responseBytes [0] EXPLICIT (may be absent)
    if p >= response.len() || response[p] != 0xA0 {
        return Ok(false);
    }
    let (_tag, _) = read_tl(response, &mut p)?;

    // ResponseBytes SEQUENCE
    let (tag, _) = read_tl(response, &mut p)?;
    if tag != 0x30 {
        return Err(OfdError::Conversion("OCSP: bad ResponseBytes".into()));
    }

    // responseType OID -- skip
    let (tag, oid_len) = read_tl(response, &mut p)?;
    if tag != 0x06 {
        return Err(OfdError::Conversion("OCSP: bad responseType".into()));
    }
    p += oid_len;

    // response OCTET STRING (contains BasicOCSPResponse)
    let (tag, _) = read_tl(response, &mut p)?;
    if tag != 0x04 {
        return Err(OfdError::Conversion(
            "OCSP: bad response OCTET STRING".into(),
        ));
    }

    // BasicOCSPResponse SEQUENCE
    let (tag, _) = read_tl(response, &mut p)?;
    if tag != 0x30 {
        return Err(OfdError::Conversion("OCSP: bad BasicOCSPResponse".into()));
    }

    // ResponseData SEQUENCE (tbsResponseData)
    let (tag, _) = read_tl(response, &mut p)?;
    if tag != 0x30 {
        return Err(OfdError::Conversion("OCSP: bad ResponseData".into()));
    }

    // Skip optional version [0] EXPLICIT
    if p < response.len() && response[p] == 0xA0 {
        let (_, len) = read_tl(response, &mut p)?;
        p += len;
    }

    // Skip responderID ([1] byName or [2] byKey)
    if p < response.len() && (response[p] & 0xE0) == 0xA0 {
        let (_, len) = read_tl(response, &mut p)?;
        p += len;
    }

    // Skip producedAt (GeneralizedTime 0x18)
    if p < response.len() && response[p] == 0x18 {
        let (_, len) = read_tl(response, &mut p)?;
        p += len;
    }

    // responses SEQUENCE OF SingleResponse
    let (tag, _) = read_tl(response, &mut p)?;
    if tag != 0x30 {
        return Err(OfdError::Conversion("OCSP: bad responses".into()));
    }

    // First SingleResponse SEQUENCE
    let (tag, _) = read_tl(response, &mut p)?;
    if tag != 0x30 {
        return Err(OfdError::Conversion("OCSP: bad SingleResponse".into()));
    }

    // CertID SEQUENCE -- skip
    let (tag, cid_len) = read_tl(response, &mut p)?;
    if tag != 0x30 {
        return Err(OfdError::Conversion("OCSP: bad CertID".into()));
    }
    p += cid_len;

    // CertStatus: [0]=good, [1]/[0xA1]=revoked, [2]=unknown
    if p >= response.len() {
        return Err(OfdError::Conversion("OCSP: missing certStatus".into()));
    }
    let (cs_tag, _) = read_tl(response, &mut p)?;
    Ok(cs_tag == 0x81 || cs_tag == 0xA1)
}

/// Perform a plain HTTP POST using `std::net::TcpStream`.
///
/// Only supports `http://` (no TLS). Returns the response body bytes.
fn http_post(url: &str, content_type: &str, body: &[u8]) -> OfdResult<Vec<u8>> {
    let rest = url
        .strip_prefix("http://")
        .ok_or_else(|| OfdError::Conversion(format!("OCSP URL must be http:// ({url})")))?;
    let (host_port, path) = rest
        .find('/')
        .map_or((rest, "/"), |i| (&rest[..i], &rest[i..]));
    let (host, port) = host_port
        .rsplit_once(':')
        .map_or((host_port, 80_u16), |(h, p)| {
            (h, p.parse::<u16>().unwrap_or(80))
        });

    let addr = format!("{host}:{port}");
    let mut stream = TcpStream::connect(&addr)
        .map_err(|e| OfdError::Conversion(format!("OCSP connect {addr}: {e}")))?;
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .map_err(|e| OfdError::Conversion(format!("OCSP timeout: {e}")))?;

    let header = format!(
        "POST {path} HTTP/1.1\r\nHost: {host}\r\n\
         Content-Type: {content_type}\r\n\
         Content-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    stream
        .write_all(header.as_bytes())
        .map_err(|e| OfdError::Conversion(format!("OCSP write: {e}")))?;
    stream
        .write_all(body)
        .map_err(|e| OfdError::Conversion(format!("OCSP write body: {e}")))?;

    let mut response = Vec::new();
    stream
        .read_to_end(&mut response)
        .map_err(|e| OfdError::Conversion(format!("OCSP read: {e}")))?;

    let header_end = response
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .ok_or_else(|| OfdError::Conversion("OCSP: no HTTP header end".into()))?;

    let headers = std::str::from_utf8(&response[..header_end])
        .map_err(|e| OfdError::Conversion(format!("OCSP headers: {e}")))?;
    let status_line = headers.lines().next().unwrap_or("");
    if !status_line.contains(" 200 ") {
        return Err(OfdError::Conversion(format!("OCSP HTTP: {status_line}")));
    }

    Ok(response[header_end + 4..].to_vec())
}

/// Internal OCSP implementation -- returns errors on failure.
fn ocsp_check_impl(
    ocsp_endpoint: &str,
    serial: &[u8],
    issuer_name_der: &[u8],
    issuer_spki_der: &[u8],
) -> OfdResult<bool> {
    let request_der = build_ocsp_request(serial, issuer_name_der, issuer_spki_der);
    let response_bytes = http_post(ocsp_endpoint, "application/ocsp-request", &request_der)?;
    parse_ocsp_response(&response_bytes)
}

/// Check certificate revocation via OCSP (RFC 6960).
///
/// Constructs an OCSP request for the certificate identified by `serial`,
/// sends it via HTTP POST to `ocsp_endpoint`, and parses the response.
///
/// `issuer_name_der` is the DER-encoded issuer distinguished name.
/// `issuer_spki_der` is the DER-encoded SubjectPublicKeyInfo of the issuer.
///
/// Returns `Ok(true)` if the certificate is **revoked**, `Ok(false)` if not
/// revoked or if the check cannot be performed (network error, parse error,
/// unsupported endpoint). This conservative approach avoids false positives.
///
/// # Limitations
///
/// - Only supports plain HTTP endpoints (no TLS/HTTPS).
/// - Does not verify the OCSP response signature.
/// - Uses SHA-1 for CertID hashing (RFC 6960 default).
#[allow(dead_code)]
pub fn ocsp_check_with_endpoint(
    ocsp_endpoint: &str,
    serial: &[u8],
    issuer_name_der: &[u8],
    issuer_spki_der: &[u8],
) -> OfdResult<bool> {
    // Any error -> Ok(false): conservative, never blocks verification.
    match ocsp_check_impl(ocsp_endpoint, serial, issuer_name_der, issuer_spki_der) {
        Ok(revoked) => Ok(revoked),
        Err(_) => Ok(false),
    }
}

// ═══════════════════════════════════════════════════════════════════════
// ocsp_check_full — issuer-cert-aware entry point
// ═══════════════════════════════════════════════════════════════════════

/// Parse an issuer X.509 certificate DER and extract the two byte slices
/// needed for OCSP CertID construction:
///
/// 1. `issuer_name_der`  -- DER-encoded issuer distinguished name.
/// 2. `issuer_spki_der`  -- DER-encoded `SubjectPublicKeyInfo`.
#[allow(dead_code)]
fn extract_ocsp_issuer_fields(issuer_cert_der: &[u8]) -> OfdResult<(Vec<u8>, Vec<u8>)> {
    let cert = Certificate::from_der(issuer_cert_der)
        .map_err(|e| OfdError::Conversion(format!("Issuer cert parse error: {e}")))?;
    let tbs = cert.tbs_certificate();
    let name_der = tbs
        .issuer()
        .to_der()
        .map_err(|e| OfdError::Conversion(format!("Issuer DN encode error: {e}")))?;
    let spki_der = tbs
        .subject_public_key_info()
        .to_der()
        .map_err(|e| OfdError::Conversion(format!("SPKI encode error: {e}")))?;
    Ok((name_der, spki_der))
}

/// Check certificate revocation via OCSP (RFC 6960).
///
/// Constructs an OCSP CertID from `serial` and the issuer certificate
/// (`issuer_cert_der`), sends it via HTTP POST to `ocsp_url`, and returns
/// the revocation status.
///
/// When `ocsp_url` is `None`, returns `Ok(false)` (conservative default --
/// certificate assumed not revoked).
///
/// Returns `Ok(true)` if the certificate is **revoked**, `Ok(false)` if
/// not revoked or when the OCSP check cannot be performed (network error,
/// non-`http://` endpoint, parse failure).
///
/// Returns `Err` only when `issuer_cert_der` cannot be parsed as valid
/// X.509 (programming error / corrupted input).
///
/// # Limitations
///
/// - Only supports plain HTTP endpoints (no TLS/HTTPS).
/// - Does not verify the OCSP response signature.
/// - Uses SHA-1 for CertID hashing (RFC 6960 default).
#[allow(dead_code)]
pub fn ocsp_check_full(
    serial: &[u8],
    issuer_cert_der: &[u8],
    ocsp_url: Option<&str>,
) -> OfdResult<bool> {
    let Some(url) = ocsp_url else {
        return Ok(false); // conservative: assume not revoked
    };
    let (name_der, spki_der) = extract_ocsp_issuer_fields(issuer_cert_der)?;
    ocsp_check_with_endpoint(url, serial, &name_der, &spki_der)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Datelike;

    #[test]
    fn check_revoked_finds_serial() {
        let crl = CrlInfo {
            issuer: "CN=TestCA".into(),
            revoked_serials: vec![vec![0x01, 0x02], vec![0x03, 0x04]],
            this_update: Utc::now(),
            next_update: Utc::now(),
        };
        assert!(check_revoked(&crl, &[0x01, 0x02]));
        assert!(check_revoked(&crl, &[0x03, 0x04]));
        assert!(!check_revoked(&crl, &[0x05, 0x06]));
    }

    #[test]
    fn check_revoked_empty_crl() {
        let crl = CrlInfo {
            issuer: "CN=TestCA".into(),
            revoked_serials: vec![],
            this_update: Utc::now(),
            next_update: Utc::now(),
        };
        assert!(!check_revoked(&crl, &[0x01]));
    }

    #[test]
    fn ocsp_check_always_not_revoked() {
        assert!(!ocsp_check(&[0x01, 0x02, 0x03]).unwrap());
        assert!(!ocsp_check(&[]).unwrap());
    }

    #[test]
    fn parse_crl_der_smoke() {
        // Smoke test: attempt to parse a minimal CRL DER.
        let garbage = &[0x30, 0x03, 0x01, 0x01, 0x00]; // not a valid CRL
        let result = parse_crl_der(garbage);
        assert!(result.is_err(), "garbage DER should fail CRL parsing");
    }

    /// Valid empty v2 CRL (no revoked certs) generated with OpenSSL.
    const TEST_CRL_DER: &[u8] = &[
        0x30, 0x82, 0x01, 0x7d, 0x30, 0x67, 0x02, 0x01, 0x01, 0x30, 0x0d, 0x06, 0x09, 0x2a, 0x86,
        0x48, 0x86, 0xf7, 0x0d, 0x01, 0x01, 0x0b, 0x05, 0x00, 0x30, 0x25, 0x31, 0x11, 0x30, 0x0f,
        0x06, 0x03, 0x55, 0x04, 0x03, 0x0c, 0x08, 0x54, 0x65, 0x73, 0x74, 0x43, 0x65, 0x72, 0x74,
        0x31, 0x10, 0x30, 0x0e, 0x06, 0x03, 0x55, 0x04, 0x0a, 0x0c, 0x07, 0x54, 0x65, 0x73, 0x74,
        0x4f, 0x72, 0x67, 0x17, 0x0d, 0x32, 0x36, 0x30, 0x38, 0x31, 0x30, 0x31, 0x33, 0x34, 0x39,
        0x30, 0x30, 0x5a, 0x17, 0x0d, 0x32, 0x36, 0x30, 0x39, 0x30, 0x39, 0x31, 0x33, 0x34, 0x39,
        0x30, 0x30, 0x5a, 0xa0, 0x0e, 0x30, 0x0c, 0x30, 0x0a, 0x06, 0x03, 0x55, 0x1d, 0x14, 0x04,
        0x03, 0x02, 0x01, 0x01, 0x30, 0x0d, 0x06, 0x09, 0x2a, 0x86, 0x48, 0x86, 0xf7, 0x0d, 0x01,
        0x01, 0x0b, 0x05, 0x00, 0x03, 0x82, 0x01, 0x01, 0x00, 0x09, 0x5f, 0x86, 0xc6, 0x18, 0x45,
        0x27, 0xd2, 0x3d, 0xc8, 0x36, 0xac, 0xbc, 0xb9, 0x57, 0x79, 0x85, 0x8d, 0x7e, 0xb0, 0xa4,
        0xb5, 0x6b, 0xb9, 0x7d, 0xc3, 0xff, 0xf7, 0x49, 0x99, 0xba, 0x78, 0xbe, 0xd1, 0xee, 0x53,
        0x85, 0x9a, 0xb6, 0xc0, 0xc4, 0xe6, 0x3e, 0x53, 0x65, 0x0c, 0xf7, 0xab, 0xa3, 0x80, 0xb7,
        0xfa, 0x7d, 0xf3, 0xd1, 0x29, 0xc3, 0xd1, 0x40, 0x25, 0x8a, 0xbf, 0xb6, 0xf2, 0xad, 0x0a,
        0x42, 0xe3, 0xa2, 0xb4, 0x53, 0xb0, 0x57, 0x06, 0x88, 0x33, 0xb5, 0xc7, 0x08, 0xa8, 0x22,
        0xef, 0x2c, 0xd6, 0xd0, 0x18, 0xb8, 0x40, 0x33, 0x0e, 0xa9, 0x92, 0x6c, 0x68, 0xc8, 0x9e,
        0xf9, 0x30, 0xee, 0xff, 0x52, 0x05, 0x3b, 0x86, 0x47, 0x43, 0x3d, 0x24, 0xa1, 0xd5, 0xbe,
        0x16, 0xe0, 0x85, 0x64, 0xbe, 0x2b, 0x51, 0xc5, 0x04, 0xf5, 0x7b, 0x69, 0x2d, 0xbe, 0x45,
        0x6e, 0x93, 0xd1, 0x4b, 0x05, 0x3d, 0x8a, 0xf6, 0x82, 0xf8, 0xfd, 0x57, 0x00, 0x0f, 0xd8,
        0x9e, 0x36, 0x6a, 0x22, 0x6d, 0x0d, 0x91, 0x2f, 0x87, 0x97, 0x79, 0x3f, 0xb9, 0x8a, 0xdc,
        0xf0, 0x4a, 0x36, 0xd0, 0x05, 0xa0, 0x56, 0x53, 0xf5, 0x80, 0x7e, 0x59, 0x5c, 0x52, 0x28,
        0x76, 0xfe, 0x33, 0x1b, 0x8a, 0xb5, 0xe0, 0x6b, 0x6d, 0xa8, 0xf0, 0xb5, 0xf2, 0x45, 0x82,
        0xe7, 0xb0, 0xe2, 0xc3, 0x45, 0x1e, 0x45, 0xdb, 0x43, 0xf2, 0x11, 0x4b, 0xab, 0x57, 0x0b,
        0x80, 0x8f, 0xda, 0xea, 0xc3, 0x0e, 0x28, 0xd0, 0x37, 0x40, 0xec, 0x04, 0x13, 0x56, 0xe7,
        0x97, 0xc0, 0xf7, 0x24, 0xaa, 0x64, 0x22, 0xa9, 0xfa, 0x43, 0x2f, 0xeb, 0xfa, 0x5f, 0xc4,
        0x3a, 0xab, 0xf0, 0xf2, 0xd9, 0xfb, 0x37, 0x76, 0x4d, 0xbe, 0xe2, 0x12, 0x0d, 0x87, 0x06,
        0xe6, 0xa1, 0x08, 0x6e, 0x89, 0x27, 0x6f, 0xc1, 0x1d, 0xc1,
    ];

    #[test]
    fn parse_crl_der_valid_empty() {
        let crl = parse_crl_der(TEST_CRL_DER).expect("valid CRL should parse");
        // DN field ordering depends on the x509-cert crate version.
        assert!(crl.issuer.contains("CN=TestCert"), "issuer: {}", crl.issuer);
        assert!(crl.issuer.contains("O=TestOrg"), "issuer: {}", crl.issuer);
        assert!(crl.revoked_serials.is_empty());
        assert!(crl.this_update < crl.next_update);
    }

    #[test]
    fn parse_crl_der_check_not_revoked() {
        let crl = parse_crl_der(TEST_CRL_DER).expect("valid CRL should parse");
        assert!(!check_revoked(&crl, &[0x01, 0x02, 0x03]));
    }

    /// CRL with one revoked certificate (serial=0x01).
    const TEST_CRL_REVOKED_DER: &[u8] = &[
        0x30, 0x82, 0x01, 0x93, 0x30, 0x7d, 0x02, 0x01, 0x01, 0x30, 0x0d, 0x06, 0x09, 0x2a, 0x86,
        0x48, 0x86, 0xf7, 0x0d, 0x01, 0x01, 0x0b, 0x05, 0x00, 0x30, 0x25, 0x31, 0x11, 0x30, 0x0f,
        0x06, 0x03, 0x55, 0x04, 0x03, 0x0c, 0x08, 0x54, 0x65, 0x73, 0x74, 0x43, 0x65, 0x72, 0x74,
        0x31, 0x10, 0x30, 0x0e, 0x06, 0x03, 0x55, 0x04, 0x0a, 0x0c, 0x07, 0x54, 0x65, 0x73, 0x74,
        0x4f, 0x72, 0x67, 0x17, 0x0d, 0x32, 0x36, 0x30, 0x38, 0x31, 0x30, 0x31, 0x33, 0x35, 0x32,
        0x30, 0x31, 0x5a, 0x17, 0x0d, 0x32, 0x36, 0x30, 0x39, 0x30, 0x39, 0x31, 0x33, 0x35, 0x32,
        0x30, 0x31, 0x5a, 0x30, 0x14, 0x30, 0x12, 0x02, 0x01, 0x01, 0x17, 0x0d, 0x32, 0x36, 0x30,
        0x38, 0x31, 0x30, 0x31, 0x33, 0x34, 0x39, 0x30, 0x30, 0x5a, 0xa0, 0x0e, 0x30, 0x0c, 0x30,
        0x0a, 0x06, 0x03, 0x55, 0x1d, 0x14, 0x04, 0x03, 0x02, 0x01, 0x01, 0x30, 0x0d, 0x06, 0x09,
        0x2a, 0x86, 0x48, 0x86, 0xf7, 0x0d, 0x01, 0x01, 0x0b, 0x05, 0x00, 0x03, 0x82, 0x01, 0x01,
        0x00, 0x6c, 0xeb, 0x62, 0x59, 0x82, 0x0c, 0x4a, 0x6a, 0x26, 0x03, 0xdc, 0xc4, 0xc4, 0x53,
        0xef, 0x45, 0xbf, 0xd3, 0x9e, 0x28, 0xba, 0x4e, 0xd5, 0xb1, 0x23, 0x1f, 0xe4, 0x12, 0xdc,
        0x46, 0xc6, 0xdb, 0x35, 0xdb, 0xc5, 0x5b, 0x1c, 0x4f, 0x1b, 0x74, 0x8c, 0xa8, 0x05, 0x28,
        0x95, 0x83, 0x93, 0x28, 0xec, 0xb8, 0xb0, 0x8b, 0x35, 0x79, 0x81, 0xfd, 0xd6, 0xf7, 0x50,
        0x5d, 0x91, 0x89, 0x0e, 0xba, 0xe5, 0x8e, 0x43, 0x8c, 0x00, 0x24, 0xf8, 0xb9, 0x78, 0x2e,
        0x59, 0xe8, 0x5e, 0xe4, 0xae, 0x62, 0xbb, 0xb4, 0x94, 0x2e, 0x79, 0x3d, 0xe8, 0x46, 0x6f,
        0x0d, 0xb4, 0x3e, 0x1c, 0x2f, 0x06, 0xcf, 0x2c, 0xba, 0x53, 0x9c, 0xd1, 0xff, 0xe4, 0x53,
        0x4c, 0x05, 0xc4, 0x4f, 0xad, 0x1e, 0xec, 0xd0, 0xce, 0x48, 0x60, 0x1c, 0x07, 0x43, 0x96,
        0x18, 0x79, 0x1e, 0x92, 0x38, 0x71, 0x11, 0xd9, 0x94, 0x75, 0x81, 0x0b, 0xde, 0xf2, 0xe8,
        0xfb, 0x3d, 0xf5, 0x54, 0xae, 0x27, 0x77, 0xef, 0x1c, 0x4a, 0xfe, 0xcd, 0x0a, 0xd1, 0xbf,
        0xa3, 0xee, 0x7f, 0x82, 0xa3, 0xfc, 0x85, 0xc4, 0x93, 0x3e, 0x96, 0xb2, 0x1d, 0x64, 0xb8,
        0xbd, 0xe8, 0x64, 0x7b, 0x03, 0x72, 0x49, 0x5c, 0xbc, 0x79, 0x9a, 0x92, 0x6c, 0x95, 0x7a,
        0x03, 0xf7, 0x7a, 0xfa, 0x5b, 0xe8, 0xee, 0xca, 0x97, 0x42, 0xb0, 0x79, 0x72, 0x1a, 0x4e,
        0x24, 0x24, 0x21, 0x9a, 0xf7, 0x5c, 0xa3, 0x93, 0xf0, 0x51, 0x25, 0xe4, 0x65, 0xfb, 0x02,
        0x9c, 0x1a, 0x6c, 0xdc, 0xb6, 0x9b, 0xab, 0x11, 0xd9, 0xf5, 0xa3, 0xd2, 0x12, 0xd3, 0xeb,
        0x47, 0x84, 0x13, 0xc8, 0x75, 0xdb, 0xf2, 0xcc, 0xff, 0x08, 0x35, 0x6e, 0xe2, 0x88, 0x90,
        0x5e, 0xc8, 0xaf, 0xba, 0xdb, 0x92, 0x91, 0x3f, 0x06, 0xdf, 0xec, 0x1b, 0xbc, 0x1f, 0xe0,
        0x0f, 0xf6,
    ];

    #[test]
    fn parse_crl_der_with_revoked_cert() {
        let crl = parse_crl_der(TEST_CRL_REVOKED_DER).expect("revoked CRL should parse");
        assert!(crl.issuer.contains("CN=TestCert"), "issuer: {}", crl.issuer);
        assert_eq!(crl.revoked_serials.len(), 1, "expected 1 revoked cert");
        assert_eq!(crl.revoked_serials[0], vec![0x01]);
    }

    #[test]
    fn check_revoked_finds_serial_in_revoked_crl() {
        let crl = parse_crl_der(TEST_CRL_REVOKED_DER).expect("revoked CRL should parse");
        assert!(check_revoked(&crl, &[0x01]));
        assert!(!check_revoked(&crl, &[0x02]));
    }

    #[test]
    fn parse_crl_der_truncated_input() {
        // SEQUENCE tag + length but not enough data.
        let bad = &[0x30, 0x82, 0x01, 0x00];
        assert!(parse_crl_der(bad).is_err());
    }

    #[test]
    fn parse_crl_der_single_byte_input() {
        assert!(parse_crl_der(&[0x30]).is_err());
    }

    #[test]
    fn check_revoked_exact_match_only() {
        let crl = CrlInfo {
            issuer: "CN=CA".into(),
            revoked_serials: vec![vec![0x0A, 0x0B]],
            this_update: Utc::now(),
            next_update: Utc::now(),
        };
        // Prefix match must not count as revoked.
        assert!(!check_revoked(&crl, &[0x0A]));
        assert!(check_revoked(&crl, &[0x0A, 0x0B]));
    }

    #[test]
    fn ocsp_check_with_empty_serial() {
        assert!(!ocsp_check(&[]).unwrap());
    }

    #[test]
    fn crl_info_fields_after_parse() {
        let crl = parse_crl_der(TEST_CRL_DER).expect("valid CRL should parse");
        assert!(crl.issuer.contains("TestCert"));
        assert!(crl.this_update.year() >= 2026);
        assert!(crl.next_update > crl.this_update);
    }

    // ═══════════════════════════════════════════════════════════════════
    // OCSP implementation tests
    // ═══════════════════════════════════════════════════════════════════

    #[test]
    fn sha1_known_hash() {
        // SHA-1("") = da39a3ee5e6b4b0d3255bfef95601890afd80709
        assert_eq!(
            sha1(b""),
            [
                0xda, 0x39, 0xa3, 0xee, 0x5e, 0x6b, 0x4b, 0x0d, 0x32, 0x55, 0xbf, 0xef, 0x95, 0x60,
                0x18, 0x90, 0xaf, 0xd8, 0x07, 0x09,
            ]
        );
        // SHA-1("abc") = a9993e364706816aba3e25717850c26c9cd0d89d
        assert_eq!(
            sha1(b"abc"),
            [
                0xa9, 0x99, 0x3e, 0x36, 0x47, 0x06, 0x81, 0x6a, 0xba, 0x3e, 0x25, 0x71, 0x78, 0x50,
                0xc2, 0x6c, 0x9c, 0xd0, 0xd8, 0x9d,
            ]
        );
    }

    #[test]
    fn der_integer_positive_strips_leading_zeros() {
        assert_eq!(der_integer_positive(&[0x00, 0x01]), vec![0x02, 0x01, 0x01]);
        assert_eq!(
            der_integer_positive(&[0x00, 0x00, 0xFF]),
            vec![0x02, 0x02, 0x00, 0xFF]
        );
        assert_eq!(der_integer_positive(&[]), vec![0x02, 0x01, 0x00]);
        assert_eq!(der_integer_positive(&[0x00]), vec![0x02, 0x01, 0x00]);
    }

    #[test]
    fn build_ocsp_request_produces_valid_der() {
        let request = build_ocsp_request(&[0x01, 0x02], &[0x30, 0x00], &[0x30, 0x00]);
        assert_eq!(request[0], 0x30, "must start with SEQUENCE");
        let mut pos = 0;
        let (tag, _) = read_tl(&request, &mut pos).unwrap();
        assert_eq!(tag, 0x30);
    }

    /// Build a minimal OCSP response with the given certStatus tag and value.
    fn build_test_ocsp_response(status_tag: u8, status_value: &[u8]) -> Vec<u8> {
        let sha1_oid: &[u8] = &[0x06, 0x05, 0x2B, 0x0E, 0x03, 0x02, 0x1A];
        let null_bytes: &[u8] = &[0x05, 0x00];
        let mut alg_val = Vec::new();
        alg_val.extend_from_slice(sha1_oid);
        alg_val.extend_from_slice(null_bytes);
        let alg_id = der_sequence(&alg_val);

        let mut cid_val = Vec::new();
        cid_val.extend_from_slice(&alg_id);
        cid_val.extend_from_slice(&der_octet_string(&[0u8; 20]));
        cid_val.extend_from_slice(&der_octet_string(&[0u8; 20]));
        cid_val.extend_from_slice(&der_integer_positive(&[0x01]));
        let cert_id = der_sequence(&cid_val);

        let cert_status = der_wrap(status_tag, status_value);
        let this_update = der_wrap(0x18, b"260810000000Z");

        let mut sr_val = Vec::new();
        sr_val.extend_from_slice(&cert_id);
        sr_val.extend_from_slice(&cert_status);
        sr_val.extend_from_slice(&this_update);
        let single_response = der_sequence(&sr_val);
        let responses = der_sequence(&single_response);

        let responder_id = der_wrap(0xA2, &der_octet_string(&[0u8; 20]));
        let produced_at = der_wrap(0x18, b"260810000000Z");

        let mut rd_val = Vec::new();
        rd_val.extend_from_slice(&responder_id);
        rd_val.extend_from_slice(&produced_at);
        rd_val.extend_from_slice(&responses);
        let response_data = der_sequence(&rd_val);

        let mut sig_alg_val = Vec::new();
        sig_alg_val.extend_from_slice(sha1_oid);
        sig_alg_val.extend_from_slice(null_bytes);
        let sig_alg = der_sequence(&sig_alg_val);
        let signature = der_wrap(0x03, &[0x00, 0x40]);

        let mut bor_val = Vec::new();
        bor_val.extend_from_slice(&response_data);
        bor_val.extend_from_slice(&sig_alg);
        bor_val.extend_from_slice(&signature);
        let basic_response = der_sequence(&bor_val);
        let response_octet = der_octet_string(&basic_response);

        let ocsp_basic_oid: &[u8] = &[
            0x06, 0x09, 0x2B, 0x06, 0x01, 0x05, 0x05, 0x07, 0x30, 0x01, 0x01,
        ];
        let mut resp_bytes_val = Vec::new();
        resp_bytes_val.extend_from_slice(ocsp_basic_oid);
        resp_bytes_val.extend_from_slice(&response_octet);
        let response_bytes = der_sequence(&resp_bytes_val);
        let response_bytes_explicit = der_wrap(0xA0, &response_bytes);
        let response_status = der_wrap(0x0A, &[0x00]);

        let mut ocsp_val = Vec::new();
        ocsp_val.extend_from_slice(&response_status);
        ocsp_val.extend_from_slice(&response_bytes_explicit);
        der_sequence(&ocsp_val)
    }

    #[test]
    fn parse_ocsp_response_good() {
        let response = build_test_ocsp_response(0x80, &[]);
        assert!(!parse_ocsp_response(&response).unwrap());
    }

    #[test]
    fn parse_ocsp_response_revoked() {
        // RevokedInfo ::= SEQUENCE { revocationTime GeneralizedTime }
        let rev_time = der_wrap(0x18, b"260810000000Z");
        let revoked_info = der_sequence(&rev_time);
        let response = build_test_ocsp_response(0xA1, &revoked_info);
        assert!(parse_ocsp_response(&response).unwrap());
    }

    #[test]
    fn parse_ocsp_response_unknown() {
        let response = build_test_ocsp_response(0x82, &[]);
        assert!(!parse_ocsp_response(&response).unwrap());
    }

    #[test]
    fn parse_ocsp_response_malformed() {
        assert!(parse_ocsp_response(&[]).is_err());
        assert!(parse_ocsp_response(&[0x30, 0x00]).is_err());
    }

    #[test]
    fn parse_ocsp_response_non_successful_status() {
        // responseStatus = 2 (internalError)
        let response_status = der_wrap(0x0A, &[0x02]);
        let mut val = Vec::new();
        val.extend_from_slice(&response_status);
        let response = der_sequence(&val);
        assert!(!parse_ocsp_response(&response).unwrap());
    }

    #[test]
    fn ocsp_check_with_endpoint_unreachable() {
        let result = ocsp_check_with_endpoint(
            "http://127.0.0.1:1/ocsp",
            &[0x01],
            &[0x30, 0x00],
            &[0x30, 0x00],
        );
        assert!(
            !result.unwrap(),
            "unreachable endpoint should return Ok(false)"
        );
    }

    #[test]
    fn ocsp_check_with_endpoint_bad_url() {
        let result = ocsp_check_with_endpoint(
            "https://example.com/ocsp",
            &[0x01],
            &[0x30, 0x00],
            &[0x30, 0x00],
        );
        assert!(!result.unwrap(), "https URL should return Ok(false)");
    }

    /// Read a full HTTP/1.1 request from `stream`, returning the body bytes.
    /// Handles Content-Length to know when the body ends.
    fn mock_read_http_request(stream: &TcpStream) -> Vec<u8> {
        use std::io::BufRead;
        let mut reader = std::io::BufReader::new(stream);
        let mut content_length: usize = 0;
        loop {
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            if line == "\r\n" || line.is_empty() {
                break;
            }
            if let Some(val) = line.strip_prefix("Content-Length:") {
                content_length = val.trim().parse().unwrap_or(0);
            }
        }
        let mut body = vec![0u8; content_length];
        if content_length > 0 {
            reader.read_exact(&mut body).unwrap();
        }
        body
    }

    #[test]
    fn ocsp_check_with_endpoint_mock_server() {
        use std::net::TcpListener;
        use std::thread;

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let url = format!("http://127.0.0.1:{port}/ocsp");

        let handle = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            mock_read_http_request(&stream);

            let ocsp_response = build_test_ocsp_response(0x80, &[]);
            let header = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/ocsp-response\r\n\
                 Content-Length: {}\r\nConnection: close\r\n\r\n",
                ocsp_response.len()
            );
            stream.write_all(header.as_bytes()).unwrap();
            stream.write_all(&ocsp_response).unwrap();
            stream.flush().unwrap();
        });

        let result = ocsp_check_with_endpoint(&url, &[0x01], &[0x30, 0x00], &[0x30, 0x00]);
        assert!(!result.unwrap(), "good status should return Ok(false)");
        handle.join().unwrap();
    }

    #[test]
    fn ocsp_check_with_endpoint_mock_server_revoked() {
        use std::net::TcpListener;
        use std::thread;

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let url = format!("http://127.0.0.1:{port}/ocsp");

        let handle = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            mock_read_http_request(&stream);

            let rev_time = der_wrap(0x18, b"260810000000Z");
            let revoked_info = der_sequence(&rev_time);
            let ocsp_response = build_test_ocsp_response(0xA1, &revoked_info);
            let header = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/ocsp-response\r\n\
                 Content-Length: {}\r\nConnection: close\r\n\r\n",
                ocsp_response.len()
            );
            stream.write_all(header.as_bytes()).unwrap();
            stream.write_all(&ocsp_response).unwrap();
            stream.flush().unwrap();
        });

        let result = ocsp_check_with_endpoint(&url, &[0x01], &[0x30, 0x00], &[0x30, 0x00]);
        assert!(result.unwrap(), "revoked status should return Ok(true)");
        handle.join().unwrap();
    }

    // ═══════════════════════════════════════════════════════════════════
    // ocsp_check_full tests
    // ═══════════════════════════════════════════════════════════════════

    /// Self-signed RSA 2048 X.509 certificate (CN=TestCert, O=TestOrg).
    /// Reused from cert.rs tests for OCSP CertID construction verification.
    const OCSP_TEST_CERT_PEM: &str = concat!(
        "-----BEGIN CERTIFICATE-----\n",
        "MIIDKzCCAhOgAwIBAgIUdCE8I095XmtRZqsbOmt4u+oqg/cwDQYJKoZIhvcNAQEL\n",
        "BQAwJTERMA8GA1UEAwwIVGVzdENlcnQxEDAOBgNVBAoMB1Rlc3RPcmcwHhcNMjYw\n",
        "ODEwMTMzNjAwWhcNMzYwODA3MTMzNjAwWjAlMREwDwYDVQQDDAhUZXN0Q2VydDEQ\n",
        "MA4GA1UECgwHVGVzdE9yZzCCASIwDQYJKoZIhvcNAQEBBQADggEPADCCAQoCggEB\n",
        "AJAUqZdgNuPf2BjtDkFJy2acVaXWnl6X8rDlvnLCuLWsCPE67nZJFFw1Mge/WyaD\n",
        "Ps+e/UUeo/O3lfi8s6zICR7NCzV0f1KVFHw4oPGOqyLoHftYB8o97B5haC9SmmbD\n",
        "VyHbDw++8zDqHk03UKooSrglQv1kap3Kix6JD/ZV8YVNTynYevR7/Mxp31U5lQ19\n",
        "jOBPF39+KTxO9lRiqIl7fp0ZP+QO/bCEBUdP6t8P6sWTPyPrdPUmUSjg+JCqNnDy\n",
        "cGiP2loTSIdDwtoV2FNhTLlZShO9dSGIaBHPgryeYPd6CofzF4OmE9c8dpDpq3bT\n",
        "/Z9Uo1wcx2Lq6JdOecAZSRcCAwEAAaNTMFEwHQYDVR0OBBYEFDmaH8gZQ5TEorRY\n",
        "8TooyQVBKgG6MB8GA1UdIwQYMBaAFDmaH8gZQ5TEorRY8TooyQVBKgG6MA8GA1Ud\n",
        "EwEB/wQFMAMBAf8wDQYJKoZIhvcNAQELBQADggEBAIlZI7gfpVRni0fdevifAouS\n",
        "I3Gz2bedWOzixeouYSsuuaxplrDgwWdDporihfIWSYjC+Yv4EJn0k3DvnP8ZCCfb\n",
        "eLiYaWmKSiEqzGbQ6fiODYH1fuhiMs3wA8Xd0ChFYlcvYTIHHcJI+s2U3JanUPJF\n",
        "I/A1bFFDV0LBE7oOSzNSroWhGSUip7mU9S8KcTdv2MrVSTy5SGNkzDAHO8ZPtnWN\n",
        "Dv34h7Ee6wJo2j68qv8k3L+YZH4/SoGNMf4e0/Zz5eH5uCjFY0qofEaYM+e5Xwz+\n",
        "QCVp1Lzu+cm1z8xC9cK738hV2ZNNZLQX89TIgcfaNJu6POEISvvtTJEFrNwtWi4=\n",
        "-----END CERTIFICATE-----"
    );

    #[test]
    fn ocsp_check_full_none_url_returns_not_revoked() {
        let pem = pem::parse(OCSP_TEST_CERT_PEM).expect("PEM");
        let der = pem.contents();
        assert!(
            !ocsp_check_full(&[0x01, 0x02], der, None).unwrap(),
            "None URL must return Ok(false)"
        );
    }

    #[test]
    fn ocsp_check_full_invalid_cert_returns_error() {
        let result = ocsp_check_full(
            &[0x01],
            &[0x00, 0x01, 0x02],
            Some("http://127.0.0.1:1/ocsp"),
        );
        assert!(result.is_err(), "invalid cert DER must return Err");
    }

    #[test]
    fn extract_ocsp_issuer_fields_from_valid_cert() {
        let pem = pem::parse(OCSP_TEST_CERT_PEM).expect("PEM");
        let der = pem.contents();
        let (name_der, spki_der) = extract_ocsp_issuer_fields(der).unwrap();
        // Both must be DER SEQUENCEs.
        assert_eq!(
            name_der[0], 0x30,
            "issuer name DER must start with SEQUENCE"
        );
        assert_eq!(spki_der[0], 0x30, "SPKI DER must start with SEQUENCE");
        assert!(!name_der.is_empty());
        assert!(!spki_der.is_empty());
        assert_ne!(name_der, spki_der, "name and SPKI must differ");
    }

    #[test]
    fn ocsp_request_der_from_real_cert_has_valid_structure() {
        let pem = pem::parse(OCSP_TEST_CERT_PEM).expect("PEM");
        let der = pem.contents();
        let (name_der, spki_der) = extract_ocsp_issuer_fields(der).unwrap();
        let request = build_ocsp_request(&[0x01, 0x02, 0x03], &name_der, &spki_der);

        // OCSPRequest is a SEQUENCE.
        assert_eq!(request[0], 0x30);

        // Walk the structure: OCSPRequest -> TBSRequest -> requestList -> Request -> CertID.
        let mut pos = 0;
        // OCSPRequest
        let (tag, _) = read_tl(&request, &mut pos).unwrap();
        assert_eq!(tag, 0x30, "outer SEQUENCE");
        // TBSRequest
        let (tag, _) = read_tl(&request, &mut pos).unwrap();
        assert_eq!(tag, 0x30, "TBSRequest");
        // requestList
        let (tag, _) = read_tl(&request, &mut pos).unwrap();
        assert_eq!(tag, 0x30, "requestList");
        // Request
        let (tag, _) = read_tl(&request, &mut pos).unwrap();
        assert_eq!(tag, 0x30, "Request");
        // CertID
        let (tag, cid_len) = read_tl(&request, &mut pos).unwrap();
        assert_eq!(tag, 0x30, "CertID");
        assert!(cid_len > 0, "CertID must have content");

        // CertID contains: alg_id (SEQUENCE), nameHash (OCTET STRING),
        // keyHash (OCTET STRING), serial (INTEGER).
        // Verify the two hashes are 20 bytes each (SHA-1).
        // alg_id SEQUENCE
        let (tag, alg_len) = read_tl(&request, &mut pos).unwrap();
        assert_eq!(tag, 0x30, "CertID alg_id");
        pos += alg_len; // skip algorithm identifier
        // issuerNameHash
        let (tag, nh_len) = read_tl(&request, &mut pos).unwrap();
        assert_eq!(tag, 0x04, "issuerNameHash OCTET STRING");
        assert_eq!(nh_len, 20, "SHA-1 hash must be 20 bytes");
    }

    #[test]
    fn ocsp_check_full_unreachable_endpoint_returns_not_revoked() {
        let pem = pem::parse(OCSP_TEST_CERT_PEM).expect("PEM");
        let der = pem.contents();
        // Port 1 is almost certainly unreachable.
        let result = ocsp_check_full(&[0x01], der, Some("http://127.0.0.1:1/ocsp"));
        assert!(
            !result.unwrap(),
            "unreachable endpoint should return Ok(false)"
        );
    }
}
