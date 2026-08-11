//! Multi-signer support for OFD signatures (GB/T 38540 section 7).
//!
//! Produces OFD packages with multiple independent signatures, each with its
//! own SM2 key pair and seal set.  Each signer gets `Signature_<n>.xml`,
//! `SignedInfo_<n>.xml`, and `SignedValue_<n>.dat`.  The OFD.xml receives a
//! `<ofd:Signatures>` element listing all `SignatureRef` entries.

use crate::{ElectronicSeal, SignatureAlgorithm, compute_sm3, hex, xml_escape};
use easyofd_core::OfdResult;
use std::io::Write;
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

/// A single signer entry in a multi-signer scenario.
pub(crate) struct SignerEntry {
    /// The signer's SM2 secret key.
    pub secret_key: sm2::SecretKey,
    /// The signer's electronic seals.
    pub seals: Vec<ElectronicSeal>,
}

/// Extract text between XML tags (greedy first match).
fn extract_between(xml: &str, start: &str, end: &str) -> Option<String> {
    let s = xml.find(start)? + start.len();
    let e = xml[s..].find(end)? + s;
    Some(xml[s..e].to_string())
}

/// Build indexed `Signature.xml` for signer `sig_index`.
fn build_indexed_signature_xml(
    signed_info_xml: &str,
    seals: &[ElectronicSeal],
    sig_index: usize,
    pub_hex: &str,
    sig_hex: &str,
) -> String {
    let method = extract_between(
        signed_info_xml,
        "<ofd:SignatureMethod>",
        "</ofd:SignatureMethod>",
    )
    .unwrap_or_else(|| "SM2WithSM3".to_string());
    let provider = extract_between(signed_info_xml, "<ofd:Provider", "</ofd:Provider>")
        .unwrap_or_else(|| "easyofd-rust".to_string());
    let datetime = extract_between(
        signed_info_xml,
        "<ofd:SignatureDateTime>",
        "</ofd:SignatureDateTime>",
    )
    .unwrap_or_default();

    let mut seal_list = String::new();
    for (i, _) in seals.iter().enumerate() {
        use std::fmt::Write as _;
        let _ = write!(
            seal_list,
            r#"<ofd:Seal ID="Seal_{sig_index}_{i}" Type="Seal">Doc_0/Res/Seal_{sig_index}_{i}.png</ofd:Seal>"#
        );
    }

    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<ofd:Signature xmlns:ofd="http://www.ofdspec.org/2016" ID="Signature_{sig_index}">
  <ofd:SignedInfoRef>Doc_0/Signs/SignedInfo_{sig_index}.xml</ofd:SignedInfoRef>
  <ofd:SignedValue>Doc_0/Signs/SignedValue_{sig_index}.dat</ofd:SignedValue>
  <ofd:Provider Version="1.0">{provider}</ofd:Provider>
  <ofd:SignatureMethod>{method}</ofd:SignatureMethod>
  <ofd:SignatureDateTime>{datetime}</ofd:SignatureDateTime>
  {seal_list}
  <ofd:PublicKey>{pub_hex}</ofd:PublicKey>
  <ofd:SignatureValue>{sig_hex}</ofd:SignatureValue>
</ofd:Signature>"#
    )
}

/// Inject `<ofd:Signatures>` element into OFD.xml content.
///
/// If a `<ofd:Signatures>` element already exists it is replaced; otherwise
/// the element is inserted before `</ofd:DocBody>` or `</ofd:OFD>`.
fn inject_signatures_element(ofd_xml: &str, signature_refs: &[String]) -> String {
    let mut signatures = String::new();
    signatures.push_str("  <ofd:Signatures>\n");
    for ref_path in signature_refs {
        use std::fmt::Write as _;
        let _ = writeln!(
            signatures,
            "    <ofd:SignatureRef>{ref_path}</ofd:SignatureRef>"
        );
    }
    signatures.push_str("  </ofd:Signatures>\n");

    // Replace existing Signatures element if present.
    if let Some(start) = ofd_xml.find("<ofd:Signatures")
        && let Some(end) = ofd_xml[start..].find("</ofd:Signatures>")
    {
        let abs_end = start + end + "</ofd:Signatures>".len();
        let mut result = ofd_xml[..start].to_string();
        result.push_str(&signatures);
        result.push_str(&ofd_xml[abs_end..]);
        return result;
    }

    // Insert before </ofd:DocBody> or </ofd:OFD>.
    if let Some(pos) = ofd_xml.find("</ofd:DocBody>") {
        let mut result = ofd_xml[..pos].to_string();
        result.push_str(&signatures);
        result.push_str(&ofd_xml[pos..]);
        result
    } else if let Some(pos) = ofd_xml.find("</ofd:OFD>") {
        let mut result = ofd_xml[..pos].to_string();
        result.push_str(&signatures);
        result.push_str(&ofd_xml[pos..]);
        result
    } else {
        let mut result = ofd_xml.to_string();
        result.push_str(&signatures);
        result
    }
}

/// Core multi-signer signing implementation.
///
/// Produces a ZIP with multiple independent signatures.  Each signer gets its
/// own `Signature_<n>.xml`, `SignedInfo_<n>.xml`, and `SignedValue_<n>.dat`.
/// The OFD.xml is modified to include a `<ofd:Signatures>` listing all
/// `SignatureRef` entries.
///
/// Returns the ZIP bytes and the list of signature file paths.
#[allow(clippy::too_many_lines)]
pub(crate) fn sign_multiple_impl(
    entry_data: &[(String, Vec<u8>)],
    entries: &[SignerEntry],
    algorithm: SignatureAlgorithm,
) -> OfdResult<(Vec<u8>, Vec<String>)> {
    let provider = "easyofd-rust";
    let signature_method = match algorithm {
        SignatureAlgorithm::Sm2WithSm3 => "SM2WithSM3",
        SignatureAlgorithm::Sha256WithRsa => "SHA256WithRSA",
    };
    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string();

    let out = std::io::Cursor::new(Vec::new());
    let mut zip = ZipWriter::new(out);
    let opts = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    // Identify OFD.xml entries.
    let ofd_xml_names: Vec<String> = entry_data
        .iter()
        .filter(|(name, _)| name.ends_with("OFD.xml"))
        .map(|(name, _)| name.clone())
        .collect();

    // Pre-compute signature paths (predictable from signer count).
    let signature_refs: Vec<String> = (0..entries.len())
        .map(|idx| format!("Doc_0/Signs/Signature_{idx}.xml"))
        .collect();

    // Build modified entry_data that includes the modified OFD.xml
    // (with Signatures element). This ensures FileRef checksums cover
    // the final OFD.xml content that will appear in the output ZIP.
    let mut modified_entry_data: Vec<(String, Vec<u8>)> = Vec::with_capacity(entry_data.len());
    for (name, data) in entry_data {
        if ofd_xml_names.contains(name) {
            let ofd_xml = String::from_utf8_lossy(data);
            let modified = inject_signatures_element(&ofd_xml, &signature_refs);
            modified_entry_data.push((name.clone(), modified.into_bytes()));
        } else {
            modified_entry_data.push((name.clone(), data.clone()));
        }
    }

    // 1. Copy all original entries except OFD.xml.
    for (name, data) in entry_data {
        if ofd_xml_names.contains(name) {
            continue;
        }
        zip.start_file(name, opts)
            .map_err(|e| easyofd_core::OfdError::Zip(e.to_string()))?;
        zip.write_all(data).map_err(easyofd_core::OfdError::Io)?;
    }

    // 2. For each signer: build SignedInfo, SM2-sign, write signature files.
    for (idx, entry) in entries.iter().enumerate() {
        // FileRef list for SignedInfo — uses modified_entry_data so that
        // the OFD.xml FileRef covers the final content (with Signatures).
        let mut file_refs = String::new();
        for (name, data) in &modified_entry_data {
            use std::fmt::Write as _;
            let _ = write!(
                file_refs,
                r#"<ofd:FileRef CheckMethod="SM3" CheckValue="{}">{}</ofd:FileRef>"#,
                hex(&compute_sm3(data)),
                xml_escape(name)
            );
        }

        let signed_info_xml = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<ofd:SignedInfo xmlns:ofd="http://www.ofdspec.org/2016">
  <ofd:Provider Version="1.0">{provider}</ofd:Provider>
  <ofd:SignatureMethod>{signature_method}</ofd:SignatureMethod>
  <ofd:SignatureDateTime>{now}</ofd:SignatureDateTime>
  <ofd:SealCount>{}</ofd:SealCount>
  <ofd:References>
    {file_refs}
  </ofd:References>
</ofd:SignedInfo>"#,
            entry.seals.len(),
        );

        // SM2 sign.
        use sm2::dsa::signature::Signer;
        let signing_key = sm2::dsa::SigningKey::new("1234567812345678", &entry.secret_key)
            .map_err(|e| easyofd_core::OfdError::Conversion(format!("SM2 密钥派生失败: {e}")))?;
        let signed_info_bytes = signed_info_xml.as_bytes();
        let sig = signing_key.sign(signed_info_bytes);
        let sig_hex = hex(&sig.to_bytes());
        let pub_hex = hex(&signing_key.verifying_key().to_sec1_bytes());

        // Seal images (indexed by signer and seal).
        for (i, seal) in entry.seals.iter().enumerate() {
            zip.start_file(format!("Doc_0/Res/Seal_{idx}_{i}.png"), opts)
                .map_err(|e| easyofd_core::OfdError::Zip(e.to_string()))?;
            zip.write_all(&seal.image_data)
                .map_err(easyofd_core::OfdError::Io)?;
        }

        // SignedValue_<n>.dat
        zip.start_file(format!("Doc_0/Signs/SignedValue_{idx}.dat"), opts)
            .map_err(|e| easyofd_core::OfdError::Zip(e.to_string()))?;
        zip.write_all(&sig.to_bytes())
            .map_err(easyofd_core::OfdError::Io)?;

        // SignedInfo_<n>.xml
        zip.start_file(format!("Doc_0/Signs/SignedInfo_{idx}.xml"), opts)
            .map_err(|e| easyofd_core::OfdError::Zip(e.to_string()))?;
        zip.write_all(signed_info_bytes)
            .map_err(easyofd_core::OfdError::Io)?;

        // Signature_<n>.xml
        let signature_xml =
            build_indexed_signature_xml(&signed_info_xml, &entry.seals, idx, &pub_hex, &sig_hex);
        zip.start_file(format!("Doc_0/Signs/Signature_{idx}.xml"), opts)
            .map_err(|e| easyofd_core::OfdError::Zip(e.to_string()))?;
        zip.write_all(signature_xml.as_bytes())
            .map_err(easyofd_core::OfdError::Io)?;
    }

    // 3. Write modified OFD.xml with Signatures element.
    for ofd_name in &ofd_xml_names {
        if let Some((_, data)) = modified_entry_data
            .iter()
            .find(|(name, _)| name == ofd_name)
        {
            zip.start_file(ofd_name, opts)
                .map_err(|e| easyofd_core::OfdError::Zip(e.to_string()))?;
            zip.write_all(data).map_err(easyofd_core::OfdError::Io)?;
        }
    }

    let data = zip
        .finish()
        .map_err(|e| easyofd_core::OfdError::Zip(e.to_string()))?
        .into_inner();

    Ok((data, signature_refs))
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── extract_between ────────────────────────────────────────────────

    #[test]
    fn extract_between_finds_content() {
        let xml = "<a>hello</a>";
        assert_eq!(extract_between(xml, "<a>", "</a>"), Some("hello".into()));
    }

    #[test]
    fn extract_between_returns_none_when_missing() {
        assert_eq!(extract_between("<a>x</a>", "<b>", "</b>"), None);
    }

    #[test]
    fn extract_between_returns_none_for_unclosed() {
        assert_eq!(extract_between("<a>hello", "<a>", "</a>"), None);
    }

    #[test]
    fn extract_between_empty_content() {
        assert_eq!(
            extract_between("<a></a>", "<a>", "</a>"),
            Some(String::new())
        );
    }

    // ── inject_signatures_element ──────────────────────────────────────

    #[test]
    fn inject_inserts_before_doc_body() {
        let ofd = "<?xml version=\"1.0\"?><ofd:OFD><ofd:DocBody></ofd:DocBody></ofd:OFD>";
        let refs = vec!["Doc_0/Signs/Signature_0.xml".into()];
        let result = inject_signatures_element(ofd, &refs);
        assert!(result.contains("<ofd:Signatures>"));
        assert!(result.contains("Signature_0.xml"));
        // Signatures must appear before DocBody closing tag.
        let sig_pos = result.find("<ofd:Signatures>").unwrap();
        let body_pos = result.find("</ofd:DocBody>").unwrap();
        assert!(sig_pos < body_pos);
    }

    #[test]
    fn inject_inserts_before_ofd_close_when_no_doc_body() {
        let ofd = "<?xml version=\"1.0\"?><ofd:OFD></ofd:OFD>";
        let refs = vec!["Sig.xml".into()];
        let result = inject_signatures_element(ofd, &refs);
        assert!(result.contains("<ofd:Signatures>"));
        let sig_pos = result.find("<ofd:Signatures>").unwrap();
        let ofd_pos = result.find("</ofd:OFD>").unwrap();
        assert!(sig_pos < ofd_pos);
    }

    #[test]
    fn inject_appends_when_no_closing_tags() {
        let ofd = "<root/>";
        let refs = vec!["Sig.xml".into()];
        let result = inject_signatures_element(ofd, &refs);
        assert!(result.contains("<ofd:Signatures>"));
        assert!(result.ends_with("</ofd:Signatures>\n"));
    }

    #[test]
    fn inject_replaces_existing_signatures() {
        let ofd = "<ofd:OFD><ofd:Signatures>\n<ofd:SignatureRef>old.xml</ofd:SignatureRef>\n</ofd:Signatures></ofd:OFD>";
        let refs = vec!["new.xml".into()];
        let result = inject_signatures_element(ofd, &refs);
        assert!(result.contains("new.xml"));
        assert!(!result.contains("old.xml"));
    }

    // ── build_indexed_signature_xml ────────────────────────────────────

    #[test]
    fn build_indexed_signature_xml_contains_expected_fields() {
        let signed_info = r#"<?xml version="1.0"?>
<ofd:SignedInfo>
  <ofd:Provider Version="1.0">easyofd-rust</ofd:Provider>
  <ofd:SignatureMethod>SM2WithSM3</ofd:SignatureMethod>
  <ofd:SignatureDateTime>2026-01-01T00:00:00</ofd:SignatureDateTime>
</ofd:SignedInfo>"#;
        let seals: Vec<ElectronicSeal> = vec![];
        let xml = build_indexed_signature_xml(signed_info, &seals, 0, "AABB", "CCDD");
        assert!(xml.contains("Signature_0"), "xml: {xml}");
        assert!(xml.contains("SignedInfo_0.xml"), "xml: {xml}");
        assert!(xml.contains("SignedValue_0.dat"), "xml: {xml}");
        assert!(xml.contains("AABB"), "pub key missing: {xml}");
        assert!(xml.contains("CCDD"), "sig value missing: {xml}");
        assert!(xml.contains("SM2WithSM3"), "method missing: {xml}");
        assert!(xml.contains("easyofd-rust"), "provider missing: {xml}");
    }

    #[test]
    fn build_indexed_signature_xml_with_seals() {
        let signed_info = r#"<?xml version="1.0"?>
<ofd:SignedInfo>
  <ofd:SignatureMethod>SM2WithSM3</ofd:SignatureMethod>
  <ofd:SignatureDateTime>2026-01-01T00:00:00</ofd:SignatureDateTime>
</ofd:SignedInfo>"#;
        let seals = vec![
            ElectronicSeal {
                image_data: vec![0x89],
                name: "S1".into(),
                position: (0.0, 0.0),
                page: 0,
            },
            ElectronicSeal {
                image_data: vec![0x50],
                name: "S2".into(),
                position: (1.0, 1.0),
                page: 0,
            },
        ];
        let xml = build_indexed_signature_xml(signed_info, &seals, 1, "AA", "BB");
        assert!(xml.contains("Seal_1_0"), "seal0 missing: {xml}");
        assert!(xml.contains("Seal_1_1"), "seal1 missing: {xml}");
    }

    #[test]
    fn build_indexed_signature_xml_defaults_method_when_missing() {
        let signed_info = "<root/>";
        let xml = build_indexed_signature_xml(signed_info, &[], 5, "AA", "BB");
        assert!(
            xml.contains("SM2WithSM3"),
            "should default to SM2WithSM3: {xml}"
        );
    }

    // ── sign_multiple_impl ─────────────────────────────────────────────

    #[test]
    fn sign_multiple_impl_produces_valid_zip() {
        use sm2::elliptic_curve::Generate;

        let entry_data = vec![
            (
                "OFD.xml".into(),
                b"<ofd:OFD><ofd:DocBody></ofd:DocBody></ofd:OFD>".to_vec(),
            ),
            ("Doc_0/Document.xml".into(), b"<doc/>".to_vec()),
        ];
        let entries: Vec<SignerEntry> = (0..2)
            .map(|_| SignerEntry {
                secret_key: sm2::SecretKey::generate(),
                seals: vec![ElectronicSeal {
                    image_data: vec![0x89, 0x50],
                    name: "S".into(),
                    position: (0.0, 0.0),
                    page: 0,
                }],
            })
            .collect();

        let (zip_data, refs) =
            sign_multiple_impl(&entry_data, &entries, SignatureAlgorithm::Sm2WithSm3).unwrap();

        assert_eq!(refs.len(), 2);
        assert!(refs[0].contains("Signature_0.xml"));
        assert!(refs[1].contains("Signature_1.xml"));

        // Verify ZIP structure.
        let cur = std::io::Cursor::new(&zip_data);
        let mut archive = zip::ZipArchive::new(cur).unwrap();
        let names: Vec<String> = (0..archive.len())
            .map(|i| archive.by_index(i).unwrap().name().to_string())
            .collect();
        assert!(names.contains(&"Doc_0/Signs/Signature_0.xml".to_string()));
        assert!(names.contains(&"Doc_0/Signs/Signature_1.xml".to_string()));
        assert!(names.contains(&"Doc_0/Signs/SignedInfo_0.xml".to_string()));
        assert!(names.contains(&"Doc_0/Signs/SignedValue_0.dat".to_string()));
        assert!(names.contains(&"Doc_0/Res/Seal_0_0.png".to_string()));

        // OFD.xml must contain Signatures element.
        let mut ofd_file = archive.by_name("OFD.xml").unwrap();
        let mut ofd_content = String::new();
        std::io::Read::read_to_string(&mut ofd_file, &mut ofd_content).unwrap();
        assert!(ofd_content.contains("<ofd:Signatures>"));
    }
}
