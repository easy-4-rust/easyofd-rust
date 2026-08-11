//! Cross-implementation comparison: ofdrw (Java) expected output vs easyofd-rust.
//!
//! This module validates that `easyofd-rust` correctly reads OFD files produced
//! by the [ofdrw](https://github.com/ofdrw/ofdrw) Java implementation by
//! comparing structural metadata against pre-generated baselines and
//! hand-crafted expected XML fragments.
//!
//! ## Why no true byte-level comparison?
//!
//! A genuine bidirectional byte-level comparison requires running the ofdrw
//! Java library on the same inputs and comparing its binary output (OFD/PDF)
//! against ours.  This requires a JDK + Maven environment, which is not
//! available in the current CI setup.  Instead, we:
//!
//! 1. **Baseline conformance**: Compare extracted metadata (page count, image
//!    count, path count, signature presence, text hash) against JSON baselines
//!    generated from the original ofdrw-produced fixtures.
//! 2. **XML structural compliance**: Extract raw XML from the OFD ZIP and
//!    verify that key OFD-spec elements and attributes are present and correct.
//! 3. **Expected XML fragments**: Compare critical XML fragments against
//!    hand-crafted expected strings derived from the GB/T 33190-2016 spec.
//!
//! When a JDK becomes available in CI, these tests should be supplemented
//! with true ofdrw-produced expected binaries.

use std::io::Read;
use std::path::{Path, PathBuf};

use easyofd_core::ContentObject;
use easyofd_reader::OfdReader;

// ─── Path helpers ──────────────────────────────────────────────────────────────

/// Path to the workspace `tests/fixtures/real_ofd/` directory.
fn fixture_dir() -> PathBuf {
    let manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    Path::new(&manifest)
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("tests")
        .join("fixtures")
        .join("real_ofd")
}

/// Path to the workspace `tests/fixtures/baseline/` directory.
fn baseline_dir() -> PathBuf {
    let manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    Path::new(&manifest)
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("tests")
        .join("fixtures")
        .join("baseline")
}

fn fixture_path(name: &str) -> PathBuf {
    fixture_dir().join(name)
}

// ─── ZIP/XML extraction helpers ────────────────────────────────────────────────

/// Read a named entry from an OFD ZIP archive and return its bytes.
fn read_zip_entry(ofd_path: &Path, entry_name: &str) -> Vec<u8> {
    let data = std::fs::read(ofd_path).unwrap_or_else(|e| {
        panic!("cannot read {}: {e}", ofd_path.display());
    });
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(data))
        .unwrap_or_else(|e| panic!("{} is not a valid ZIP: {e}", ofd_path.display()));
    let mut file = archive
        .by_name(entry_name)
        .unwrap_or_else(|e| panic!("{entry_name} not found in {}: {e}", ofd_path.display()));
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)
        .unwrap_or_else(|e| panic!("cannot read {entry_name}: {e}"));
    buf
}

/// Read a named entry from an OFD ZIP archive as a UTF-8 string.
fn read_zip_entry_as_string(ofd_path: &Path, entry_name: &str) -> String {
    let bytes = read_zip_entry(ofd_path, entry_name);
    String::from_utf8(bytes).unwrap_or_else(|e| panic!("{entry_name} is not valid UTF-8: {e}"))
}

/// List all entry names in an OFD ZIP archive.
fn list_zip_entries(ofd_path: &Path) -> Vec<String> {
    let data = std::fs::read(ofd_path).unwrap_or_else(|e| {
        panic!("cannot read {}: {e}", ofd_path.display());
    });
    let archive = zip::ZipArchive::new(std::io::Cursor::new(data))
        .unwrap_or_else(|e| panic!("{} is not a valid ZIP: {e}", ofd_path.display()));
    archive
        .file_names()
        .map(std::string::ToString::to_string)
        .collect()
}

// ─── Baseline JSON types ──────────────────────────────────────────────────────

#[derive(Debug, serde::Deserialize)]
#[allow(dead_code)] // `fixture` is used for deserialization only
struct BaselineEntry {
    fixture: String,
    page_count: usize,
    text_content_hash: String,
    image_count: usize,
    path_count: usize,
    signature_present: bool,
}

fn load_baseline(fixture_name: &str) -> BaselineEntry {
    let out_name = format!("expected_{}.json", fixture_name.replace(".ofd", ""));
    let path = baseline_dir().join(&out_name);
    let data = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("cannot read baseline {}: {e}", path.display()));
    serde_json::from_str(&data)
        .unwrap_or_else(|e| panic!("invalid baseline JSON {}: {e}", path.display()))
}

// ─── Content counting helpers ──────────────────────────────────────────────────

fn count_images(reader: &OfdReader) -> usize {
    reader
        .pages()
        .iter()
        .flat_map(|p| &p.content)
        .filter(|c| matches!(c, ContentObject::Image(_)))
        .count()
}

fn count_paths(reader: &OfdReader) -> usize {
    reader
        .pages()
        .iter()
        .flat_map(|p| &p.content)
        .filter(|c| matches!(c, ContentObject::Path(_)))
        .count()
}

/// Detect whether the OFD ZIP contains a `Signs/` directory.
fn has_signature(ofd_path: &Path) -> bool {
    let entries = list_zip_entries(ofd_path);
    entries.iter().any(|n| n.contains("Signs/"))
}

/// Compute a stable hash hex string of the input.
fn stable_hash_hex(data: &[u8]) -> String {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    use std::hash::{Hash, Hasher};
    data.hash(&mut hasher);
    let h = hasher.finish();
    format!("{h:016x}")
}

// ═══════════════════════════════════════════════════════════════════════════════
// Test 1: simple_1.ofd OFD.xml matches expected XML structure
// ═══════════════════════════════════════════════════════════════════════════════

/// Verify that `simple_1.ofd` contains the OFD-spec-required XML elements.
///
/// This test extracts the raw `OFD.xml` from the ZIP and checks for:
/// - The `xmlns:ofd="http://www.ofdspec.org/2016"` namespace
/// - `DocType="OFD"` attribute
/// - `Version="1.0"` attribute
/// - `<ofd:DocRoot>Doc_0/Document.xml</ofd:DocRoot>` element
/// - `<ofd:DocBody>` wrapper element
#[test]
fn test_simple_1_ofd_xml_matches_expected_structure() {
    let path = fixture_path("simple_1.ofd");
    let xml = read_zip_entry_as_string(&path, "OFD.xml");

    // Namespace declaration
    assert!(
        xml.contains("xmlns:ofd=\"http://www.ofdspec.org/2016\""),
        "OFD.xml missing ofd namespace"
    );

    // DocType attribute
    assert!(
        xml.contains("DocType=\"OFD\""),
        "OFD.xml missing DocType=\"OFD\""
    );

    // Version attribute
    assert!(
        xml.contains("Version=\"1.0\""),
        "OFD.xml missing Version=\"1.0\""
    );

    // DocBody element
    assert!(
        xml.contains("<ofd:DocBody>"),
        "OFD.xml missing <ofd:DocBody>"
    );

    // DocRoot pointing to Document.xml
    assert!(
        xml.contains("<ofd:DocRoot>Doc_0/Document.xml</ofd:DocRoot>"),
        "OFD.xml DocRoot does not point to Doc_0/Document.xml"
    );

    // DocInfo with DocID
    assert!(xml.contains("<ofd:DocID>"), "OFD.xml missing <ofd:DocID>");
}

// ═══════════════════════════════════════════════════════════════════════════════
// Test 2: multi_page_image Document.xml has 5 pages and correct structure
// ═══════════════════════════════════════════════════════════════════════════════

/// Verify that `multi_page_image.ofd` Document.xml contains 5 page entries
/// with correct `BaseLoc` attributes and the `CommonData` section.
#[test]
fn test_multi_page_image_document_xml_page_count() {
    let path = fixture_path("multi_page_image.ofd");
    let xml = read_zip_entry_as_string(&path, "Doc_0/Document.xml");

    // Must contain CommonData
    assert!(
        xml.contains("<ofd:CommonData>"),
        "Document.xml missing <ofd:CommonData>"
    );
    assert!(
        xml.contains("<ofd:PhysicalBox>"),
        "Document.xml missing <ofd:PhysicalBox>"
    );

    // Count page entries.  The pattern "<ofd:Page " does NOT match inside
    // "<ofd:TemplatePage " because the '<ofd:' prefix is followed by 'T'
    // in TemplatePage but by 'P' in Page, so they are distinct substrings.
    let actual_pages = xml.matches("<ofd:Page ").count();
    assert_eq!(
        actual_pages, 5,
        "expected 5 pages in multi_page_image Document.xml, found {actual_pages}"
    );

    // Verify BaseLoc references for all 5 pages
    for i in 0..5 {
        let expected = format!("Pages/Page_{i}/Content.xml");
        assert!(
            xml.contains(&expected),
            "Document.xml missing BaseLoc reference to {expected}"
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Test 3: signed.ofd contains Signature.xml with required elements
// ═══════════════════════════════════════════════════════════════════════════════

/// Verify that `signed.ofd` contains a `Signature.xml` with the OFD-spec
/// required elements: `SignedInfo`, `SignatureMethod`, `References`,
/// `SignedValue`.
#[test]
fn test_signed_signature_xml_has_required_elements() {
    let path = fixture_path("signed.ofd");

    // Verify Signs/ directory exists
    assert!(
        has_signature(&path),
        "signed.ofd must contain a Signs/ directory"
    );

    // Verify Signatures.xml exists and contains a Signature reference
    let sig_xml = read_zip_entry_as_string(&path, "Doc_0/Signs/Signatures.xml");
    assert!(
        sig_xml.contains("<ofd:Signature "),
        "Signatures.xml must contain <ofd:Signature> element"
    );
    assert!(
        sig_xml.contains("BaseLoc="),
        "Signatures.xml Signature must have BaseLoc attribute"
    );

    // Verify Signature.xml contains required elements
    let signature_xml = read_zip_entry_as_string(&path, "Doc_0/Signs/Sign_0/Signature.xml");
    assert!(
        signature_xml.contains("<ofd:SignedInfo>"),
        "Signature.xml missing <ofd:SignedInfo>"
    );
    assert!(
        signature_xml.contains("<ofd:SignatureMethod>"),
        "Signature.xml missing <ofd:SignatureMethod>"
    );
    assert!(
        signature_xml.contains("<ofd:SignatureDateTime>"),
        "Signature.xml missing <ofd:SignatureDateTime>"
    );
    assert!(
        signature_xml.contains("<ofd:References"),
        "Signature.xml missing <ofd:References>"
    );
    assert!(
        signature_xml.contains("<ofd:Reference "),
        "Signature.xml missing <ofd:Reference> entries"
    );
    assert!(
        signature_xml.contains("<ofd:CheckValue>"),
        "Signature.xml missing <ofd:CheckValue>"
    );
    assert!(
        signature_xml.contains("<ofd:SignedValue>"),
        "Signature.xml missing <ofd:SignedValue>"
    );
    assert!(
        signature_xml.contains("SignedValue.dat"),
        "Signature.xml SignedValue must reference SignedValue.dat"
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// Test 4: All 5 fixtures page_count matches baseline JSON
// ═══════════════════════════════════════════════════════════════════════════════

/// For each of the 5 real-world fixtures, verify that the page count extracted
/// by `OfdReader` matches the pre-generated baseline JSON.
#[test]
fn test_all_fixtures_page_count_matches_baseline() {
    let fixtures = [
        "simple_1.ofd",
        "simple_2.ofd",
        "multi_page_image.ofd",
        "signed.ofd",
        "with_table.ofd",
    ];

    for name in &fixtures {
        let path = fixture_path(name);
        if !path.exists() {
            eprintln!("SKIP: {name} not found");
            continue;
        }
        let baseline = load_baseline(name);
        let reader =
            OfdReader::open(&path).unwrap_or_else(|e| panic!("failed to parse {name}: {e}"));

        assert_eq!(
            baseline.page_count,
            reader.page_count(),
            "{name}: page_count mismatch vs baseline (expected {}, got {})",
            baseline.page_count,
            reader.page_count()
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Test 5: OFD.xml namespace compliance across all fixtures
// ═══════════════════════════════════════════════════════════════════════════════

/// Verify that every fixture's `OFD.xml` declares the correct OFD namespace
/// and contains `DocType="OFD"`.
#[test]
fn test_ofd_xml_namespace_compliance() {
    let fixtures = [
        "simple_1.ofd",
        "simple_2.ofd",
        "multi_page_image.ofd",
        "signed.ofd",
        "with_table.ofd",
    ];

    for name in &fixtures {
        let path = fixture_path(name);
        if !path.exists() {
            eprintln!("SKIP: {name} not found");
            continue;
        }
        let xml = read_zip_entry_as_string(&path, "OFD.xml");

        assert!(
            xml.contains("xmlns:ofd=\"http://www.ofdspec.org/2016\""),
            "{name}: OFD.xml missing xmlns:ofd namespace declaration"
        );
        assert!(
            xml.contains("DocType=\"OFD\""),
            "{name}: OFD.xml missing DocType=\"OFD\""
        );
        assert!(
            xml.contains("Version=\""),
            "{name}: OFD.xml missing Version attribute"
        );
        assert!(
            xml.contains("<ofd:DocRoot>"),
            "{name}: OFD.xml missing <ofd:DocRoot>"
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Test 6: Document.xml has page areas across all fixtures
// ═══════════════════════════════════════════════════════════════════════════════

/// Verify that every fixture's `Document.xml` contains the `CommonData`
/// section and a `Pages` section with page entries.
///
/// Note: `PageArea`/`PhysicalBox` are optional per GB/T 33190-2016 (defaults
/// to A4 210x297 mm).  Some fixtures (e.g. `with_table.ofd`) omit them.
#[test]
fn test_document_xml_has_page_areas() {
    let fixtures = [
        "simple_1.ofd",
        "simple_2.ofd",
        "multi_page_image.ofd",
        "signed.ofd",
        "with_table.ofd",
    ];

    for name in &fixtures {
        let path = fixture_path(name);
        if !path.exists() {
            eprintln!("SKIP: {name} not found");
            continue;
        }
        let xml = read_zip_entry_as_string(&path, "Doc_0/Document.xml");

        // CommonData and Pages are mandatory in every OFD document.
        assert!(
            xml.contains("<ofd:CommonData>"),
            "{name}: Document.xml missing <ofd:CommonData>"
        );
        assert!(
            xml.contains("<ofd:Pages>"),
            "{name}: Document.xml missing <ofd:Pages>"
        );
        assert!(
            xml.contains("<ofd:Page "),
            "{name}: Document.xml missing <ofd:Page> entries"
        );
    }

    // Fixtures known to have explicit PageArea/PhysicalBox.
    let fixtures_with_page_area = [
        "simple_1.ofd",
        "simple_2.ofd",
        "multi_page_image.ofd",
        "signed.ofd",
    ];
    for name in &fixtures_with_page_area {
        let path = fixture_path(name);
        if !path.exists() {
            continue;
        }
        let xml = read_zip_entry_as_string(&path, "Doc_0/Document.xml");
        assert!(
            xml.contains("<ofd:PageArea>"),
            "{name}: Document.xml missing <ofd:PageArea>"
        );
        assert!(
            xml.contains("<ofd:PhysicalBox>"),
            "{name}: Document.xml missing <ofd:PhysicalBox>"
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Test 7: All 5 fixtures metadata matches baseline (full comparison)
// ═══════════════════════════════════════════════════════════════════════════════

/// For each of the 5 fixtures, verify all baseline metadata fields:
/// page_count, text_content_hash, image_count, path_count, signature_present.
#[test]
fn test_all_fixtures_full_metadata_matches_baseline() {
    let fixtures = [
        "simple_1.ofd",
        "simple_2.ofd",
        "multi_page_image.ofd",
        "signed.ofd",
        "with_table.ofd",
    ];

    for name in &fixtures {
        let path = fixture_path(name);
        if !path.exists() {
            eprintln!("SKIP: {name} not found");
            continue;
        }
        let baseline = load_baseline(name);
        let reader =
            OfdReader::open(&path).unwrap_or_else(|e| panic!("failed to parse {name}: {e}"));

        // Page count
        assert_eq!(
            baseline.page_count,
            reader.page_count(),
            "{name}: page_count mismatch"
        );

        // Text content hash
        let actual_hash = format!(
            "hash:{}",
            stable_hash_hex(reader.extract_all_text().as_bytes())
        );
        assert_eq!(
            baseline.text_content_hash, actual_hash,
            "{name}: text_content_hash mismatch"
        );

        // Image count
        assert_eq!(
            baseline.image_count,
            count_images(&reader),
            "{name}: image_count mismatch"
        );

        // Path count
        assert_eq!(
            baseline.path_count,
            count_paths(&reader),
            "{name}: path_count mismatch"
        );

        // Signature presence
        assert_eq!(
            baseline.signature_present,
            has_signature(&path),
            "{name}: signature_present mismatch"
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Test 8: Signed OFD Signatures.xml references Signature.xml correctly
// ═══════════════════════════════════════════════════════════════════════════════

/// Verify that both `signed.ofd` and `multi_page_image.ofd` have correctly
/// structured `Signatures.xml` files that reference `Signature.xml` with
/// proper `BaseLoc` and `ID` attributes.
#[test]
fn test_signed_ofd_signatures_references() {
    let signed_fixtures = ["signed.ofd", "multi_page_image.ofd"];

    for name in &signed_fixtures {
        let path = fixture_path(name);
        if !path.exists() {
            eprintln!("SKIP: {name} not found");
            continue;
        }
        let baseline = load_baseline(name);
        if !baseline.signature_present {
            eprintln!("SKIP: {name} has no signature");
            continue;
        }

        let sig_xml = read_zip_entry_as_string(&path, "Doc_0/Signs/Signatures.xml");

        // Must contain ofd namespace
        assert!(
            sig_xml.contains("xmlns:ofd=\"http://www.ofdspec.org/2016\""),
            "{name}: Signatures.xml missing ofd namespace"
        );

        // Must contain MaxSignId
        assert!(
            sig_xml.contains("<ofd:MaxSignId>"),
            "{name}: Signatures.xml missing <ofd:MaxSignId>"
        );

        // Must contain at least one Signature element with ID and BaseLoc
        assert!(
            sig_xml.contains("<ofd:Signature ") && sig_xml.contains("BaseLoc="),
            "{name}: Signatures.xml missing <ofd:Signature> with BaseLoc"
        );

        // The Signature entry must reference a Signature.xml file
        let entries = list_zip_entries(&path);
        let has_signature_xml = entries
            .iter()
            .any(|e| e.contains("Signs/") && e.ends_with("Signature.xml"));
        assert!(
            has_signature_xml,
            "{name}: ZIP must contain a Signature.xml under Signs/"
        );

        // SignedValue.dat must exist
        let has_signed_value = entries.iter().any(|e| e.contains("SignedValue.dat"));
        assert!(has_signed_value, "{name}: ZIP must contain SignedValue.dat");
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Test 9: PhysicalBox dimensions match expected values
// ═══════════════════════════════════════════════════════════════════════════════

/// Verify that the `PhysicalBox` dimensions in each fixture's `Document.xml`
/// match the expected values from the ofdrw-produced documents.
#[test]
fn test_physical_box_dimensions_match_expected() {
    // simple_1 and simple_2: A4 portrait (210 x 297 mm)
    for name in &["simple_1.ofd", "simple_2.ofd"] {
        let path = fixture_path(name);
        if !path.exists() {
            continue;
        }
        let xml = read_zip_entry_as_string(&path, "Doc_0/Document.xml");
        assert!(
            xml.contains("<ofd:PhysicalBox>0 0 210 297</ofd:PhysicalBox>"),
            "{name}: expected PhysicalBox 0 0 210 297"
        );
    }

    // multi_page_image and signed: invoice format (210 x 140 mm)
    for name in &["multi_page_image.ofd", "signed.ofd"] {
        let path = fixture_path(name);
        if !path.exists() {
            continue;
        }
        let xml = read_zip_entry_as_string(&path, "Doc_0/Document.xml");
        assert!(
            xml.contains("<ofd:PhysicalBox>0 0 210 140</ofd:PhysicalBox>"),
            "{name}: expected PhysicalBox 0 0 210 140"
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Test 10: OFD.xml Version attribute matches expected values
// ═══════════════════════════════════════════════════════════════════════════════

/// Verify that the `Version` attribute in each fixture's `OFD.xml` matches
/// the expected OFD spec version.
#[test]
fn test_ofd_xml_version_matches_expected() {
    // Most fixtures use Version="1.0"
    for name in &[
        "simple_1.ofd",
        "simple_2.ofd",
        "multi_page_image.ofd",
        "with_table.ofd",
    ] {
        let path = fixture_path(name);
        if !path.exists() {
            continue;
        }
        let xml = read_zip_entry_as_string(&path, "OFD.xml");
        assert!(
            xml.contains("Version=\"1.0\""),
            "{name}: expected Version=\"1.0\" in OFD.xml"
        );
    }

    // signed.ofd uses Version="1.1"
    let path = fixture_path("signed.ofd");
    if path.exists() {
        let xml = read_zip_entry_as_string(&path, "OFD.xml");
        assert!(
            xml.contains("Version=\"1.1\""),
            "signed.ofd: expected Version=\"1.1\" in OFD.xml"
        );
    }
}
