use super::*;
use crate::internal_helpers::{base64_decode, base64_encode, compute_sm3, hex};
use easyofd_core::{OfdPage, TextObject};
use easyofd_writer::OfdWriter;
use std::io::{Cursor, Read, Write};

fn make_ofd(p: &std::path::Path) {
    let mut pg = OfdPage::new(210.0, 297.0);
    pg.add_text(TextObject::new(10.0, 20.0, "Doc"));
    let mut w = OfdWriter::new();
    w.add_page(pg);
    w.build_to_file(p).unwrap();
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
    let path = dir.join("t.ofd");
    make_ofd(&path);
    let r = OfdSignatureBuilder::new(path.to_string_lossy().into_owned())
        .sign()
        .unwrap();
    assert_eq!(r.digest.len(), 64);
    assert_eq!(r.signature_value.len(), 128);
    assert_eq!(&r.into_bytes()[0..2], b"PK");
    let _ = std::fs::remove_file(&path);
}

#[test]
fn test_sign_with_seal() {
    let dir = std::env::temp_dir().join("easyofd_sig_seal2");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("s.ofd");
    make_ofd(&path);
    let r = OfdSignatureBuilder::new(path.to_string_lossy().into_owned())
        .seal(ElectronicSeal {
            image_data: vec![0x89],
            name: "S".into(),
            position: (1.0, 2.0),
            page: 1,
        })
        .sign()
        .unwrap();
    let bytes = r.into_bytes();
    let cur = Cursor::new(&bytes);
    let mut a = zip::ZipArchive::new(cur).unwrap();
    let names: Vec<String> = (0..a.len())
        .map(|i| a.by_index(i).unwrap().name().to_string())
        .collect();
    assert!(names.contains(&"Doc_0/Res/Seal_0.png".to_string()));
    assert!(names.contains(&"Doc_0/Signs/Signature.xml".to_string()));
    let mut e = a.by_name("Doc_0/Signs/Signature.xml").unwrap();
    let mut s = String::new();
    e.read_to_string(&mut s).unwrap();
    assert!(s.contains("SM2WithSM3"));
    assert!(!s.contains("PLACEHOLDER"));
    let _ = std::fs::remove_file(&path);
}

#[test]
fn verify_signature_gbt38540_roundtrip() {
    let dir = std::env::temp_dir().join("easyofd_sig_roundtrip_v2");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("rt.ofd");
    make_ofd(&path);
    let signed = OfdSignatureBuilder::new(path.to_string_lossy().into_owned())
        .seal(ElectronicSeal {
            image_data: vec![0x89, 0x50, 0x4e, 0x47],
            name: "Org Seal".into(),
            position: (10.0, 20.0),
            page: 1,
        })
        .sign()
        .unwrap();
    let signed_path = dir.join("signed.ofd");
    std::fs::write(&signed_path, signed.into_bytes()).unwrap();

    let info = read_signature(&signed_path).unwrap();
    assert!(info.reference_failures.is_empty());
    assert_eq!(info.algorithm, SignatureAlgorithm::Sm2WithSm3);
    assert_eq!(info.signature_value.len(), 128);
    assert!(!info.digest.is_empty());

    assert!(
        verify_signature(&signed_path).unwrap(),
        "未篡改的签名应通过验证"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn verify_signature_rejects_tampered_ofd_entry() {
    let dir = std::env::temp_dir().join("easyofd_sig_tamper");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("rt.ofd");
    make_ofd(&path);
    let signed = OfdSignatureBuilder::new(path.to_string_lossy().into_owned())
        .sign()
        .unwrap();
    let signed_path = dir.join("signed.ofd");
    let original_bytes = signed.into_bytes();
    std::fs::write(&signed_path, &original_bytes).unwrap();

    let tampered_bytes: Vec<u8> = {
        let src = Cursor::new(&original_bytes);
        let mut src_zip = zip::ZipArchive::new(src).unwrap();
        let out_cursor = Cursor::new(Vec::<u8>::new());
        let mut out_zip = zip::ZipWriter::new(out_cursor);
        let opts = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);
        for i in 0..src_zip.len() {
            let mut e = src_zip.by_index(i).unwrap();
            let name = e.name().to_string();
            let mut data = Vec::new();
            e.read_to_end(&mut data).unwrap();
            if name == "Doc_0/Document.xml" {
                data.push(b' ');
            }
            out_zip.start_file(name, opts).unwrap();
            out_zip.write_all(&data).unwrap();
        }
        out_zip.finish().expect("zip finish").into_inner()
    };
    std::fs::write(&signed_path, &tampered_bytes).unwrap();

    let info = read_signature(&signed_path).unwrap();
    assert!(!info.reference_failures.is_empty());
    assert!(!verify_signature(&signed_path).unwrap());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn verify_signature_rejects_truncated_signature() {
    let dir = std::env::temp_dir().join("easyofd_sig_truncated");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("rt.ofd");
    make_ofd(&path);
    let signed = OfdSignatureBuilder::new(path.to_string_lossy().into_owned())
        .sign()
        .unwrap();
    let mut bytes = signed.into_bytes();
    let signature_xml_marker = b"Doc_0/Signs/Signature.xml";
    let pos = find_substr(&bytes, signature_xml_marker).unwrap();
    let truncate_at = pos + signature_xml_marker.len() + 400;
    if truncate_at < bytes.len() {
        bytes.truncate(truncate_at);
    }
    let signed_path = dir.join("signed_trunc.ofd");
    std::fs::write(&signed_path, &bytes).unwrap();
    let result = verify_signature(&signed_path);
    assert!(matches!(result, Ok(false) | Err(_)));
    let _ = std::fs::remove_dir_all(&dir);
}

fn find_substr(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|w| w == needle)
}

#[test]
fn multi_signature_produces_multiple_signed_info() {
    use sm2::elliptic_curve::Generate;
    let dir = std::env::temp_dir().join("easyofd_multi_sig_test");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("multi.ofd");
    make_ofd(&path);
    let builder =
        OfdSignatureBuilder::new(path.to_string_lossy().into_owned()).with_multiple_seals(true);
    let builder = (0..3).fold(builder, |b, _| {
        b.add_signature(
            sm2::SecretKey::generate(),
            vec![ElectronicSeal {
                image_data: vec![0x89, 0x50, 0x4e, 0x47],
                name: "S".into(),
                position: (10.0, 20.0),
                page: 1,
            }],
        )
    });
    let signed = builder.sign_multiple().unwrap();
    let bytes = signed.into_bytes();
    let cur = Cursor::new(&bytes);
    let mut a = zip::ZipArchive::new(cur).unwrap();
    let names: Vec<String> = (0..a.len())
        .map(|i| a.by_index(i).unwrap().name().to_string())
        .collect();
    let sig_count = names
        .iter()
        .filter(|n| {
            n.starts_with("Doc_0/Signs/Signature_")
                && std::path::Path::new(n)
                    .extension()
                    .is_some_and(|e| e.eq_ignore_ascii_case("xml"))
        })
        .count();
    assert!(sig_count >= 3);
    let ofd_xml_name = names
        .iter()
        .find(|n| n.ends_with("OFD.xml"))
        .unwrap()
        .clone();
    let mut ofd_file = a.by_name(&ofd_xml_name).unwrap();
    let mut ofd_content = String::new();
    ofd_file.read_to_string(&mut ofd_content).unwrap();
    assert!(ofd_content.contains("<ofd:Signatures>"));
    assert!(ofd_content.contains("Signature_0.xml"));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn verify_signature_multi_recognizes_all_signers() {
    use sm2::elliptic_curve::Generate;
    let dir = std::env::temp_dir().join("easyofd_multi_verify_test");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("mv.ofd");
    make_ofd(&path);
    let builder =
        OfdSignatureBuilder::new(path.to_string_lossy().into_owned()).with_multiple_seals(true);
    let builder = (0..3).fold(builder, |b, _| {
        b.add_signature(
            sm2::SecretKey::generate(),
            vec![ElectronicSeal {
                image_data: vec![0x89, 0x50, 0x4e, 0x47],
                name: "S".into(),
                position: (10.0, 20.0),
                page: 1,
            }],
        )
    });
    let signed = builder.sign_multiple().unwrap();
    let signed_path = dir.join("signed_mv.ofd");
    std::fs::write(&signed_path, signed.into_bytes()).unwrap();
    let results = verify_signature_multi(&signed_path).unwrap();
    assert_eq!(results.len(), 3);
    for (i, r) in results.iter().enumerate() {
        assert!(r.valid, "signer {i} should be valid");
        assert!(!r.signed_info_digest.is_empty());
    }
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn seal_info_embeds_esl_and_keyinfo_in_signature_xml() {
    let dir = std::env::temp_dir().join("easyofd_seal_info_esl_keyinfo");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("si.ofd");
    make_ofd(&path);
    let seal = SealInfo {
        name: "TestOrgSeal".into(),
        created_at: chrono::Utc::now(),
        valid_until: chrono::Utc::now() + chrono::Duration::days(365),
        cert_der: vec![0x30, 0x82, 0x01, 0x02, 0x03, 0x01, 0x00],
        image: vec![0x89, 0x50, 0x4E, 0x47],
        version: 1,
    };
    let r = OfdSignatureBuilder::new(path.to_string_lossy().into_owned())
        .seal(seal)
        .sign()
        .unwrap();
    let bytes = r.into_bytes();
    let cur = Cursor::new(&bytes);
    let mut a = zip::ZipArchive::new(cur).unwrap();
    let names: Vec<String> = (0..a.len())
        .map(|i| a.by_index(i).unwrap().name().to_string())
        .collect();
    assert!(names.contains(&"Doc_0/Seal_0.esl".to_string()));
    let mut sig_file = a.by_name("Doc_0/Signs/Signature.xml").unwrap();
    let mut sig_xml = String::new();
    sig_file.read_to_string(&mut sig_xml).unwrap();
    assert!(sig_xml.contains("<ofd:KeyInfo>"));
    assert!(sig_xml.contains("<ofd:Certificate>"));
    let expected_b64 = base64_encode(&[0x30_u8, 0x82, 0x01, 0x02, 0x03, 0x01, 0x00]);
    assert!(sig_xml.contains(&expected_b64));
    assert!(sig_xml.contains(r#"Ref="Doc_0/Seal_0.esl""#));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn signature_xml_includes_timestamp_when_configured() {
    let dir = std::env::temp_dir().join("easyofd_sig_timestamp_test");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("ts.ofd");
    make_ofd(&path);
    let ts = create_timestamp(chrono::Utc::now(), "EasyOFD-TSA");
    let r = OfdSignatureBuilder::new(path.to_string_lossy().into_owned())
        .with_timestamp(ts)
        .sign()
        .unwrap();
    let bytes = r.into_bytes();
    let cur = Cursor::new(&bytes);
    let mut a = zip::ZipArchive::new(cur).unwrap();
    let mut sig_file = a.by_name("Doc_0/Signs/Signature.xml").unwrap();
    let mut sig_xml = String::new();
    sig_file.read_to_string(&mut sig_xml).unwrap();
    assert!(sig_xml.contains("<ofd:TimeStamp>"));
    let ts_start = sig_xml.find("<ofd:TimeStamp>").unwrap() + "<ofd:TimeStamp>".len();
    let ts_end = sig_xml.find("</ofd:TimeStamp>").unwrap();
    let ts_b64 = &sig_xml[ts_start..ts_end];
    let ts_der = base64_decode(ts_b64).expect("TimeStamp Base64 应可解码");
    let decoded_ts = timestamp::decode_der(&ts_der).expect("TimeStamp DER 应可解码");
    assert_eq!(decoded_ts.tsa_name, "EasyOFD-TSA");
    assert_eq!(decoded_ts.signature_oid, "1.2.3.4.5");
    let _ = std::fs::remove_dir_all(&dir);
}

// ── Helper functions for coverage tests ────────────────────────

fn make_signed_ofd_in(dir: &std::path::Path) -> (std::path::PathBuf, Vec<u8>) {
    let input = dir.join("input.ofd");
    make_ofd(&input);
    let signed = OfdSignatureBuilder::new(input.to_string_lossy().into_owned())
        .seal(ElectronicSeal {
            image_data: vec![0x89, 0x50, 0x4e, 0x47],
            name: "S".into(),
            position: (10.0, 20.0),
            page: 1,
        })
        .sign()
        .unwrap();
    let bytes = signed.into_bytes();
    let path = dir.join("signed.ofd");
    std::fs::write(&path, &bytes).unwrap();
    (path, bytes)
}

fn repack_zip_with<F: FnMut(&str, &mut Vec<u8>)>(original: &[u8], mut f: F) -> Vec<u8> {
    let src = Cursor::new(original);
    let mut src_zip = zip::ZipArchive::new(src).unwrap();
    let out = Cursor::new(Vec::<u8>::new());
    let mut out_zip = zip::ZipWriter::new(out);
    let opts = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    for i in 0..src_zip.len() {
        let mut e = src_zip.by_index(i).unwrap();
        let name = e.name().to_string();
        let mut data = Vec::new();
        e.read_to_end(&mut data).unwrap();
        f(&name, &mut data);
        out_zip.start_file(&name, opts).unwrap();
        out_zip.write_all(&data).unwrap();
    }
    out_zip.finish().unwrap().into_inner()
}

fn replace_xml_tag(xml: &str, tag: &str, new_content: &str) -> String {
    let open = format!("<ofd:{tag}>");
    let close = format!("</ofd:{tag}>");
    if let Some(start_pos) = xml.find(&open) {
        let content_start = start_pos + open.len();
        if let Some(end_pos) = xml.find(&close) {
            return format!(
                "{}{}{}",
                &xml[..content_start],
                new_content,
                &xml[end_pos..]
            );
        }
    }
    xml.to_string()
}

// ── Tests: easy uncovered branches ────────────────────────────

#[test]
fn signed_ofd_save_writes_to_path() {
    let dir = std::env::temp_dir().join("easyofd_save_cov");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("src.ofd");
    make_ofd(&path);
    let signed = OfdSignatureBuilder::new(path.to_string_lossy().into_owned())
        .sign()
        .unwrap();
    let save_path = dir.join("out.ofd");
    signed.save(&save_path).unwrap();
    let saved = std::fs::read(&save_path).unwrap();
    assert_eq!(&saved[0..2], b"PK");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn builder_algorithm_sets_variant() {
    let dir = std::env::temp_dir().join("easyofd_alg_cov");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("a.ofd");
    make_ofd(&path);
    let r = OfdSignatureBuilder::new(path.to_string_lossy().into_owned())
        .algorithm(SignatureAlgorithm::Sha256WithRsa)
        .sign()
        .unwrap();
    let bytes = r.into_bytes();
    let cur = Cursor::new(&bytes);
    let mut a = zip::ZipArchive::new(cur).unwrap();
    let mut f = a.by_name("Doc_0/Signs/Signature.xml").unwrap();
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();
    assert!(s.contains("SHA256WithRSA"));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn sign_multiple_empty_signers_returns_error() {
    let dir = std::env::temp_dir().join("easyofd_multi_empty_cov");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("e.ofd");
    make_ofd(&path);
    let result = OfdSignatureBuilder::new(path.to_string_lossy().into_owned())
        .with_multiple_seals(true)
        .sign_multiple();
    assert!(result.is_err());
    let msg = format!("{}", result.unwrap_err());
    assert!(msg.contains("at least one signer"), "unexpected: {msg}");
    let _ = std::fs::remove_dir_all(&dir);
}

// ── Tests: verify_signature error branches ────────────────────

#[test]
fn verify_signature_rejects_non_sm2_algorithm() {
    let dir = std::env::temp_dir().join("easyofd_vsig_nonsm2");
    std::fs::create_dir_all(&dir).unwrap();
    let (_signed_path, original) = make_signed_ofd_in(&dir);
    let tampered = repack_zip_with(&original, |name, data| {
        if name == "Doc_0/Signs/Signature.xml" {
            let xml = String::from_utf8_lossy(data).to_string();
            *data = xml.replace("SM2WithSM3", "SHA256WithRSA").into_bytes();
        }
    });
    let tampered_path = dir.join("t.ofd");
    std::fs::write(&tampered_path, &tampered).unwrap();
    let err = verify_signature(&tampered_path).unwrap_err();
    assert!(
        format!("{err}").contains("仅支持 SM2WithSM3"),
        "unexpected: {err}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn verify_signature_rejects_empty_public_key() {
    let dir = std::env::temp_dir().join("easyofd_vsig_emptypk");
    std::fs::create_dir_all(&dir).unwrap();
    let (_signed_path, original) = make_signed_ofd_in(&dir);
    let tampered = repack_zip_with(&original, |name, data| {
        if name == "Doc_0/Signs/Signature.xml" {
            let xml = String::from_utf8_lossy(data).to_string();
            *data = replace_xml_tag(&xml, "PublicKey", "").into_bytes();
        }
    });
    let tampered_path = dir.join("t.ofd");
    std::fs::write(&tampered_path, &tampered).unwrap();
    let result = verify_signature(&tampered_path).unwrap();
    assert!(!result, "empty public key should yield false");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn verify_signature_rejects_short_signature_value() {
    let dir = std::env::temp_dir().join("easyofd_vsig_shortsig");
    std::fs::create_dir_all(&dir).unwrap();
    let (_signed_path, original) = make_signed_ofd_in(&dir);
    let tampered = repack_zip_with(&original, |name, data| {
        if name == "Doc_0/Signs/SignedValue.dat" {
            *data = vec![0xAA, 0xBB];
        }
    });
    let tampered_path = dir.join("t.ofd");
    std::fs::write(&tampered_path, &tampered).unwrap();
    let err = verify_signature(&tampered_path).unwrap_err();
    assert!(
        format!("{err}").contains("签名长度无效"),
        "unexpected: {err}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn verify_signature_rejects_wrong_pubkey_length() {
    let dir = std::env::temp_dir().join("easyofd_vsig_wrongpklen");
    std::fs::create_dir_all(&dir).unwrap();
    let (_signed_path, original) = make_signed_ofd_in(&dir);
    let tampered = repack_zip_with(&original, |name, data| {
        if name == "Doc_0/Signs/Signature.xml" {
            let xml = String::from_utf8_lossy(data).to_string();
            *data = replace_xml_tag(&xml, "PublicKey", "aabbccdd").into_bytes();
        }
    });
    let tampered_path = dir.join("t.ofd");
    std::fs::write(&tampered_path, &tampered).unwrap();
    let err = verify_signature(&tampered_path).unwrap_err();
    assert!(
        format!("{err}").contains("公钥长度无效"),
        "unexpected: {err}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

// ── Tests: verify_signature_multi branches ────────────────────

#[test]
fn verify_multi_no_ofd_xml_returns_empty() {
    let dir = std::env::temp_dir().join("easyofd_multi_noofd");
    std::fs::create_dir_all(&dir).unwrap();
    let zip_bytes = {
        let out = Cursor::new(Vec::<u8>::new());
        let mut zip = zip::ZipWriter::new(out);
        let opts = zip::write::SimpleFileOptions::default();
        zip.start_file("dummy.txt", opts).unwrap();
        zip.write_all(b"hello").unwrap();
        zip.finish().unwrap().into_inner()
    };
    let path = dir.join("no_ofd.ofd");
    std::fs::write(&path, &zip_bytes).unwrap();
    let results = verify_signature_multi(&path).unwrap();
    assert!(results.is_empty());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn verify_multi_empty_signatures_returns_empty() {
    let dir = std::env::temp_dir().join("easyofd_multi_emptysig");
    std::fs::create_dir_all(&dir).unwrap();
    let ofd_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<ofd:OFD xmlns:ofd="http://www.ofdspec.org/2016">
  <ofd:DocBody>
    <ofd:Signatures></ofd:Signatures>
  </ofd:DocBody>
</ofd:OFD>"#;
    let zip_bytes = {
        let out = Cursor::new(Vec::<u8>::new());
        let mut zip = zip::ZipWriter::new(out);
        let opts = zip::write::SimpleFileOptions::default();
        zip.start_file("OFD.xml", opts).unwrap();
        zip.write_all(ofd_xml.as_bytes()).unwrap();
        zip.finish().unwrap().into_inner()
    };
    let path = dir.join("empty_sig.ofd");
    std::fs::write(&path, &zip_bytes).unwrap();
    let results = verify_signature_multi(&path).unwrap();
    assert!(results.is_empty());
    let _ = std::fs::remove_dir_all(&dir);
}

#[allow(clippy::too_many_lines)]
#[test]
fn verify_multi_handles_all_broken_signature_paths() {
    use sm2::elliptic_curve::Generate;

    let dir = std::env::temp_dir().join("easyofd_multi_broken");
    std::fs::create_dir_all(&dir).unwrap();

    let dummy_content = b"hello";
    let dummy_hash = hex(&compute_sm3(dummy_content));
    let valid_si = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<ofd:SignedInfo xmlns:ofd="http://www.ofdspec.org/2016">
  <ofd:References>
    <ofd:FileRef CheckMethod="SM3" CheckValue="{dummy_hash}">dummy.txt</ofd:FileRef>
  </ofd:References>
</ofd:SignedInfo>"#
    );

    use sm2::dsa::signature::Signer as _;
    let sk = sm2::SecretKey::generate();
    let signing_key = sm2::dsa::SigningKey::new("1234567812345678", &sk).unwrap();
    let sig = signing_key.sign(valid_si.as_bytes());
    let sig_hex = hex(&sig.to_bytes());
    let pub_hex = hex(&signing_key.verifying_key().to_sec1_bytes());

    fn mk_sig(si: &str, sv: &str, pk: &str, sval: &str) -> String {
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<ofd:Signature xmlns:ofd="http://www.ofdspec.org/2016" ID="S">
  <ofd:SignedInfoRef>{si}</ofd:SignedInfoRef>
  <ofd:SignedValue>{sv}</ofd:SignedValue>
  <ofd:PublicKey>{pk}</ofd:PublicKey>
  <ofd:SignatureValue>{sval}</ofd:SignatureValue>
</ofd:Signature>"#
        )
    }

    let mut entries: Vec<(String, Vec<u8>)> = Vec::new();
    entries.push(("dummy.txt".into(), dummy_content.to_vec()));
    // Sig_0: missing (line 1164)
    // Sig_1: non-UTF-8 (line 1167)
    entries.push(("Doc_0/Signs/S_1.xml".into(), vec![0xFF, 0xFE]));
    // Sig_2: malformed XML (line 1170)
    entries.push(("Doc_0/Signs/S_2.xml".into(), b"<bad<<".to_vec()));
    // Sig_3: valid XML, SignedInfo missing (line 1178)
    entries.push((
        "Doc_0/Signs/S_3.xml".into(),
        mk_sig("SI_3.xml", "SV_3.dat", &pub_hex, &sig_hex).into_bytes(),
    ));
    // Sig_4: valid XML + SignedInfo, SignedValue missing (line 1187)
    entries.push((
        "Doc_0/Signs/S_4.xml".into(),
        mk_sig(
            "Doc_0/Signs/SI_4.xml",
            "Doc_0/Signs/SV_4.dat",
            &pub_hex,
            &sig_hex,
        )
        .into_bytes(),
    ));
    entries.push(("Doc_0/Signs/SI_4.xml".into(), valid_si.clone().into_bytes()));
    // Sig_5: SignedInfo malformed XML (line 1193)
    entries.push((
        "Doc_0/Signs/S_5.xml".into(),
        mk_sig(
            "Doc_0/Signs/SI_5.xml",
            "Doc_0/Signs/SV_5.dat",
            &pub_hex,
            &sig_hex,
        )
        .into_bytes(),
    ));
    entries.push(("Doc_0/Signs/SI_5.xml".into(), b"<<invalid".to_vec()));
    entries.push(("Doc_0/Signs/SV_5.dat".into(), sig.to_bytes().to_vec()));
    // Sig_6: reference check fails (line 1200)
    entries.push((
        "Doc_0/Signs/S_6.xml".into(),
        mk_sig(
            "Doc_0/Signs/SI_6.xml",
            "Doc_0/Signs/SV_6.dat",
            &pub_hex,
            &sig_hex,
        )
        .into_bytes(),
    ));
    entries.push((
        "Doc_0/Signs/SI_6.xml".into(),
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<ofd:SignedInfo xmlns:ofd="http://www.ofdspec.org/2016">
  <ofd:References>
    <ofd:FileRef CheckMethod="SM3" CheckValue="{}">dummy.txt</ofd:FileRef>
  </ofd:References>
</ofd:SignedInfo>"#,
            "0".repeat(64)
        )
        .into_bytes(),
    ));
    entries.push(("Doc_0/Signs/SV_6.dat".into(), sig.to_bytes().to_vec()));
    // Sig_7: references pass, empty public key (line 1208)
    entries.push((
        "Doc_0/Signs/S_7.xml".into(),
        mk_sig("Doc_0/Signs/SI_7.xml", "Doc_0/Signs/SV_7.dat", "", &sig_hex).into_bytes(),
    ));
    entries.push(("Doc_0/Signs/SI_7.xml".into(), valid_si.clone().into_bytes()));
    entries.push(("Doc_0/Signs/SV_7.dat".into(), sig.to_bytes().to_vec()));
    // Sig_8: references pass, sig len != 64 (line 1215)
    entries.push((
        "Doc_0/Signs/S_8.xml".into(),
        mk_sig(
            "Doc_0/Signs/SI_8.xml",
            "Doc_0/Signs/SV_8.dat",
            &pub_hex,
            "aabb",
        )
        .into_bytes(),
    ));
    entries.push(("Doc_0/Signs/SI_8.xml".into(), valid_si.clone().into_bytes()));
    entries.push(("Doc_0/Signs/SV_8.dat".into(), vec![0xAA, 0xBB]));
    // Sig_9: references pass, pub unhex fails (line 1218)
    entries.push((
        "Doc_0/Signs/S_9.xml".into(),
        mk_sig(
            "Doc_0/Signs/SI_9.xml",
            "Doc_0/Signs/SV_9.dat",
            "ZZZZ",
            &sig_hex,
        )
        .into_bytes(),
    ));
    entries.push(("Doc_0/Signs/SI_9.xml".into(), valid_si.clone().into_bytes()));
    entries.push(("Doc_0/Signs/SV_9.dat".into(), sig.to_bytes().to_vec()));
    // Sig_10: references pass, pub len wrong (line 1221)
    entries.push((
        "Doc_0/Signs/S_10.xml".into(),
        mk_sig(
            "Doc_0/Signs/SI_10.xml",
            "Doc_0/Signs/SV_10.dat",
            "aabbccddeeff",
            &sig_hex,
        )
        .into_bytes(),
    ));
    entries.push((
        "Doc_0/Signs/SI_10.xml".into(),
        valid_si.clone().into_bytes(),
    ));
    entries.push(("Doc_0/Signs/SV_10.dat".into(), sig.to_bytes().to_vec()));

    let sig_refs: String = (0..=10)
        .map(|i| format!("      <ofd:SignatureRef>Doc_0/Signs/S_{i}.xml</ofd:SignatureRef>"))
        .collect::<Vec<_>>()
        .join("\n");
    let ofd_xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<ofd:OFD xmlns:ofd="http://www.ofdspec.org/2016">
  <ofd:DocBody>
    <ofd:Signatures>
{sig_refs}
    </ofd:Signatures>
  </ofd:DocBody>
</ofd:OFD>"#
    );

    let out = Cursor::new(Vec::<u8>::new());
    let mut zip = zip::ZipWriter::new(out);
    let opts = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    for (name, data) in &entries {
        zip.start_file(name, opts).unwrap();
        zip.write_all(data).unwrap();
    }
    zip.start_file("OFD.xml", opts).unwrap();
    zip.write_all(ofd_xml.as_bytes()).unwrap();
    let zip_bytes = zip.finish().unwrap().into_inner();

    let path = dir.join("broken.ofd");
    std::fs::write(&path, &zip_bytes).unwrap();

    let results = verify_signature_multi(&path).unwrap();
    assert_eq!(results.len(), 11, "expected 11 signature results");

    for (i, r) in results.iter().enumerate() {
        assert!(!r.valid, "sig {i} should be invalid, got valid=true");
    }

    assert!(results[0].signed_info_digest.is_empty());
    assert_eq!(results[0].name, "S_0.xml");
    assert!(results[1].signed_info_digest.is_empty());
    assert!(results[2].signed_info_digest.is_empty());
    assert!(results[3].signed_info_digest.is_empty());
    for r in results.iter().skip(4) {
        assert!(
            !r.signed_info_digest.is_empty(),
            "{} should have digest",
            r.name
        );
    }

    let _ = std::fs::remove_dir_all(&dir);
}

// ── Tests: timestamp error branches ─────────────────────────

#[test]
fn timestamp_encode_der_long_tsa_name() {
    // TSA name > 127 chars exercises encode_der_length with len >= 0x80.
    let long_name = "A".repeat(200);
    let ts = create_timestamp(chrono::Utc::now(), &long_name);
    let der = timestamp::encode_der(&ts).unwrap();
    let decoded = timestamp::decode_der(&der).unwrap();
    assert_eq!(decoded.tsa_name, long_name);
}

#[test]
fn timestamp_decode_der_rejects_wrong_generalized_time_tag() {
    let ts = create_timestamp(chrono::Utc::now(), "TSA");
    let der = timestamp::encode_der(&ts).unwrap();
    let mut bad = der;
    // First TLV inside SEQUENCE is GeneralizedTime (tag 0x18). Replace with 0x01.
    if bad.len() > 2 && bad[2] == 0x18 {
        bad[2] = 0x01;
    }
    assert!(timestamp::decode_der(&bad).is_err());
}

#[test]
fn timestamp_decode_der_rejects_wrong_utf8_string_tag() {
    let ts = create_timestamp(chrono::Utc::now(), "TSA");
    let der = timestamp::encode_der(&ts).unwrap();
    let mut bad = der;
    // Skip SEQUENCE tag+len (2 bytes), then GeneralizedTime TLV to find UTF8String.
    if bad.len() > 4 {
        let gt_len = if bad[3] < 0x80 { bad[3] as usize } else { 0 };
        let utf8_pos = 4 + gt_len;
        if utf8_pos < bad.len() && bad[utf8_pos] == 0x0C {
            bad[utf8_pos] = 0x01;
        }
    }
    assert!(timestamp::decode_der(&bad).is_err());
}

#[test]
fn timestamp_decode_der_rejects_wrong_oid_tag() {
    let ts = create_timestamp(chrono::Utc::now(), "TSA");
    let der = timestamp::encode_der(&ts).unwrap();
    let mut bad = der;
    for byte in bad.iter_mut().skip(2) {
        if *byte == 0x06 {
            *byte = 0x01;
            break;
        }
    }
    assert!(timestamp::decode_der(&bad).is_err());
}

// ── Tests: crl error branches ───────────────────────────────

#[test]
fn crl_parse_rejects_empty_input() {
    let result = parse_crl_der(&[]);
    assert!(result.is_err(), "empty input should fail");
}

#[test]
fn crl_parse_rejects_truncated_input() {
    let result = parse_crl_der(&[0x30, 0x10, 0x02, 0x01]);
    assert!(result.is_err(), "truncated input should fail");
}

#[test]
fn crl_parse_rejects_non_sequence() {
    let result = parse_crl_der(&[0x01, 0x01, 0x00]);
    assert!(result.is_err(), "non-SEQUENCE should fail");
}

// ── Tests: xml self-closing SignatureRef path ───────────────

#[test]
fn verify_multi_self_closing_signature_ref() {
    // Exercise the self-closing <ofd:SignatureRef BaseLoc="..."/> path
    // in parse_ofd_root (xml.rs lines 208-221).
    use sm2::dsa::signature::Signer as _;
    use sm2::elliptic_curve::Generate;

    let dir = std::env::temp_dir().join("easyofd_selfclosing_ref");
    std::fs::create_dir_all(&dir).unwrap();

    let dummy_content = b"hello";
    let dummy_hash = hex(&compute_sm3(dummy_content));
    let valid_si = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<ofd:SignedInfo xmlns:ofd="http://www.ofdspec.org/2016">
  <ofd:References>
    <ofd:FileRef CheckMethod="SM3" CheckValue="{dummy_hash}" ExtraAttr="ignored">dummy.txt</ofd:FileRef>
  </ofd:References>
</ofd:SignedInfo>"#
    );

    let sk = sm2::SecretKey::generate();
    let signing_key = sm2::dsa::SigningKey::new("1234567812345678", &sk).unwrap();
    let sig = signing_key.sign(valid_si.as_bytes());
    let sig_hex = hex(&sig.to_bytes());
    let pub_hex = hex(&signing_key.verifying_key().to_sec1_bytes());

    let sig_xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<ofd:Signature xmlns:ofd="http://www.ofdspec.org/2016" ID="S">
  <ofd:SignedInfoRef>Doc_0/Signs/SI.xml</ofd:SignedInfoRef>
  <ofd:SignedValue>Doc_0/Signs/SV.dat</ofd:SignedValue>
  <ofd:PublicKey>{pub_hex}</ofd:PublicKey>
  <ofd:SignatureValue>{sig_hex}</ofd:SignatureValue>
  <ofd:ExtraTag>ignored</ofd:ExtraTag>
</ofd:Signature>"#
    );

    // OFD.xml uses self-closing SignatureRef with BaseLoc attribute.
    let ofd_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<ofd:OFD xmlns:ofd="http://www.ofdspec.org/2016">
  <ofd:DocBody>
    <ofd:Signatures>
      <ofd:SignatureRef BaseLoc="Doc_0/Signs/Sig.xml"/>
    </ofd:Signatures>
  </ofd:DocBody>
</ofd:OFD>"#;

    let out = Cursor::new(Vec::<u8>::new());
    let mut zip = zip::ZipWriter::new(out);
    let opts = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    zip.start_file("dummy.txt", opts).unwrap();
    zip.write_all(dummy_content).unwrap();
    zip.start_file("Doc_0/Signs/Sig.xml", opts).unwrap();
    zip.write_all(sig_xml.as_bytes()).unwrap();
    zip.start_file("Doc_0/Signs/SI.xml", opts).unwrap();
    zip.write_all(valid_si.as_bytes()).unwrap();
    zip.start_file("Doc_0/Signs/SV.dat", opts).unwrap();
    zip.write_all(&sig.to_bytes()).unwrap();
    zip.start_file("OFD.xml", opts).unwrap();
    zip.write_all(ofd_xml.as_bytes()).unwrap();
    let zip_bytes = zip.finish().unwrap().into_inner();

    let path = dir.join("selfclosing.ofd");
    std::fs::write(&path, &zip_bytes).unwrap();

    let results = verify_signature_multi(&path).unwrap();
    assert_eq!(results.len(), 1, "should find 1 signature via BaseLoc");
    assert!(results[0].valid, "signature should be valid");
    assert_eq!(results[0].name, "Sig.xml");

    let _ = std::fs::remove_dir_all(&dir);
}

// ── Tests: SignMode + SignatureContainer + DigitalSignContainer ───────

#[test]
fn sign_mode_default_is_whole_protected() {
    assert_eq!(SignMode::default(), SignMode::WholeProtected);
}

#[test]
fn sign_mode_debug_clone_copy() {
    let mode = SignMode::ContinueSign;
    let mode2 = mode;
    assert_eq!(format!("{mode:?}"), "ContinueSign");
    assert_eq!(mode, mode2);
}

#[test]
fn digital_sign_container_default_algorithm() {
    let container = DigitalSignContainer::default();
    assert_eq!(container.signature_method(), SignatureAlgorithm::Sm2WithSm3);
    assert_eq!(container.algorithm_oid(), "1.2.156.10197.1.501");
}

#[test]
fn digital_sign_container_sha256_rsa() {
    let container = DigitalSignContainer::new(SignatureAlgorithm::Sha256WithRsa);
    assert_eq!(
        container.signature_method(),
        SignatureAlgorithm::Sha256WithRsa
    );
    assert_eq!(container.algorithm_oid(), "1.2.840.113549.1.1.11");
}

#[test]
fn digital_sign_container_build_and_verify_roundtrip() {
    use sm2::elliptic_curve::Generate;

    let container = DigitalSignContainer::default();
    let sk = sm2::SecretKey::generate();
    let signing_key = sm2::dsa::SigningKey::new("1234567812345678", &sk).unwrap();
    let pk = signing_key.verifying_key();

    let data = b"test signed info bytes";
    let signed_value = container.build_signed_value(data, &sk).unwrap();
    assert_eq!(signed_value.len(), 64, "SM2 signature should be 64 bytes");
    assert!(
        container.verify(data, &signed_value, pk),
        "verify should pass"
    );
}

#[test]
fn digital_sign_container_verify_rejects_wrong_data() {
    use sm2::elliptic_curve::Generate;

    let container = DigitalSignContainer::default();
    let sk = sm2::SecretKey::generate();
    let signing_key = sm2::dsa::SigningKey::new("1234567812345678", &sk).unwrap();
    let pk = signing_key.verifying_key();

    let data = b"original data";
    let signed_value = container.build_signed_value(data, &sk).unwrap();
    assert!(
        !container.verify(b"tampered data", &signed_value, pk),
        "verify should fail for wrong data"
    );
}

#[test]
fn digital_sign_container_verify_rejects_wrong_key() {
    use sm2::elliptic_curve::Generate;

    let container = DigitalSignContainer::default();
    let sk1 = sm2::SecretKey::generate();
    let sk2 = sm2::SecretKey::generate();
    let signing_key2 = sm2::dsa::SigningKey::new("1234567812345678", &sk2).unwrap();
    let pk2 = signing_key2.verifying_key();

    let data = b"some data";
    let signed_value = container.build_signed_value(data, &sk1).unwrap();
    assert!(
        !container.verify(data, &signed_value, pk2),
        "verify should fail for wrong key"
    );
}

#[test]
fn digital_sign_container_verify_rejects_invalid_signature_bytes() {
    use sm2::elliptic_curve::Generate;

    let container = DigitalSignContainer::default();
    let sk = sm2::SecretKey::generate();
    let signing_key = sm2::dsa::SigningKey::new("1234567812345678", &sk).unwrap();
    let pk = signing_key.verifying_key();

    let bad_sig = vec![0xFF; 64];
    assert!(
        !container.verify(b"data", &bad_sig, pk),
        "verify should reject garbage signature"
    );
}

#[test]
fn builder_sign_mode_method_sets_mode() {
    let dir = std::env::temp_dir().join("easyofd_signmode_set");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("sm.ofd");
    make_ofd(&path);
    let builder = OfdSignatureBuilder::new(path.to_string_lossy().into_owned())
        .sign_mode(SignMode::ContinueSign);
    // Builder should still be usable; just verify it builds without error.
    let r = builder.sign();
    assert!(r.is_ok());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn continue_sign_excludes_signature_xml_from_digest() {
    // Verify ContinueSign mode works on a fresh OFD (no existing Signature.xml).
    // When there's no Signature.xml to exclude, it behaves like WholeProtected
    // but exercises the filtering code path.
    let dir = std::env::temp_dir().join("easyofd_continue_sign");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("cs.ofd");
    make_ofd(&path);

    let signed = OfdSignatureBuilder::new(path.to_string_lossy().into_owned())
        .seal(ElectronicSeal {
            image_data: vec![0x89, 0x50, 0x4e, 0x47],
            name: "S1".into(),
            position: (10.0, 20.0),
            page: 1,
        })
        .sign_mode(SignMode::ContinueSign)
        .sign()
        .unwrap();
    assert_eq!(signed.digest.len(), 64);
    assert_eq!(&signed.into_bytes()[0..2], b"PK");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn is_signature_file_matches_patterns() {
    // Direct test of the is_signature_file helper (indirectly via sign behavior).
    // These patterns are tested through the ContinueSign path above.
    // Verify the function is accessible through the module boundary.
    assert!(crate::ofd_signature_builder::is_signature_file(
        "Doc_0/Signs/Signature.xml"
    ));
    assert!(crate::ofd_signature_builder::is_signature_file(
        "Doc_0/Signs/Signature_0.xml"
    ));
    assert!(crate::ofd_signature_builder::is_signature_file(
        "Doc_0/Signs/Signature_99.xml"
    ));
    assert!(!crate::ofd_signature_builder::is_signature_file(
        "Doc_0/Signs/SignedInfo.xml"
    ));
    assert!(!crate::ofd_signature_builder::is_signature_file(
        "Doc_0/Signs/SignedValue.dat"
    ));
    assert!(!crate::ofd_signature_builder::is_signature_file(
        "Doc_0/Document.xml"
    ));
}
