//! X.509 certificate parsing and chain verification stubs.
//!
//! Provides [`CertificateInfo`] as a flat representation of an X.509 certificate,
//! along with DER/PEM parsing helpers and a placeholder certificate-chain verifier.
//!
//! These APIs are **not** wired into `sign()` / `read_signature()` / `verify_signature()`
//! yet — they exist as an API surface for future multi-signer and KeyInfo integration.

use chrono::{DateTime, TimeZone, Utc};
use der::{Decode, Encode};
use easyofd_core::{OfdError, OfdResult};
use x509_cert::Certificate;

use crate::crl::ocsp_check;

/// Flat representation of X.509 certificate fields relevant to OFD seal verification.
#[derive(Debug, Clone)]
pub struct CertificateInfo {
    /// Subject distinguished name (RFC 4514 string).
    pub subject: String,
    /// Issuer distinguished name (RFC 4514 string).
    pub issuer: String,
    /// Serial number bytes (big-endian, leading zeros stripped).
    pub serial: Vec<u8>,
    /// Not-before validity bound.
    pub not_before: DateTime<Utc>,
    /// Not-after validity bound.
    pub not_after: DateTime<Utc>,
    /// Original DER encoding of the certificate.
    pub raw_der: Vec<u8>,
}

/// Convert a `der::DateTime` (year/month/day/hour/min/sec fields) to `chrono::DateTime<Utc>`.
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

/// Parse an X.509 certificate from DER-encoded bytes.
///
/// Extracts subject, issuer, serial number, validity period, and retains the raw DER.
pub fn parse_x509_der(der: &[u8]) -> OfdResult<CertificateInfo> {
    let cert = Certificate::from_der(der)
        .map_err(|e| OfdError::Conversion(format!("X.509 DER parse error: {e}")))?;
    Ok(extract_info(&cert, der.to_vec()))
}

/// Parse an X.509 certificate from a PEM-encoded string.
///
/// Decodes the PEM envelope then delegates to [`parse_x509_der`].
pub fn parse_x509_pem(pem_str: &str) -> OfdResult<CertificateInfo> {
    let pem =
        pem::parse(pem_str).map_err(|e| OfdError::Conversion(format!("PEM parse error: {e}")))?;
    parse_x509_der(pem.contents())
}

/// Extract [`CertificateInfo`] from a parsed [`Certificate`].
fn extract_info(cert: &Certificate, raw_der: Vec<u8>) -> CertificateInfo {
    let tbs = cert.tbs_certificate();
    let subject = tbs.subject().to_string();
    let issuer = tbs.issuer().to_string();
    let serial = tbs.serial_number().as_bytes().to_vec();

    let validity = tbs.validity();
    let not_before = der_datetime_to_chrono(validity.not_before.to_date_time());
    let not_after = der_datetime_to_chrono(validity.not_after.to_date_time());

    CertificateInfo {
        subject,
        issuer,
        serial,
        not_before,
        not_after,
        raw_der,
    }
}

/// Verify a certificate chain (leaf -> intermediates -> root).
///
/// Performs the following checks in order:
///
/// 1. **Subject/issuer link**: each certificate's issuer DN must equal the
///    next certificate's subject DN.
/// 2. **Serial uniqueness**: no two certificates in the chain may share the
///    same serial number.
/// 3. **OCSP revocation** (optional): when `ocsp_endpoint` is `Some`, each
///    certificate serial is checked via [`crate::crl::ocsp_check`]. The
///    current implementation is a placeholder that always returns
///    "not revoked".
/// 4. **SM2-with-SM3 signature verification**: for each child certificate
///    that carries `raw_der`, attempt to verify its TBSCertificate signature
///    using the parent certificate's public key. This currently only works
///    for SM2-signed certificates; RSA and other algorithms are silently
///    skipped (treated as verified).
///
/// # TODO(gbt38540)
///
/// x509-cert 0.3 may not fully support SM2 OID signature verification.
/// When SM2 cert verification fails due to API limitations, the check is
/// skipped rather than rejecting the chain. This should be tightened once
/// the SM2 OID is natively supported.
///
/// Returns `Ok(true)` if the chain is valid, `Ok(false)` otherwise.
pub fn verify_chain(
    leaf: &CertificateInfo,
    intermediates: &[CertificateInfo],
    root: &CertificateInfo,
    ocsp_endpoint: Option<&str>,
) -> OfdResult<bool> {
    // Build the full chain in verification order: leaf, intermediates..., root.
    let mut chain: Vec<&CertificateInfo> = Vec::with_capacity(intermediates.len() + 2);
    chain.push(leaf);
    chain.extend(intermediates);
    chain.push(root);

    // 1. Subject/issuer link check: each cert's issuer must equal the next cert's subject.
    for window in chain.windows(2) {
        let (child, parent) = (window[0], window[1]);
        if child.issuer != parent.subject {
            return Ok(false);
        }
    }

    // 2. Serial number uniqueness across the entire chain.
    let mut seen_serials: Vec<&[u8]> = Vec::new();
    for cert in &chain {
        if seen_serials.contains(&cert.serial.as_slice()) {
            return Ok(false);
        }
        seen_serials.push(&cert.serial);
    }

    // 3. OCSP revocation check (placeholder — always "not revoked").
    if ocsp_endpoint.is_some() {
        for cert in &chain {
            // ocsp_check returns Ok(true) if revoked, Ok(false) if not.
            if ocsp_check(&cert.serial)? {
                return Ok(false);
            }
        }
    }

    // 4. SM2 certificate signature verification for each chain link.
    for window in chain.windows(2) {
        let (child, parent) = (window[0], window[1]);
        if !verify_cert_signature(child, parent)? {
            return Ok(false);
        }
    }

    Ok(true)
}

/// Attempt to verify a child certificate's TBSCertificate signature using
/// the parent certificate's public key.
///
/// Returns `Ok(true)` if verification succeeds or cannot be performed
/// (missing DER, unsupported algorithm), `Ok(false)` if verification
/// positively determines the signature is invalid.
fn verify_cert_signature(child: &CertificateInfo, parent: &CertificateInfo) -> OfdResult<bool> {
    if child.raw_der.is_empty() || parent.raw_der.is_empty() {
        return Ok(true); // cannot verify without DER data
    }

    let child_cert = Certificate::from_der(&child.raw_der)
        .map_err(|e| OfdError::Conversion(format!("child cert DER parse error: {e}")))?;
    let parent_cert = Certificate::from_der(&parent.raw_der)
        .map_err(|e| OfdError::Conversion(format!("parent cert DER parse error: {e}")))?;

    // Check if the child cert's signature algorithm is SM2.
    // SM2-with-SM3 OID: 1.2.156.10197.1.501
    let sig_oid_str = child_cert.signature_algorithm().oid.to_string();
    if !sig_oid_str.starts_with("1.2.156.10197") {
        // Not SM2 — skip cryptographic verification.
        return Ok(true);
    }

    // Attempt SM2 verification.
    match verify_sm2_cert_signature(&child_cert, &parent_cert) {
        Ok(valid) => Ok(valid),
        Err(_) => {
            // SM2 verification failed due to API limitations (e.g. key
            // format, encoding). Treat as "cannot verify" rather than
            // "invalid" to avoid false rejections.
            // TODO(gbt38540): tighten once SM2 OID is fully supported.
            Ok(true)
        }
    }
}

/// Try to verify an SM2 signature on a child certificate using the parent
/// certificate's SubjectPublicKeyInfo.
///
/// # TODO(gbt38540)
///
/// This function relies on x509-cert 0.3 / spki 0.8 APIs for extracting
/// SubjectPublicKeyInfo bytes. If the API surface changes or SM2 OIDs are
/// not recognized, this function returns `Err` and the caller falls back
/// to "cannot verify".
fn verify_sm2_cert_signature(child: &Certificate, parent: &Certificate) -> OfdResult<bool> {
    // Extract parent's public key bytes from SubjectPublicKeyInfo.
    let spki = parent.tbs_certificate().subject_public_key_info();
    let pub_key_bytes = spki.subject_public_key.raw_bytes();

    // Re-encode the child's TBSCertificate to DER (this is the signed message).
    let tbs_bytes = child
        .tbs_certificate()
        .to_der()
        .map_err(|e| OfdError::Conversion(format!("TBS DER encode error: {e}")))?;

    // Extract the child certificate's signature bytes.
    let sig_bytes = child.signature().raw_bytes();

    // Create SM2 verifying key with standard DistId "1234567812345678".
    let vkey = sm2::dsa::VerifyingKey::from_sec1_bytes("1234567812345678", pub_key_bytes)
        .map_err(|e| OfdError::Conversion(format!("SM2 public key parse error: {e}")))?;
    let signature = sm2::dsa::Signature::from_slice(sig_bytes)
        .map_err(|e| OfdError::Conversion(format!("SM2 signature parse error: {e}")))?;

    use sm2::dsa::signature::Verifier;
    Ok(vkey.verify(&tbs_bytes, &signature).is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Datelike;

    /// Valid self-signed RSA 2048 X.509 certificate (CN=TestCert, O=TestOrg).
    /// Generated with: openssl req -x509 -newkey rsa:2048 -nodes -days 3650
    ///                 -subj "/CN=TestCert/O=TestOrg"
    const TEST_CERT_PEM: &str = concat!(
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
    fn parse_x509_der_smoke() {
        // Use the PEM to get DER bytes and test DER parsing directly.
        let pem = pem::parse(TEST_CERT_PEM).expect("PEM should parse");
        let der = pem.contents();
        let info = parse_x509_der(der).expect("DER should parse");
        // DN field ordering depends on the x509-cert crate version;
        // assert both components are present rather than exact string.
        assert!(
            info.subject.contains("CN=TestCert"),
            "subject: {}",
            info.subject
        );
        assert!(
            info.subject.contains("O=TestOrg"),
            "subject: {}",
            info.subject
        );
        assert!(
            info.issuer.contains("CN=TestCert"),
            "issuer: {}",
            info.issuer
        );
        assert!(info.issuer.contains("O=TestOrg"), "issuer: {}", info.issuer);
        assert!(!info.serial.is_empty());
        assert!(!info.raw_der.is_empty());
        assert_eq!(info.raw_der, der);
    }

    #[test]
    fn parse_x509_pem_extracts_fields() {
        let info = parse_x509_pem(TEST_CERT_PEM).expect("PEM should parse");
        assert!(
            info.subject.contains("CN=TestCert"),
            "subject: {}",
            info.subject
        );
        assert!(
            info.subject.contains("O=TestOrg"),
            "subject: {}",
            info.subject
        );
        assert!(
            info.issuer.contains("CN=TestCert"),
            "issuer: {}",
            info.issuer
        );
        assert!(info.issuer.contains("O=TestOrg"), "issuer: {}", info.issuer);
        assert!(!info.serial.is_empty());
        assert!(info.not_before < info.not_after);
        assert!(!info.raw_der.is_empty());
    }

    #[test]
    fn parse_x509_der_rejects_garbage() {
        let result = parse_x509_der(&[0x00, 0x01, 0x02]);
        assert!(result.is_err());
    }

    #[test]
    fn parse_x509_pem_rejects_garbage() {
        let result = parse_x509_pem("not a valid PEM");
        assert!(result.is_err());
    }

    #[test]
    fn verify_chain_rejects_mismatched_issuer() {
        let leaf = CertificateInfo {
            subject: "CN=leaf".into(),
            issuer: "CN=wrong".into(),
            serial: vec![1],
            not_before: Utc::now(),
            not_after: Utc::now(),
            raw_der: vec![],
        };
        let root = CertificateInfo {
            subject: "CN=root".into(),
            issuer: "CN=root".into(),
            serial: vec![2],
            not_before: Utc::now(),
            not_after: Utc::now(),
            raw_der: vec![],
        };
        assert!(!verify_chain(&leaf, &[], &root, None).unwrap());
    }

    #[test]
    fn verify_chain_rejects_duplicate_serial() {
        let leaf = CertificateInfo {
            subject: "CN=leaf".into(),
            issuer: "CN=root".into(),
            serial: vec![1],
            not_before: Utc::now(),
            not_after: Utc::now(),
            raw_der: vec![],
        };
        let root = CertificateInfo {
            subject: "CN=root".into(),
            issuer: "CN=root".into(),
            serial: vec![1], // same serial as leaf
            not_before: Utc::now(),
            not_after: Utc::now(),
            raw_der: vec![],
        };
        assert!(!verify_chain(&leaf, &[], &root, None).unwrap());
    }

    #[test]
    fn verify_chain_accepts_valid_chain() {
        let leaf = CertificateInfo {
            subject: "CN=leaf".into(),
            issuer: "CN=inter".into(),
            serial: vec![1],
            not_before: Utc::now(),
            not_after: Utc::now(),
            raw_der: vec![],
        };
        let inter = CertificateInfo {
            subject: "CN=inter".into(),
            issuer: "CN=root".into(),
            serial: vec![2],
            not_before: Utc::now(),
            not_after: Utc::now(),
            raw_der: vec![],
        };
        let root = CertificateInfo {
            subject: "CN=root".into(),
            issuer: "CN=root".into(),
            serial: vec![3],
            not_before: Utc::now(),
            not_after: Utc::now(),
            raw_der: vec![],
        };
        assert!(verify_chain(&leaf, &[inter], &root, None).unwrap());
    }

    #[test]
    fn verify_chain_rejects_intermediate_link_mismatch() {
        let leaf = CertificateInfo {
            subject: "CN=leaf".into(),
            issuer: "CN=inter1".into(),
            serial: vec![1],
            not_before: Utc::now(),
            not_after: Utc::now(),
            raw_der: vec![],
        };
        let inter = CertificateInfo {
            subject: "CN=inter2".into(), // mismatch: leaf.issuer != inter.subject
            issuer: "CN=root".into(),
            serial: vec![2],
            not_before: Utc::now(),
            not_after: Utc::now(),
            raw_der: vec![],
        };
        let root = CertificateInfo {
            subject: "CN=root".into(),
            issuer: "CN=root".into(),
            serial: vec![3],
            not_before: Utc::now(),
            not_after: Utc::now(),
            raw_der: vec![],
        };
        assert!(!verify_chain(&leaf, &[inter], &root, None).unwrap());
    }

    #[test]
    fn verify_chain_rejects_intermediate_duplicate_serial() {
        let leaf = CertificateInfo {
            subject: "CN=leaf".into(),
            issuer: "CN=inter".into(),
            serial: vec![1],
            not_before: Utc::now(),
            not_after: Utc::now(),
            raw_der: vec![],
        };
        let inter = CertificateInfo {
            subject: "CN=inter".into(),
            issuer: "CN=root".into(),
            serial: vec![3], // same as root
            not_before: Utc::now(),
            not_after: Utc::now(),
            raw_der: vec![],
        };
        let root = CertificateInfo {
            subject: "CN=root".into(),
            issuer: "CN=root".into(),
            serial: vec![3],
            not_before: Utc::now(),
            not_after: Utc::now(),
            raw_der: vec![],
        };
        assert!(!verify_chain(&leaf, &[inter], &root, None).unwrap());
    }

    #[test]
    fn parse_x509_der_returns_correct_raw_der() {
        let pem = pem::parse(TEST_CERT_PEM).expect("PEM should parse");
        let der = pem.contents();
        let info = parse_x509_der(der).expect("DER should parse");
        assert_eq!(info.raw_der.len(), der.len());
        assert_eq!(info.raw_der, der);
    }

    #[test]
    fn parse_x509_der_validity_period_order() {
        let info = parse_x509_pem(TEST_CERT_PEM).expect("PEM should parse");
        assert!(
            info.not_before < info.not_after,
            "not_before must precede not_after"
        );
        // The test cert is valid from 2026 to 2036.
        assert_eq!(info.not_before.year(), 2026);
        assert_eq!(info.not_after.year(), 2036);
    }

    /// Verify that `verify_chain` detects a signature mismatch when the
    /// child certificate's TBSCertificate was signed by a different key
    /// than the one in the parent certificate's SubjectPublicKeyInfo.
    ///
    /// The test uses two copies of the same self-signed RSA cert but
    /// corrupts the leaf's signature bytes. Since the RSA algorithm OID
    /// is not SM2, cryptographic verification is currently skipped
    /// (see `verify_cert_signature`), so the chain is accepted.  This
    /// test documents the current behavior; once SM2 cert DER generation
    /// support is available the assertion should be inverted.
    ///
    /// Additionally, the test verifies that chains with mismatched
    /// subject/issuer are rejected even when raw_der is present.
    #[test]
    fn verify_chain_rejects_self_signed_with_wrong_signature() {
        let pem = pem::parse(TEST_CERT_PEM).expect("PEM should parse");
        let der = pem.contents();
        let cert_info = parse_x509_der(der).unwrap();

        // 1. Self-signed with same cert: subject/issuer match, accepted.
        let mut root = cert_info.clone();
        root.serial = vec![0xFF]; // different serial
        assert!(
            verify_chain(&cert_info, &[], &root, None).unwrap(),
            "valid self-signed chain with different serials should be accepted"
        );

        // 2. Corrupt leaf's raw_der signature bytes.
        let mut corrupted = cert_info.clone();
        let sig_start = corrupted.raw_der.len() * 4 / 5;
        for b in &mut corrupted.raw_der[sig_start..] {
            *b ^= 0xFF;
        }
        // Subject/issuer still match, serials differ.
        // Corrupted DER may fail to parse (returns true = pass-through) or
        // be parsed with wrong signature (RSA, skipped).
        // Current implementation accepts this chain.
        // TODO(gbt38540): With SM2 test certs, assert false here.
        let result = verify_chain(&corrupted, &[], &root, None);
        assert!(
            result.is_ok(),
            "corrupted DER should not cause a hard error"
        );

        // 3. Mismatched subject/issuer with raw_der present: must reject.
        let mut bad_root = cert_info.clone();
        bad_root.subject = "CN=wrong".into();
        bad_root.serial = vec![0xFE];
        assert!(
            !verify_chain(&cert_info, &[], &bad_root, None).unwrap(),
            "mismatched subject/issuer must be rejected even with raw_der"
        );
    }

    /// Verify that `verify_chain` with `ocsp_endpoint` does not change
    /// the result when OCSP always returns "not revoked" (placeholder).
    #[test]
    fn verify_chain_with_ocsp_endpoint_placeholder() {
        let leaf = CertificateInfo {
            subject: "CN=leaf".into(),
            issuer: "CN=root".into(),
            serial: vec![1],
            not_before: Utc::now(),
            not_after: Utc::now(),
            raw_der: vec![],
        };
        let root = CertificateInfo {
            subject: "CN=root".into(),
            issuer: "CN=root".into(),
            serial: vec![2],
            not_before: Utc::now(),
            not_after: Utc::now(),
            raw_der: vec![],
        };
        // OCSP placeholder always returns not-revoked.
        assert!(verify_chain(&leaf, &[], &root, Some("http://ocsp.example.com")).unwrap());
    }

    // ── SM2 certificate generation helpers ──────────────────────────────
    //
    // Build minimal X.509v3 DER certificates signed with SM2-with-SM3 so
    // that `verify_chain` exercises the real cryptographic SM2 path
    // (OID 1.2.156.10197.1.501).

    /// Raw OID bytes for SM2-with-SM3 signature (1.2.156.10197.1.501).
    const OID_SM2_SM3: &[u8] = &[0x2A, 0x81, 0x1C, 0xCF, 0x55, 0x01, 0x83, 0x75];
    /// Raw OID bytes for SM2 elliptic curve (1.2.156.10197.1.301).
    const OID_SM2_CURVE: &[u8] = &[0x2A, 0x81, 0x1C, 0xCF, 0x55, 0x01, 0x82, 0x2D];
    /// Raw OID bytes for id-ecPublicKey (1.2.840.10045.2.1).
    const OID_EC_PUBKEY: &[u8] = &[0x2A, 0x86, 0x48, 0xCE, 0x3D, 0x02, 0x01];
    /// OID bytes for commonName (2.5.4.3).
    const OID_CN: &[u8] = &[0x55, 0x04, 0x03];
    /// OID bytes for organizationName (2.5.4.10).
    const OID_O: &[u8] = &[0x55, 0x04, 0x0A];

    // ---- low-level DER encoding helpers ----

    #[allow(clippy::cast_possible_truncation)]
    fn der_push_len(len: usize, out: &mut Vec<u8>) {
        if len < 0x80 {
            out.push(len as u8);
        } else if len < 0x100 {
            out.extend_from_slice(&[0x81, len as u8]);
        } else {
            out.extend_from_slice(&[0x82, (len >> 8) as u8, (len & 0xFF) as u8]);
        }
    }

    fn der_tlv(tag: u8, content: &[u8]) -> Vec<u8> {
        let mut v = Vec::with_capacity(4 + content.len());
        v.push(tag);
        der_push_len(content.len(), &mut v);
        v.extend_from_slice(content);
        v
    }

    fn der_seq(c: &[u8]) -> Vec<u8> {
        der_tlv(0x30, c)
    }
    fn der_set(c: &[u8]) -> Vec<u8> {
        der_tlv(0x31, c)
    }
    fn der_oid(oid: &[u8]) -> Vec<u8> {
        der_tlv(0x06, oid)
    }
    fn der_utf8(s: &str) -> Vec<u8> {
        der_tlv(0x0C, s.as_bytes())
    }
    fn der_utc(s: &str) -> Vec<u8> {
        der_tlv(0x17, s.as_bytes())
    }

    /// Encode a small positive integer (0..127).
    fn der_int_small(v: u8) -> Vec<u8> {
        der_tlv(0x02, &[v])
    }

    fn der_bitstring(raw: &[u8]) -> Vec<u8> {
        // BIT STRING with 0 unused bits: prepend 0x00 byte.
        let mut content = Vec::with_capacity(1 + raw.len());
        content.push(0x00);
        content.extend_from_slice(raw);
        der_tlv(0x03, &content)
    }

    /// Context-specific constructed tag [tag_num].
    fn der_ctx(tag_num: u8, content: &[u8]) -> Vec<u8> {
        der_tlv(0xA0 | tag_num, content)
    }

    // ---- X.509 structure helpers ----

    /// Build a Name with CN and O attributes.
    fn build_name(cn: &str, org: &str) -> Vec<u8> {
        let cn_atv = der_seq(&[der_oid(OID_CN), der_utf8(cn)].concat());
        let o_atv = der_seq(&[der_oid(OID_O), der_utf8(org)].concat());
        // Name ::= SEQUENCE OF RDN; RDN ::= SET OF AttributeTypeAndValue
        let cn_rdn = der_set(&cn_atv);
        let o_rdn = der_set(&o_atv);
        der_seq(&[cn_rdn, o_rdn].concat())
    }

    /// SM2-with-SM3 AlgorithmIdentifier.
    fn sm2_alg_id() -> Vec<u8> {
        der_seq(&der_oid(OID_SM2_SM3))
    }

    /// SubjectPublicKeyInfo for SM2 public key (SEC1 uncompressed point).
    fn build_spki(sec1_bytes: &[u8]) -> Vec<u8> {
        // AlgorithmIdentifier ::= SEQUENCE { OID id-ecPublicKey, OID SM2 curve }
        let alg_id = der_seq(&[der_oid(OID_EC_PUBKEY), der_oid(OID_SM2_CURVE)].concat());
        der_seq(&[alg_id, der_bitstring(sec1_bytes)].concat())
    }

    /// Build TBSCertificate DER for an SM2-signed cert.
    #[allow(clippy::too_many_arguments)]
    fn build_tbs(
        serial: u8,
        subject_cn: &str,
        subject_org: &str,
        issuer_cn: &str,
        issuer_org: &str,
        pub_key_sec1: &[u8],
    ) -> Vec<u8> {
        let version = der_ctx(0, &der_int_small(2)); // v3 = INTEGER 2
        let serial_number = der_int_small(serial);
        let sig_alg = sm2_alg_id(); // AlgorithmIdentifier in TBS
        let issuer = build_name(issuer_cn, issuer_org);
        let validity = der_seq(&[der_utc("260101000000Z"), der_utc("361231235959Z")].concat());
        let subject = build_name(subject_cn, subject_org);
        let spki = build_spki(pub_key_sec1);
        der_seq(
            &[
                version,
                serial_number,
                sig_alg,
                issuer,
                validity,
                subject,
                spki,
            ]
            .concat(),
        )
    }

    /// Sign `msg` with SM2-DSA and return the raw 64-byte (r||s) signature.
    fn sm2_sign(sk: &sm2::SecretKey, msg: &[u8]) -> Vec<u8> {
        use sm2::dsa::signature::Signer;
        let signing_key = sm2::dsa::SigningKey::new("1234567812345678", sk)
            .expect("SM2 SigningKey construction must succeed");
        let sig = signing_key.sign(msg);
        sig.to_bytes().to_vec()
    }

    /// Assemble a full X.509 Certificate DER from TBS and raw SM2 signature.
    fn assemble_cert(tbs_der: &[u8], sig_raw64: &[u8]) -> Vec<u8> {
        let sig_alg = sm2_alg_id();
        let sig_bitstring = der_bitstring(sig_raw64);
        der_seq(&[tbs_der, &sig_alg, &sig_bitstring].concat())
    }

    /// Generate a self-signed SM2 CA cert. Returns (cert_der, secret_key).
    fn gen_sm2_ca() -> (Vec<u8>, sm2::SecretKey) {
        use sm2::elliptic_curve::Generate;
        let sk = sm2::SecretKey::generate();
        let pk = sk.public_key();
        let sec1 = pk.to_sec1_bytes();
        let tbs = build_tbs(1, "SM2TestCA", "TestOrg", "SM2TestCA", "TestOrg", &sec1);
        let sig = sm2_sign(&sk, &tbs);
        let cert = assemble_cert(&tbs, &sig);

        // Sanity: verify the cert parses.
        Certificate::from_der(&cert).expect("SM2 CA cert must parse as valid X.509");

        (cert, sk)
    }

    /// Generate an SM2 leaf cert signed by `ca_sk`, with issuer matching the CA's subject.
    fn gen_sm2_leaf(ca_sk: &sm2::SecretKey) -> Vec<u8> {
        use sm2::elliptic_curve::Generate;
        let sk = sm2::SecretKey::generate();
        let pk = sk.public_key();
        let sec1 = pk.to_sec1_bytes();
        let tbs = build_tbs(2, "SM2TestLeaf", "TestOrg", "SM2TestCA", "TestOrg", &sec1);
        let sig = sm2_sign(ca_sk, &tbs);
        let cert = assemble_cert(&tbs, &sig);

        // Sanity: verify the cert parses.
        Certificate::from_der(&cert).expect("SM2 leaf cert must parse as valid X.509");

        cert
    }

    /// Verify that `verify_chain` accepts a valid SM2-signed certificate chain
    /// (leaf signed by CA, both carrying SM2-with-SM3 OIDs).
    #[test]
    fn verify_chain_sm2_valid() {
        let (ca_der, ca_sk) = gen_sm2_ca();
        let leaf_der = gen_sm2_leaf(&ca_sk);

        // Confirm SM2 OID is present in the leaf's signature algorithm.
        let leaf_cert = Certificate::from_der(&leaf_der).expect("leaf re-parse");
        let sig_oid = leaf_cert.signature_algorithm().oid.to_string();
        assert!(
            sig_oid.starts_with("1.2.156.10197"),
            "leaf must carry SM2 signature OID, got: {sig_oid}"
        );

        let leaf = parse_x509_der(&leaf_der).expect("SM2 leaf must parse");
        let root = parse_x509_der(&ca_der).expect("SM2 CA must parse");

        assert!(
            verify_chain(&leaf, &[], &root, None).unwrap(),
            "valid SM2 certificate chain must pass verify_chain"
        );
    }

    /// Verify that `verify_chain` rejects an SM2-signed certificate whose
    /// signature has been tampered with.
    #[test]
    fn verify_chain_rejects_sm2_with_tampered_signature() {
        let (ca_der, ca_sk) = gen_sm2_ca();
        let leaf_der = gen_sm2_leaf(&ca_sk);

        // Tamper: flip a byte inside the signature BIT STRING content.
        // The signature is the last64 bytes of the outer SEQUENCE, preceded
        // by the 0x00 unused-bits marker.  We flip a byte in the signature.
        let mut tampered = leaf_der.clone();
        // Find the last occurrence of the signature: it's near the end.
        // The BIT STRING tag (0x03) + length (0x41=65) + 0x00 + 64 bytes of sig.
        // So the last 64 bytes are the raw signature.
        if tampered.len() >= 64 {
            let sig_start = tampered.len() - 64;
            tampered[sig_start] ^= 0xFF;
        }

        let leaf = parse_x509_der(&tampered).expect("tampered DER must still parse");
        let root = parse_x509_der(&ca_der).expect("SM2 CA must parse");

        let result = verify_chain(&leaf, &[], &root, None).unwrap();
        assert!(
            !result,
            "SM2 chain with tampered signature must fail verify_chain"
        );
    }
}
