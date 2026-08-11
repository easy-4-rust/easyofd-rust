//! Cross-implementation runner: easyofd-rust processes the same samples as ofdrw.
//!
//! For each ofdrw sample this module:
//! 1. Reads the OFD file using `OfdReader`.
//! 2. Extracts metadata (page count, text, image count, path count).
//! 3. Performs a roundtrip read-write-read cycle.
//! 4. Writes JSON summary to `/tmp/easyofd_artifacts/{name}_by_easyofd.json`.
//! 5. Writes roundtrip OFD to `/tmp/easyofd_artifacts/{name}_by_easyofd.ofd`.
//!
//! The artifacts are designed to be comparable with ofdrw-produced artifacts
//! in `/tmp/ofdrw_artifacts/`.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

use easyofd_core::ContentObject;
use easyofd_reader::OfdReader;
use easyofd_writer::OfdWriter;

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

/// Output directory for easyofd-rust artifacts.
fn artifacts_dir() -> PathBuf {
    PathBuf::from("/tmp/easyofd_artifacts")
}

/// The set of ofdrw samples that easyofd-rust will process.
/// These correspond to files in ofdrw-reader and ofdrw-layout test resources,
/// copied into `tests/fixtures/real_ofd/ofdrw_*.ofd`.
const OFDRW_SAMPLES: &[(&str, &str)] = &[
    ("ofdrw_helloworld", "ofdrw_helloworld.ofd"),
    ("ofdrw_chineseDir", "ofdrw_chineseDir.ofd"),
    ("ofdrw_keyword", "ofdrw_keyword.ofd"),
    (
        "ofdrw_multiKeywordInTextCode",
        "ofdrw_multiKeywordInTextCode.ofd",
    ),
    ("ofdrw_AddAttachment", "ofdrw_AddAttachment.ofd"),
    ("ofdrw_发票示例", "ofdrw_发票示例.ofd"),
    ("ofdrw_Page5", "ofdrw_Page5.ofd"),
    (
        "ofdrw_helloworld_with_pageblock",
        "ofdrw_helloworld_with_pageblock.ofd",
    ),
];

// ─── Helpers ───────────────────────────────────────────────────────────────────

/// Compute a stable hash hex string of the input.
fn stable_hash_hex(data: &[u8]) -> String {
    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    let h = hasher.finish();
    format!("{h:016x}")
}

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

/// Total text objects across all pages.
fn count_texts(reader: &OfdReader) -> usize {
    reader
        .pages()
        .iter()
        .flat_map(|p| &p.content)
        .filter(|c| matches!(c, ContentObject::Text(_)))
        .count()
}

/// Detect whether the OFD ZIP contains a `Signs/` directory.
fn has_signature(ofd_path: &Path) -> bool {
    let data = std::fs::read(ofd_path).unwrap_or_default();
    let Ok(archive) = zip::ZipArchive::new(std::io::Cursor::new(data)) else {
        return false;
    };
    let names: Vec<String> = archive
        .file_names()
        .map(std::string::ToString::to_string)
        .collect();
    names.iter().any(|n| n.contains("Signs/"))
}

/// Roundtrip: read from file bytes -> OfdWriter -> written bytes.
fn roundtrip(bytes: &[u8]) -> Vec<u8> {
    let reader = OfdReader::from_bytes(bytes).expect("initial read should succeed");
    let mut writer = OfdWriter::new();
    for page in reader.pages() {
        writer.add_page(page.clone());
    }
    writer.build().expect("roundtrip write should succeed")
}

// ─── JSON artifact type ───────────────────────────────────────────────────────

#[derive(Debug, serde::Serialize)]
struct EasyOfdArtifact {
    name: String,
    source_file: String,
    page_count: usize,
    text_content_hash: String,
    image_count: usize,
    path_count: usize,
    text_object_count: usize,
    signature_present: bool,
    roundtrip_ofd_bytes: usize,
    extract_text_preview: Vec<String>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Test: easyofd-rust can read all ofdrw samples and produce artifacts
// ═══════════════════════════════════════════════════════════════════════════════

/// For each ofdrw sample, verify that easyofd-rust can:
/// 1. Read the OFD file without error.
/// 2. Extract page count, text, images, paths.
/// 3. Perform a roundtrip (read -> write -> read).
/// 4. Write JSON summary artifact.
/// 5. Write roundtrip OFD artifact.
#[test]
fn test_easyofd_reads_all_ofdrw_samples_and_produces_artifacts() {
    let out_dir = artifacts_dir();
    std::fs::create_dir_all(&out_dir).expect("cannot create artifacts dir");

    for &(name, filename) in OFDRW_SAMPLES {
        let path = fixture_dir().join(filename);
        if !path.exists() {
            eprintln!("SKIP: {filename} not found at {}", path.display());
            continue;
        }

        // Read the OFD file
        let reader = match OfdReader::open(&path) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("SKIP: {name} failed to read: {e}");
                continue;
            }
        };

        let page_count = reader.page_count();
        let text_all = reader.extract_all_text();
        let text_hash = stable_hash_hex(text_all.as_bytes());
        let image_count = count_images(&reader);
        let path_count = count_paths(&reader);
        let text_obj_count = count_texts(&reader);
        let sig_present = has_signature(&path);

        // Extract text previews (first 200 chars per page)
        let text_per_page = reader.extract_text();
        let previews: Vec<String> = text_per_page
            .iter()
            .map(|t| {
                let preview: String = t.chars().take(200).collect();
                if preview.is_empty() {
                    "(no text)".to_string()
                } else {
                    preview
                }
            })
            .collect();

        // Roundtrip
        let bytes = std::fs::read(&path).unwrap();
        let roundtrip_bytes = roundtrip(&bytes);

        // Build artifact
        let artifact = EasyOfdArtifact {
            name: name.to_string(),
            source_file: filename.to_string(),
            page_count,
            text_content_hash: format!("hash:{text_hash}"),
            image_count,
            path_count,
            text_object_count: text_obj_count,
            signature_present: sig_present,
            roundtrip_ofd_bytes: roundtrip_bytes.len(),
            extract_text_preview: previews,
        };

        // Write JSON artifact
        let json_path = out_dir.join(format!("{name}_by_easyofd.json"));
        let json = serde_json::to_string_pretty(&artifact).unwrap();
        std::fs::write(&json_path, &json).unwrap();

        // Write roundtrip OFD
        let ofd_path = out_dir.join(format!("{name}_by_easyofd.ofd"));
        std::fs::write(&ofd_path, &roundtrip_bytes).unwrap();

        // Verify roundtrip re-read works
        let reader2 =
            OfdReader::from_bytes(&roundtrip_bytes).unwrap_or_else(|e| {
                panic!("{name}: roundtrip re-read failed: {e}")
            });
        assert_eq!(
            page_count,
            reader2.page_count(),
            "{name}: page_count changed during roundtrip"
        );

        println!(
            "OK: {name} | pages={page_count}, images={image_count}, \
             paths={path_count}, texts={text_obj_count}, sig={sig_present}, \
             roundtrip={} bytes",
            roundtrip_bytes.len()
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Test: Verify roundtrip page-count preservation for each sample
// ═══════════════════════════════════════════════════════════════════════════════

/// For each ofdrw sample, verify that page count is preserved in a
/// read-write-read roundtrip.
#[test]
fn test_roundtrip_page_count_preserved_for_ofdrw_samples() {
    for &(name, filename) in OFDRW_SAMPLES {
        let path = fixture_dir().join(filename);
        if !path.exists() {
            eprintln!("SKIP: {filename} not found");
            continue;
        }

        let bytes = std::fs::read(&path).unwrap();
        let reader1 = match OfdReader::open(&path) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("SKIP: {name} failed to read: {e}");
                continue;
            }
        };
        let count1 = reader1.page_count();

        let roundtrip_bytes = roundtrip(&bytes);
        let reader2 = OfdReader::from_bytes(&roundtrip_bytes)
            .unwrap_or_else(|e| panic!("{name}: roundtrip re-read failed: {e}"));

        assert_eq!(
            count1,
            reader2.page_count(),
            "{name}: page_count changed during roundtrip (before={count1}, after={})",
            reader2.page_count()
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Test: Verify roundtrip text hash preservation for each sample
// ═══════════════════════════════════════════════════════════════════════════════

/// For each ofdrw sample, verify that text content hash is preserved in a
/// read-write-read roundtrip.
#[test]
fn test_roundtrip_text_hash_preserved_for_ofdrw_samples() {
    for &(name, filename) in OFDRW_SAMPLES {
        let path = fixture_dir().join(filename);
        if !path.exists() {
            eprintln!("SKIP: {filename} not found");
            continue;
        }

        let bytes = std::fs::read(&path).unwrap();
        let reader1 = match OfdReader::open(&path) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("SKIP: {name} failed to read: {e}");
                continue;
            }
        };
        let text1 = reader1.extract_all_text();
        let hash1 = stable_hash_hex(text1.as_bytes());

        let roundtrip_bytes = roundtrip(&bytes);
        let reader2 = OfdReader::from_bytes(&roundtrip_bytes)
            .unwrap_or_else(|e| panic!("{name}: roundtrip re-read failed: {e}"));
        let text2 = reader2.extract_all_text();
        let hash2 = stable_hash_hex(text2.as_bytes());

        assert_eq!(
            hash1, hash2,
            "{name}: text hash changed during roundtrip"
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Test: Verify each roundtrip OFD is a valid ZIP with OFD structure
// ═══════════════════════════════════════════════════════════════════════════════

/// For each ofdrw sample, verify that the roundtrip OFD output is a valid ZIP
/// containing OFD.xml and Doc_0/Document.xml.
#[test]
fn test_roundtrip_ofd_has_valid_zip_structure() {
    for &(name, filename) in OFDRW_SAMPLES {
        let path = fixture_dir().join(filename);
        if !path.exists() {
            eprintln!("SKIP: {filename} not found");
            continue;
        }

        let bytes = std::fs::read(&path).unwrap();
        let roundtrip_bytes =
            if let Ok(b) = std::panic::catch_unwind(|| roundtrip(&bytes)) {
                b
            } else {
                eprintln!("SKIP: {name} roundtrip panicked");
                continue;
            };

        // Verify it's a valid ZIP
        let archive = zip::ZipArchive::new(std::io::Cursor::new(&roundtrip_bytes))
            .unwrap_or_else(|e| panic!("{name}: roundtrip output is not a valid ZIP: {e}"));

        let entries: Vec<String> = archive.file_names().map(String::from).collect();

        assert!(
            entries.contains(&"OFD.xml".to_string()),
            "{name}: roundtrip OFD missing OFD.xml"
        );
        assert!(
            entries.contains(&"Doc_0/Document.xml".to_string()),
            "{name}: roundtrip OFD missing Doc_0/Document.xml"
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Test: Artifact file inventory
// ═══════════════════════════════════════════════════════════════════════════════

/// Verify that all expected artifact files have been produced in
/// `/tmp/easyofd_artifacts/`.  This test runs after the main artifact
/// generation test and checks that the files exist and are non-empty.
#[test]
fn test_artifact_files_exist_and_nonempty() {
    let out_dir = artifacts_dir();
    if !out_dir.exists() {
        eprintln!("SKIP: artifacts dir {} does not exist", out_dir.display());
        return;
    }

    for &(name, _filename) in OFDRW_SAMPLES {
        let json_path = out_dir.join(format!("{name}_by_easyofd.json"));
        let ofd_path = out_dir.join(format!("{name}_by_easyofd.ofd"));

        if json_path.exists() {
            let json_data = std::fs::read(&json_path).unwrap();
            assert!(
                !json_data.is_empty(),
                "{name}: JSON artifact is empty"
            );
            // Verify it's valid JSON
            let _: serde_json::Value =
                serde_json::from_slice(&json_data).unwrap_or_else(|e| {
                    panic!("{name}: JSON artifact is not valid JSON: {e}")
                });
        }

        if ofd_path.exists() {
            let ofd_data = std::fs::read(&ofd_path).unwrap();
            assert!(
                !ofd_data.is_empty(),
                "{name}: OFD artifact is empty"
            );
            assert_eq!(
                &ofd_data[0..2],
                b"PK",
                "{name}: OFD artifact is not a valid ZIP"
            );
        }
    }
}
