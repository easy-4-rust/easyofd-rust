// Copyright 2024 easy-4-rust contributors
// SPDX-License-Identifier: Apache-2.0

//! Field-by-field diff framework: compare ofdrw (Java) vs easyofd-rust outputs.
//!
//! ## Inputs
//!
//! - `/tmp/ofdrw_artifacts/{name}_by_ofdrw.ofd`
//! - `/tmp/ofdrw_artifacts/{name}_by_ofdrw.json`
//! - `/tmp/ofdrw_artifacts/{name}_by_ofdrw.pdf`
//! - `/tmp/easyofd_artifacts/ofdrw_{name}_by_easyofd.ofd`
//! - `/tmp/easyofd_artifacts/ofdrw_{name}_by_easyofd.json`
//! - `/tmp/easyofd_artifacts/ofdrw_{name}_by_easyofd.pdf`
//!
//! ## Comparison dimensions
//!
//! 1. **JSON summary** -- page_count, text count, image count, path count,
//!    signature presence.
//! 2. **OFD XML elements** -- decode each XML file inside the OFD ZIP and
//!    compare element-name distributions (counts per element).
//! 3. **PDF byte-level** -- rough byte length and PDF object count.
//! 4. **ZIP entry list** -- compare entry presence between ofdrw and easyofd.

use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};

use quick_xml::events::Event;
use quick_xml::reader::Reader;
use zip::ZipArchive;

// ─── Artifact directory helpers ────────────────────────────────────────────────

fn ofdrw_dir() -> &'static str {
    "/tmp/ofdrw_artifacts"
}

fn easyofd_dir() -> &'static str {
    "/tmp/easyofd_artifacts"
}

/// Returns `true` when both artifact directories exist.
fn artifacts_available() -> bool {
    Path::new(ofdrw_dir()).is_dir() && Path::new(easyofd_dir()).is_dir()
}

// ─── Sample discovery ─────────────────────────────────────────────────────────

/// Discover all sample names by scanning `ofdrw_dir` for `*_by_ofdrw.json` files.
fn discover_samples() -> Vec<String> {
    let mut samples = vec![];
    if let Ok(entries) = fs::read_dir(ofdrw_dir()) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with("_by_ofdrw.json") {
                samples.push(name.trim_end_matches("_by_ofdrw.json").to_string());
            }
        }
    }
    samples.sort();
    samples
}

// ─── Path helpers ─────────────────────────────────────────────────────────────

/// ofdrw path: `{ofdrw_dir}/{name}_by_ofdrw.{ext}`
fn ofdrw_path(name: &str, ext: &str) -> String {
    format!("{}/{name}_by_ofdrw.{ext}", ofdrw_dir())
}

/// easyofd path: `{easyofd_dir}/ofdrw_{name}_by_easyofd.{ext}`
fn easyofd_path(name: &str, ext: &str) -> String {
    format!("{}/ofdrw_{name}_by_easyofd.{ext}", easyofd_dir())
}

// ─── JSON helpers ──────────────────────────────────────────────────────────────

fn load_json(path: &str) -> serde_json::Value {
    let data = fs::read_to_string(path).unwrap_or_else(|_| "{}".to_string());
    serde_json::from_str(&data).unwrap_or_else(|_| serde_json::json!({}))
}

// ─── XML element counting ─────────────────────────────────────────────────────

/// Count occurrences of each element name in an XML string.
fn count_xml_elements(xml: &str) -> HashMap<String, usize> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut counts: HashMap<String, usize> = HashMap::new();
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e) | Event::Empty(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).into_owned();
                *counts.entry(name).or_insert(0) += 1;
            }
            Ok(Event::Eof) | Err(_) => break,
            _ => {}
        }
        buf.clear();
    }
    counts
}

// ─── OFD ZIP extraction ───────────────────────────────────────────────────────

/// Extract all XML files (except `META-INF/container.xml`) from an OFD ZIP.
fn extract_ofd_xml(zip_path: &str) -> HashMap<String, String> {
    let mut result = HashMap::new();
    let Ok(file) = fs::File::open(zip_path) else {
        return result;
    };
    let Ok(mut zip) = ZipArchive::new(file) else {
        return result;
    };
    for i in 0..zip.len() {
        if let Ok(mut entry) = zip.by_index(i) {
            let name = entry.name().to_string();
            if Path::new(&name)
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("xml"))
                && name != "META-INF/container.xml"
            {
                let mut data = String::new();
                if entry.read_to_string(&mut data).is_ok() {
                    result.insert(name, data);
                }
            }
        }
    }
    result
}

/// List all entry names in an OFD ZIP archive.
fn list_ofd_entries(ofd_path: &str) -> Vec<String> {
    let Ok(data) = fs::read(ofd_path) else {
        return vec![];
    };
    let Ok(archive) = ZipArchive::new(std::io::Cursor::new(data)) else {
        return vec![];
    };
    archive
        .file_names()
        .map(std::string::ToString::to_string)
        .collect()
}

// ─── Diff counters (shared across tests via atomics) ──────────────────────────

static TOTAL_SAMPLES: AtomicUsize = AtomicUsize::new(0);
static JSON_DIFFS: AtomicUsize = AtomicUsize::new(0);
static XML_DIFFS: AtomicUsize = AtomicUsize::new(0);
static PDF_DIFFS: AtomicUsize = AtomicUsize::new(0);

// ═══════════════════════════════════════════════════════════════════════════════
// Test 1: JSON summary consistency
// ═══════════════════════════════════════════════════════════════════════════════

/// Compare key metadata fields between ofdrw and easyofd JSON outputs.
///
/// Fields checked:
/// - `page_count`
/// - `text_count` / `texts` (number of text objects)
/// - `image_count`
/// - `path_count`
/// - `signature_present`
#[test]
fn test_json_summary_consistency() {
    if !artifacts_available() {
        eprintln!(
            "SKIP: artifact directories not found at {} and {}",
            ofdrw_dir(),
            easyofd_dir()
        );
        return;
    }

    let samples = discover_samples();
    assert!(!samples.is_empty(), "no samples discovered in {}", ofdrw_dir());

    for name in &samples {
        TOTAL_SAMPLES.fetch_add(1, Ordering::Relaxed);

        let ofdrw_json_path = ofdrw_path(name, "json");
        let easyofd_json_path = easyofd_path(name, "json");

        if !Path::new(&easyofd_json_path).exists() {
            eprintln!("SKIP [{name}]: easyofd JSON missing at {easyofd_json_path}");
            continue;
        }

        let ofdrw_json = load_json(&ofdrw_json_path);
        let easyofd_json = load_json(&easyofd_json_path);

        // -- page_count --
        let ofdrw_page = ofdrw_json
            .get("page_count")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0);
        let easyofd_page = easyofd_json
            .get("page_count")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0);
        if ofdrw_page != easyofd_page {
            JSON_DIFFS.fetch_add(1, Ordering::Relaxed);
            eprintln!("DIFF [{name}] page_count: ofdrw={ofdrw_page} easyofd={easyofd_page}");
        }

        // -- text_count (try both "text_count" and "texts" keys) --
        let ofdrw_text = ofdrw_json
            .get("text_count")
            .or_else(|| ofdrw_json.get("texts"))
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0);
        let easyofd_text = easyofd_json
            .get("text_count")
            .or_else(|| easyofd_json.get("texts"))
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0);
        if ofdrw_text != easyofd_text {
            JSON_DIFFS.fetch_add(1, Ordering::Relaxed);
            eprintln!("DIFF [{name}] text_count: ofdrw={ofdrw_text} easyofd={easyofd_text}");
        }

        // -- image_count --
        let ofdrw_img = ofdrw_json
            .get("image_count")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0);
        let easyofd_img = easyofd_json
            .get("image_count")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0);
        if ofdrw_img != easyofd_img {
            JSON_DIFFS.fetch_add(1, Ordering::Relaxed);
            eprintln!("DIFF [{name}] image_count: ofdrw={ofdrw_img} easyofd={easyofd_img}");
        }

        // -- path_count --
        let ofdrw_path_count = ofdrw_json
            .get("path_count")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0);
        let easyofd_path_count = easyofd_json
            .get("path_count")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0);
        if ofdrw_path_count != easyofd_path_count {
            JSON_DIFFS.fetch_add(1, Ordering::Relaxed);
            eprintln!("DIFF [{name}] path_count: ofdrw={ofdrw_path_count} easyofd={easyofd_path_count}");
        }

        // -- signature_present --
        let ofdrw_sig = ofdrw_json
            .get("signature_present")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);
        let easyofd_sig = easyofd_json
            .get("signature_present")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);
        if ofdrw_sig != easyofd_sig {
            JSON_DIFFS.fetch_add(1, Ordering::Relaxed);
            eprintln!("DIFF [{name}] signature_present: ofdrw={ofdrw_sig} easyofd={easyofd_sig}");
        }

        // At least one side must report pages
        assert!(
            ofdrw_page > 0 || easyofd_page > 0,
            "neither ofdrw nor easyofd reports pages for sample {name}"
        );
    }

    eprintln!(
        "\n=== JSON summary: {} samples processed, {} field diffs ===",
        TOTAL_SAMPLES.load(Ordering::Relaxed),
        JSON_DIFFS.load(Ordering::Relaxed)
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// Test 2: OFD XML element distribution comparison
// ═══════════════════════════════════════════════════════════════════════════════

/// For each XML file inside the OFD ZIP, compare the element-name count
/// distribution between ofdrw and easyofd outputs.
#[test]
fn test_xml_element_distribution() {
    if !artifacts_available() {
        eprintln!("SKIP: artifact directories not found");
        return;
    }

    let samples = discover_samples();
    assert!(!samples.is_empty(), "no samples discovered");

    for name in &samples {
        let ofdrw_ofd = ofdrw_path(name, "ofd");
        let easyofd_ofd = easyofd_path(name, "ofd");

        if !Path::new(&ofdrw_ofd).exists() {
            eprintln!("SKIP [{name}]: ofdrw OFD missing");
            continue;
        }
        if !Path::new(&easyofd_ofd).exists() {
            eprintln!("SKIP [{name}]: easyofd OFD missing at {easyofd_ofd}");
            continue;
        }

        let ofdrw_xmls = extract_ofd_xml(&ofdrw_ofd);
        let easyofd_xmls = extract_ofd_xml(&easyofd_ofd);

        // -- XMLs present in ofdrw --
        for (xml_name, ofdrw_xml) in &ofdrw_xmls {
            if let Some(easyofd_xml) = easyofd_xmls.get(xml_name) {
                let ofdrw_counts = count_xml_elements(ofdrw_xml);
                let easyofd_counts = count_xml_elements(easyofd_xml);

                // Report elements present in ofdrw but different in easyofd
                for (elem, count) in &ofdrw_counts {
                    let easyofd_count = easyofd_counts.get(elem).copied().unwrap_or(0);
                    if *count != easyofd_count {
                        XML_DIFFS.fetch_add(1, Ordering::Relaxed);
                        eprintln!(
                            "DIFF [{name}]/{xml_name} element {elem}: ofdrw={count} easyofd={easyofd_count}"
                        );
                    }
                }

                // Report elements only in easyofd
                for (elem, count) in &easyofd_counts {
                    if !ofdrw_counts.contains_key(elem) {
                        XML_DIFFS.fetch_add(1, Ordering::Relaxed);
                        eprintln!(
                            "DIFF [{name}]/{xml_name} element {elem} only in easyofd (count={count})"
                        );
                    }
                }
            } else {
                XML_DIFFS.fetch_add(1, Ordering::Relaxed);
                eprintln!("DIFF [{name}] XML {xml_name} only in ofdrw");
            }
        }

        // -- XMLs only in easyofd --
        for xml_name in easyofd_xmls.keys() {
            if !ofdrw_xmls.contains_key(xml_name) {
                XML_DIFFS.fetch_add(1, Ordering::Relaxed);
                eprintln!("DIFF [{name}] XML {xml_name} only in easyofd");
            }
        }
    }

    eprintln!(
        "\n=== XML element distribution: {} total element-level diffs ===",
        XML_DIFFS.load(Ordering::Relaxed)
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// Test 3: PDF byte-length and object-count comparison
// ═══════════════════════════════════════════════════════════════════════════════

/// Count PDF objects by scanning for `N 0 obj` patterns in raw bytes.
fn count_pdf_objects(pdf_bytes: &[u8]) -> usize {
    let text = String::from_utf8_lossy(pdf_bytes);
    text.lines()
        .filter(|line| {
            let trimmed = line.trim();
            trimmed.ends_with(" 0 obj")
        })
        .count()
}

/// Compare PDF byte lengths and rough object counts between ofdrw and easyofd.
#[test]
fn test_pdf_byte_and_object_comparison() {
    if !artifacts_available() {
        eprintln!("SKIP: artifact directories not found");
        return;
    }

    let samples = discover_samples();
    assert!(!samples.is_empty(), "no samples discovered");

    for name in &samples {
        let ofdrw_pdf = ofdrw_path(name, "pdf");
        let easyofd_pdf = easyofd_path(name, "pdf");

        let ofdrw_exists = Path::new(&ofdrw_pdf).exists();
        let easyofd_exists = Path::new(&easyofd_pdf).exists();

        if !ofdrw_exists && !easyofd_exists {
            eprintln!("SKIP [{name}]: PDF missing on both sides");
            continue;
        }

        let ofdrw_bytes = if ofdrw_exists {
            fs::read(&ofdrw_pdf).unwrap_or_default()
        } else {
            vec![]
        };
        let easyofd_bytes = if easyofd_exists {
            fs::read(&easyofd_pdf).unwrap_or_default()
        } else {
            vec![]
        };

        let ofdrw_size = ofdrw_bytes.len();
        let easyofd_size = easyofd_bytes.len();
        let size_diff = ofdrw_size.abs_diff(easyofd_size);

        let ofdrw_objs = count_pdf_objects(&ofdrw_bytes);
        let easyofd_objs = count_pdf_objects(&easyofd_bytes);

        // Report size difference
        if ofdrw_size != easyofd_size {
            PDF_DIFFS.fetch_add(1, Ordering::Relaxed);
            eprintln!(
                "DIFF [{name}] PDF byte_size: ofdrw={ofdrw_size} easyofd={easyofd_size} diff={size_diff}"
            );
        }

        // Report object count difference
        if ofdrw_objs != easyofd_objs {
            PDF_DIFFS.fetch_add(1, Ordering::Relaxed);
            eprintln!("DIFF [{name}] PDF object_count: ofdrw={ofdrw_objs} easyofd={easyofd_objs}");
        }

        // Report missing PDFs
        if !ofdrw_exists {
            eprintln!("DIFF [{name}] PDF only in easyofd (size={easyofd_size})");
        }
        if !easyofd_exists {
            eprintln!("DIFF [{name}] PDF only in ofdrw (size={ofdrw_size})");
        }
    }

    eprintln!(
        "\n=== PDF comparison: {} samples, {} PDF-level diffs ===",
        samples.len(),
        PDF_DIFFS.load(Ordering::Relaxed)
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// Test 4: OFD ZIP entry-list comparison
// ═══════════════════════════════════════════════════════════════════════════════

/// Compare the list of ZIP entries between ofdrw and easyofd OFD files.
/// Reports entries that are present on only one side.
#[test]
fn test_ofd_zip_entry_comparison() {
    if !artifacts_available() {
        eprintln!("SKIP: artifact directories not found");
        return;
    }

    let samples = discover_samples();
    assert!(!samples.is_empty(), "no samples discovered");

    for name in &samples {
        let ofdrw_ofd = ofdrw_path(name, "ofd");
        let easyofd_ofd = easyofd_path(name, "ofd");

        if !Path::new(&ofdrw_ofd).exists() {
            eprintln!("SKIP [{name}]: ofdrw OFD missing");
            continue;
        }
        if !Path::new(&easyofd_ofd).exists() {
            eprintln!("SKIP [{name}]: easyofd OFD missing");
            continue;
        }

        let ofdrw_entries = list_ofd_entries(&ofdrw_ofd);
        let easyofd_entries = list_ofd_entries(&easyofd_ofd);

        let ofdrw_set: std::collections::HashSet<&str> =
            ofdrw_entries.iter().map(String::as_str).collect();
        let easyofd_set: std::collections::HashSet<&str> =
            easyofd_entries.iter().map(String::as_str).collect();

        for entry in &ofdrw_set {
            if !easyofd_set.contains(entry) {
                eprintln!("DIFF [{name}] ZIP entry only in ofdrw: {entry}");
            }
        }
        for entry in &easyofd_set {
            if !ofdrw_set.contains(entry) {
                eprintln!("DIFF [{name}] ZIP entry only in easyofd: {entry}");
            }
        }
    }
}
