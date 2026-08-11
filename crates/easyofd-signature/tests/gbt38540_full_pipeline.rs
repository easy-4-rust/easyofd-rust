//! GB/T 38540 end-to-end conformance tests.
//!
//! These tests exercise the full sign -> read -> verify pipeline as specified
//! in GB/T 38540-2020, including multi-signer and seal ESL roundtrip scenarios.

use easyofd_core::{OfdPage, TextObject};
use easyofd_signature::{
    ElectronicSeal, OfdSignatureBuilder, SealInfo, decode_seal_esl, encode_seal_esl,
    read_signature, verify_signature,
};
use easyofd_writer::OfdWriter;
use std::io::{Cursor, Read as _};
use std::path::PathBuf;

/// Helper: create a minimal OFD file at `path`.
fn make_ofd(path: &std::path::Path) {
    let mut pg = OfdPage::new(210.0, 297.0);
    pg.add_text(TextObject::new(10.0, 20.0, "ConformanceTest"));
    let mut w = OfdWriter::new();
    w.add_page(pg);
    w.build_to_file(path).unwrap();
}

/// Helper: create a unique temp directory for a test.
fn make_temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "easyofd_conformance_{name}_{id}",
        id = std::process::id()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

// ── conformance_sign_verify_roundtrip ─────────────────────────────────
//
// GB/T 38540 conformance: sign -> read -> verify -> tamper one byte -> verify fails.

#[test]
fn conformance_sign_verify_roundtrip() {
    let dir = make_temp_dir("roundtrip");
    let ofd_path = dir.join("input.ofd");
    make_ofd(&ofd_path);

    // 1. Sign the OFD.
    let signed = OfdSignatureBuilder::new(ofd_path.to_string_lossy().into_owned())
        .seal(ElectronicSeal {
            image_data: vec![0x89, 0x50, 0x4E, 0x47], // PNG magic
            name: "ConformanceSeal".into(),
            position: (10.0, 20.0),
            page: 1,
        })
        .sign()
        .unwrap();
    let signed_path = dir.join("signed.ofd");
    std::fs::write(&signed_path, signed.into_bytes()).unwrap();

    // 2. Read signature: must parse all fields, no reference failures.
    let info = read_signature(&signed_path).unwrap();
    assert!(
        info.reference_failures.is_empty(),
        "clean signed OFD must have zero reference failures, got: {:?}",
        info.reference_failures
    );
    assert_eq!(
        info.signature_value.len(),
        128,
        "SM2 signature must be 64 bytes = 128 hex"
    );
    assert!(!info.digest.is_empty());

    // 3. Verify: must return true.
    assert!(
        verify_signature(&signed_path).unwrap(),
        "untampered signed OFD must pass verification"
    );

    // 4. Tamper one byte in a protected entry (Document.xml).
    let original_bytes = std::fs::read(&signed_path).unwrap();
    let tampered_bytes = {
        let src = Cursor::new(&original_bytes);
        let mut src_zip = zip::ZipArchive::new(src).unwrap();
        let out = Cursor::new(Vec::<u8>::new());
        let mut out_zip = zip::ZipWriter::new(out);
        let opts = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);
        for i in 0..src_zip.len() {
            let mut entry = src_zip.by_index(i).unwrap();
            let name = entry.name().to_string();
            let mut data = Vec::new();
            entry.read_to_end(&mut data).unwrap();
            if name == "Doc_0/Document.xml" {
                // Tamper: flip one byte.
                if let Some(last) = data.last_mut() {
                    *last = last.wrapping_add(1);
                }
            }
            out_zip.start_file(&name, opts).unwrap();
            std::io::Write::write_all(&mut out_zip, &data).unwrap();
        }
        out_zip.finish().unwrap().into_inner()
    };
    std::fs::write(&signed_path, &tampered_bytes).unwrap();

    // 5. After tampering: read_signature must report failures, verify must return false.
    let tampered_info = read_signature(&signed_path).unwrap();
    assert!(
        !tampered_info.reference_failures.is_empty(),
        "tampered OFD must have at least one reference failure"
    );
    assert!(
        !verify_signature(&signed_path).unwrap(),
        "tampered OFD must fail verification"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

// ── conformance_multi_signature ───────────────────────────────────────
//
// GB/T 38540 conformance: sign_multiple() with 3 signers -> OFD.xml contains
// <ofd:Signatures> and 3 Signature_<n>.xml files.

#[test]
fn conformance_multi_signature() {
    use sm2::elliptic_curve::Generate;

    let dir = make_temp_dir("multi");
    let ofd_path = dir.join("multi.ofd");
    make_ofd(&ofd_path);

    // Build 3 independent signers.
    let builder =
        OfdSignatureBuilder::new(ofd_path.to_string_lossy().into_owned()).with_multiple_seals(true);
    let builder = (0..3).fold(builder, |b, i| {
        b.add_signature(
            sm2::SecretKey::generate(),
            vec![ElectronicSeal {
                image_data: vec![0x89, 0x50, 0x4E, 0x47],
                name: format!("Seal_{i}"),
                position: (10.0 * f64::from(i), 20.0),
                page: 1,
            }],
        )
    });

    let signed = builder.sign_multiple().unwrap();
    let bytes = signed.into_bytes();

    // Verify ZIP structure.
    let cur = Cursor::new(&bytes);
    let mut archive = zip::ZipArchive::new(cur).unwrap();
    let names: Vec<String> = (0..archive.len())
        .map(|i| archive.by_index(i).unwrap().name().to_string())
        .collect();

    // Must have 3 Signature_<n>.xml files.
    let sig_xml_count = names
        .iter()
        .filter(|n| {
            n.starts_with("Doc_0/Signs/Signature_")
                && std::path::Path::new(n)
                    .extension()
                    .is_some_and(|e| e.eq_ignore_ascii_case("xml"))
        })
        .count();
    assert_eq!(
        sig_xml_count, 3,
        "expected 3 Signature_<n>.xml, found {sig_xml_count}"
    );

    // Must have 3 SignedInfo_<n>.xml files.
    let signed_info_count = names
        .iter()
        .filter(|n| n.starts_with("Doc_0/Signs/SignedInfo_"))
        .count();
    assert_eq!(
        signed_info_count, 3,
        "expected 3 SignedInfo_<n>.xml, found {signed_info_count}"
    );

    // Must have 3 SignedValue_<n>.dat files.
    let signed_value_count = names
        .iter()
        .filter(|n| n.starts_with("Doc_0/Signs/SignedValue_"))
        .count();
    assert_eq!(
        signed_value_count, 3,
        "expected 3 SignedValue_<n>.dat, found {signed_value_count}"
    );

    // OFD.xml must contain <ofd:Signatures> with all 3 refs.
    let ofd_xml_name = names
        .iter()
        .find(|n| n.ends_with("OFD.xml"))
        .expect("OFD.xml must exist")
        .clone();
    let mut ofd_file = archive.by_name(&ofd_xml_name).unwrap();
    let mut ofd_content = String::new();
    ofd_file.read_to_string(&mut ofd_content).unwrap();

    assert!(
        ofd_content.contains("<ofd:Signatures>"),
        "OFD.xml must contain <ofd:Signatures>"
    );
    assert!(
        ofd_content.contains("Signature_0.xml"),
        "OFD.xml must reference Signature_0.xml"
    );
    assert!(
        ofd_content.contains("Signature_1.xml"),
        "OFD.xml must reference Signature_1.xml"
    );
    assert!(
        ofd_content.contains("Signature_2.xml"),
        "OFD.xml must reference Signature_2.xml"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

// ── conformance_seal_esl ──────────────────────────────────────────────
//
// GB/T 38540 conformance: encode_seal_esl -> decode_seal_esl roundtrip
// with field equality check.

#[test]
fn conformance_seal_esl() {
    let info = SealInfo {
        name: "ConformanceSeal".into(),
        created_at: chrono::Utc::now(),
        valid_until: chrono::Utc::now() + chrono::Duration::days(365),
        cert_der: vec![0x30, 0x82, 0x01, 0x22, 0x30, 0x0D], // dummy cert DER
        image: vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A], // PNG header
        version: 2,
    };

    // Encode to DER.
    let der = encode_seal_esl(&info).expect("encode_seal_esl must succeed");
    assert!(!der.is_empty(), "DER output must not be empty");
    assert_eq!(der[0], 0x30, "DER must start with SEQUENCE tag");

    // Decode from DER.
    let decoded = decode_seal_esl(&der).expect("decode_seal_esl must succeed");

    // Fields that are part of the ASN.1 structure must roundtrip exactly.
    assert_eq!(decoded.version, info.version, "version must roundtrip");
    assert_eq!(decoded.cert_der, info.cert_der, "cert_der must roundtrip");

    // Fields NOT in the ASN.1 structure (name, created_at, valid_until, image)
    // are defaulted on decode -- verify they are NOT equal to the original
    // (this documents the expected behavior).
    assert!(
        decoded.name.is_empty(),
        "name is not in ASN.1, should be empty on decode"
    );
    assert!(
        decoded.image.is_empty(),
        "image is not in ASN.1, should be empty on decode"
    );

    // Roundtrip idempotency: encode(decode(encode(info))) == encode(info).
    let re_encoded = encode_seal_esl(&decoded).expect("re-encode must succeed");
    assert_eq!(
        der, re_encoded,
        "encode -> decode -> encode must be idempotent"
    );
}

// ── conformance_seal_esl_large_cert ───────────────────────────────────

#[test]
fn conformance_seal_esl_large_cert() {
    // Exercise 2-byte DER length encoding (cert > 127 bytes).
    let info = SealInfo {
        name: "LargeCertSeal".into(),
        created_at: chrono::Utc::now(),
        valid_until: chrono::Utc::now() + chrono::Duration::days(365),
        cert_der: vec![0xAB; 256],
        image: Vec::new(),
        version: 1,
    };
    let der = encode_seal_esl(&info).unwrap();
    let decoded = decode_seal_esl(&der).unwrap();
    assert_eq!(decoded.cert_der.len(), 256);
    assert_eq!(decoded.cert_der, info.cert_der);
    assert_eq!(decoded.version, 1);
}

// ── conformance_seal_esl_high_version ─────────────────────────────────

#[test]
fn conformance_seal_esl_high_version() {
    // Exercise DER INTEGER encoding with high bit set (needs 0x00 prefix).
    let info = SealInfo {
        name: "HighVersionSeal".into(),
        created_at: chrono::Utc::now(),
        valid_until: chrono::Utc::now() + chrono::Duration::days(365),
        cert_der: vec![0x01, 0x02],
        image: Vec::new(),
        version: 0x8000_0001,
    };
    let der = encode_seal_esl(&info).unwrap();
    let decoded = decode_seal_esl(&der).unwrap();
    assert_eq!(decoded.version, 0x8000_0001);
}
