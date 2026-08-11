//! Bidirectional roundtrip comparison framework (P8).
//!
//! For each real-world OFD fixture this module verifies:
//!
//! 1. **Text preservation** -- read -> extract text -> write -> re-read ->
//!    extract text -> compare.
//! 2. **Page-count preservation** -- read -> page_count -> write -> re-read ->
//!    page_count -> compare.
//! 3. **Image-count preservation** -- read -> count images -> write -> re-read
//!    -> count images -> compare.
//! 4. **Path-count preservation** -- read -> count paths -> write -> re-read ->
//!    count paths -> compare.
//! 5. **Baseline conformance** -- the first read is compared against a
//!    pre-generated JSON baseline stored in `tests/fixtures/baseline/`.

use std::path::{Path, PathBuf};

use easyofd_core::ContentObject;
use easyofd_reader::OfdReader;
use easyofd_writer::OfdWriter;

// ─── Fixture helpers ──────────────────────────────────────────────────────────

/// Path to `tests/fixtures/real_ofd/` at the workspace root.
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

/// Path to `tests/fixtures/baseline/` at the workspace root.
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

// ─── Extraction helpers ───────────────────────────────────────────────────────

/// Total image objects across all pages.
fn count_images(reader: &OfdReader) -> usize {
    reader
        .pages()
        .iter()
        .flat_map(|p| &p.content)
        .filter(|c| matches!(c, ContentObject::Image(_)))
        .count()
}

/// Total path objects across all pages.
fn count_paths(reader: &OfdReader) -> usize {
    reader
        .pages()
        .iter()
        .flat_map(|p| &p.content)
        .filter(|c| matches!(c, ContentObject::Path(_)))
        .count()
}

/// Roundtrip: read from bytes -> write -> read back.
fn roundtrip(bytes: &[u8]) -> OfdReader {
    let reader1 = OfdReader::from_bytes(bytes).expect("initial read should succeed");
    let mut writer = OfdWriter::new();
    for page in reader1.pages() {
        writer.add_page(page.clone());
    }
    let written = writer.build().expect("roundtrip write should succeed");
    OfdReader::from_bytes(&written).expect("roundtrip re-read should succeed")
}

// ─── Baseline JSON types ─────────────────────────────────────────────────────

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct BaselineEntry {
    fixture: String,
    page_count: usize,
    text_content_hash: String,
    image_count: usize,
    path_count: usize,
    signature_present: bool,
}

/// Compute a stable hash hex string of the input.
fn stable_hash_hex(data: &[u8]) -> String {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    std::hash::Hash::hash(data, &mut hasher);
    let h = std::hash::Hasher::finish(&hasher);
    format!("{h:016x}")
}

/// Detect whether the OFD ZIP contains a `Signs/` directory (signature
/// present).  This is a lightweight check that does not parse the signature
/// XML.
fn has_signature(bytes: &[u8]) -> bool {
    let Ok(archive) = zip::ZipArchive::new(std::io::Cursor::new(bytes)) else {
        return false;
    };
    // Check if any entry path contains "Signs/" (case-insensitive on some
    // platforms, but OFD spec uses exact case).
    let names: Vec<String> = archive
        .file_names()
        .map(std::string::ToString::to_string)
        .collect();
    names.iter().any(|n| n.contains("Signs/"))
}

/// Build a `BaselineEntry` from the first-read of a fixture.
fn build_baseline(fixture_name: &str, bytes: &[u8]) -> BaselineEntry {
    let reader = OfdReader::open(fixture_dir().join(fixture_name))
        .unwrap_or_else(|e| panic!("failed to parse {fixture_name}: {e}"));
    let text = reader.extract_all_text();
    BaselineEntry {
        fixture: fixture_name.to_string(),
        page_count: reader.page_count(),
        text_content_hash: format!("hash:{}", stable_hash_hex(text.as_bytes())),
        image_count: count_images(&reader),
        path_count: count_paths(&reader),
        signature_present: has_signature(bytes),
    }
}

// ─── Baseline generation (run manually) ──────────────────────────────────────

/// Generate baseline JSON files for all fixtures.  This test is ignored by
/// default; run with `cargo test --test diff_compare generate_baselines
/// -- --ignored` to regenerate.
#[test]
#[ignore = "run manually to regenerate baseline JSON files"]
fn generate_baselines() {
    let dir = baseline_dir();
    std::fs::create_dir_all(&dir).expect("cannot create baseline dir");

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
        let bytes = std::fs::read(&path).unwrap();
        let entry = build_baseline(name, &bytes);
        let json = serde_json::to_string_pretty(&entry).unwrap();
        let out_name = format!("expected_{}.json", name.replace(".ofd", ""));
        let out_path = dir.join(&out_name);
        std::fs::write(&out_path, json).unwrap();
        println!("Wrote {}", out_path.display());
    }
}

// ─── Baseline loading ────────────────────────────────────────────────────────

fn load_baseline(fixture_name: &str) -> Option<BaselineEntry> {
    let out_name = format!("expected_{}.json", fixture_name.replace(".ofd", ""));
    let path = baseline_dir().join(&out_name);
    if !path.exists() {
        return None;
    }
    let data = std::fs::read_to_string(&path).ok()?;
    serde_json::from_str(&data).ok()
}

// ─── Roundtrip tests ─────────────────────────────────────────────────────────

// -- Text preservation --

#[test]
fn roundtrip_text_preserved_simple_1() {
    let path = fixture_path("simple_1.ofd");
    let bytes = std::fs::read(&path).unwrap();
    let reader1 = OfdReader::open(&path).unwrap();
    let text1 = reader1.extract_all_text();
    let reader2 = roundtrip(&bytes);
    let text2 = reader2.extract_all_text();
    assert_eq!(
        text1, text2,
        "text changed during roundtrip for simple_1.ofd"
    );
}

#[test]
fn roundtrip_text_preserved_simple_2() {
    let path = fixture_path("simple_2.ofd");
    let bytes = std::fs::read(&path).unwrap();
    let reader1 = OfdReader::open(&path).unwrap();
    let text1 = reader1.extract_all_text();
    let reader2 = roundtrip(&bytes);
    let text2 = reader2.extract_all_text();
    assert_eq!(
        text1, text2,
        "text changed during roundtrip for simple_2.ofd"
    );
}

#[test]
fn roundtrip_text_preserved_with_table() {
    let path = fixture_path("with_table.ofd");
    let bytes = std::fs::read(&path).unwrap();
    let reader1 = OfdReader::open(&path).unwrap();
    let text1 = reader1.extract_all_text();
    let reader2 = roundtrip(&bytes);
    let text2 = reader2.extract_all_text();
    assert_eq!(
        text1, text2,
        "text changed during roundtrip for with_table.ofd"
    );
}

// -- Page-count preservation --

#[test]
fn roundtrip_page_count_preserved_simple_1() {
    let path = fixture_path("simple_1.ofd");
    let bytes = std::fs::read(&path).unwrap();
    let reader1 = OfdReader::open(&path).unwrap();
    let count1 = reader1.page_count();
    let reader2 = roundtrip(&bytes);
    assert_eq!(
        count1,
        reader2.page_count(),
        "page count changed during roundtrip for simple_1.ofd"
    );
}

#[test]
fn roundtrip_page_count_preserved_multi_page_image() {
    let path = fixture_path("multi_page_image.ofd");
    let bytes = std::fs::read(&path).unwrap();
    let reader1 = OfdReader::open(&path).unwrap();
    let count1 = reader1.page_count();
    let reader2 = roundtrip(&bytes);
    assert_eq!(
        count1,
        reader2.page_count(),
        "page count changed during roundtrip for multi_page_image.ofd"
    );
}

// -- Image-count preservation --

#[test]
fn roundtrip_image_count_preserved_multi_page_image() {
    let path = fixture_path("multi_page_image.ofd");
    let bytes = std::fs::read(&path).unwrap();
    let reader1 = OfdReader::open(&path).unwrap();
    let count1 = count_images(&reader1);
    let reader2 = roundtrip(&bytes);
    assert_eq!(
        count1,
        count_images(&reader2),
        "image count changed during roundtrip for multi_page_image.ofd"
    );
}

#[test]
fn roundtrip_image_count_preserved_with_table() {
    let path = fixture_path("with_table.ofd");
    let bytes = std::fs::read(&path).unwrap();
    let reader1 = OfdReader::open(&path).unwrap();
    let count1 = count_images(&reader1);
    let reader2 = roundtrip(&bytes);
    assert_eq!(
        count1,
        count_images(&reader2),
        "image count changed during roundtrip for with_table.ofd"
    );
}

// -- Path-count preservation --

#[test]
fn roundtrip_path_count_preserved_with_table() {
    let path = fixture_path("with_table.ofd");
    let bytes = std::fs::read(&path).unwrap();
    let reader1 = OfdReader::open(&path).unwrap();
    let count1 = count_paths(&reader1);
    let reader2 = roundtrip(&bytes);
    assert_eq!(
        count1,
        count_paths(&reader2),
        "path count changed during roundtrip for with_table.ofd"
    );
}

// -- Baseline conformance tests --

fn assert_baseline(fixture_name: &str) {
    let Some(expected) = load_baseline(fixture_name) else {
        eprintln!("SKIP baseline: no expected JSON for {fixture_name}");
        return;
    };
    let path = fixture_path(fixture_name);
    let bytes = std::fs::read(&path).unwrap();
    let reader =
        OfdReader::open(&path).unwrap_or_else(|e| panic!("failed to parse {fixture_name}: {e}"));

    // Page count
    assert_eq!(
        expected.page_count,
        reader.page_count(),
        "{fixture_name}: page_count mismatch vs baseline"
    );

    // Text hash
    let actual_hash = format!(
        "hash:{}",
        stable_hash_hex(reader.extract_all_text().as_bytes())
    );
    assert_eq!(
        expected.text_content_hash, actual_hash,
        "{fixture_name}: text_content_hash mismatch vs baseline"
    );

    // Image count
    assert_eq!(
        expected.image_count,
        count_images(&reader),
        "{fixture_name}: image_count mismatch vs baseline"
    );

    // Path count
    assert_eq!(
        expected.path_count,
        count_paths(&reader),
        "{fixture_name}: path_count mismatch vs baseline"
    );

    // Signature presence
    assert_eq!(
        expected.signature_present,
        has_signature(&bytes),
        "{fixture_name}: signature_present mismatch vs baseline"
    );
}

#[test]
fn baseline_simple_1() {
    assert_baseline("simple_1.ofd");
}

#[test]
fn baseline_simple_2() {
    assert_baseline("simple_2.ofd");
}

#[test]
fn baseline_multi_page_image() {
    assert_baseline("multi_page_image.ofd");
}

#[test]
fn baseline_signed() {
    assert_baseline("signed.ofd");
}

#[test]
fn baseline_with_table() {
    assert_baseline("with_table.ofd");
}
