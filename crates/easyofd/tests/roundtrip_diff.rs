// Copyright 2024 easy-4-rust contributors
// SPDX-License-Identifier: Apache-2.0

//! Roundtrip diff: read ofdrw-produced fixtures -> write -> compare against original.
//!
//! This test quantifies how many structural deviations remain between
//! ofdrw output and easyofd-rust roundtrip output.

use std::collections::HashMap;
use std::io::Read;
use std::path::{Path, PathBuf};

use easyofd_reader::OfdReader;
use easyofd_writer::OfdWriter;
use quick_xml::events::Event;
use quick_xml::reader::Reader;

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

fn fixture_path(name: &str) -> PathBuf {
    fixture_dir().join(name)
}

fn read_zip_entry_as_string(zip_bytes: &[u8], entry_name: &str) -> Option<String> {
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(zip_bytes)).ok()?;
    let mut file = archive.by_name(entry_name).ok()?;
    let mut buf = String::new();
    file.read_to_string(&mut buf).ok()?;
    Some(buf)
}

fn list_zip_entries(zip_bytes: &[u8]) -> Vec<String> {
    let Ok(archive) = zip::ZipArchive::new(std::io::Cursor::new(zip_bytes)) else {
        return vec![];
    };
    archive
        .file_names()
        .map(std::string::ToString::to_string)
        .collect()
}

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

fn roundtrip(bytes: &[u8]) -> Vec<u8> {
    let reader = OfdReader::from_bytes(bytes).expect("initial read should succeed");
    let mut writer = OfdWriter::new();
    writer.set_metadata(reader.metadata().clone());
    // Carry over entries the writer does not regenerate (template pages,
    // annotations, attachments, signatures, custom tags and payload files).
    writer.preserve_entries(reader.raw_entries().to_vec());
    for page in reader.pages() {
        writer.add_page(page.clone());
    }
    writer.build().expect("roundtrip write should succeed")
}

#[allow(dead_code)]
struct DiffReport {
    fixture: String,
    zip_diffs: Vec<String>,
    xml_diffs: Vec<String>,
}

fn analyze_fixture(name: &str) -> DiffReport {
    let path = fixture_path(name);
    let orig_bytes = std::fs::read(&path).unwrap();
    let rt_bytes = roundtrip(&orig_bytes);

    let orig_entries = list_zip_entries(&orig_bytes);
    let rt_entries = list_zip_entries(&rt_bytes);

    let mut zip_diffs = Vec::new();
    let mut xml_diffs = Vec::new();

    // Normalize page paths for comparison: Page_N/Content.xml -> Page_N.xml
    let normalize = |e: &String| -> String {
        if e.starts_with("Doc_0/Pages/Page_") && e.ends_with("/Content.xml") {
            e.replace("/Content.xml", ".xml")
        } else {
            e.clone()
        }
    };

    let orig_norm: Vec<String> = orig_entries.iter().map(normalize).collect();
    let rt_norm: Vec<String> = rt_entries.iter().map(normalize).collect();

    let orig_set: std::collections::HashSet<&str> = orig_norm.iter().map(String::as_str).collect();
    let rt_set: std::collections::HashSet<&str> = rt_norm.iter().map(String::as_str).collect();

    for entry in &orig_set {
        if !rt_set.contains(entry) {
            zip_diffs.push(format!("only in ofdrw: {entry}"));
        }
    }
    for entry in &rt_set {
        if !orig_set.contains(entry) {
            zip_diffs.push(format!("only in easyofd: {entry}"));
        }
    }

    // Compare XML element distributions for shared files
    for (xml_name, orig_xml) in &[
        ("OFD.xml", read_zip_entry_as_string(&orig_bytes, "OFD.xml")),
        (
            "Doc_0/Document.xml",
            read_zip_entry_as_string(&orig_bytes, "Doc_0/Document.xml"),
        ),
    ] {
        let (Some(orig), Some(rt_xml)) = (
            orig_xml.as_ref(),
            read_zip_entry_as_string(&rt_bytes, xml_name),
        ) else {
            continue;
        };
        let orig_counts = count_xml_elements(orig);
        let rt_counts = count_xml_elements(&rt_xml);

        for (elem, count) in &orig_counts {
            let rt_count = rt_counts.get(elem).copied().unwrap_or(0);
            if *count != rt_count {
                xml_diffs.push(format!(
                    "{xml_name} element {elem}: ofdrw={count} easyofd={rt_count}"
                ));
            }
        }
        for (elem, count) in &rt_counts {
            if !orig_counts.contains_key(elem) {
                xml_diffs.push(format!(
                    "{xml_name} element {elem} only in easyofd (count={count})"
                ));
            }
        }
    }

    DiffReport {
        fixture: name.to_string(),
        zip_diffs,
        xml_diffs,
    }
}

const FIXTURES: &[&str] = &[
    "simple_1.ofd",
    "simple_2.ofd",
    "multi_page_image.ofd",
    "signed.ofd",
    "with_table.ofd",
    // Previously unreadable samples: leading-slash DocRoot, non-standard
    // Document file name, case-mismatched resource directories.
    "ofdrw_containsJPEG.ofd",
    "ofdrw_n.ofd",
    "ofdrw_path_unstd.ofd",
    "ofdrw_testImageNotFound.ofd",
    "ofdrw_testImageOverridePage.ofd",
    "ofdrw_testPathClip.ofd",
];

#[test]
fn roundtrip_diff_report() {
    let mut total_zip = 0;
    let mut total_xml = 0;

    for name in FIXTURES {
        let path = fixture_path(name);
        if !path.exists() {
            println!("[SKIP] {name}: not found");
            continue;
        }
        let report = analyze_fixture(name);
        let total = report.zip_diffs.len() + report.xml_diffs.len();
        if total == 0 {
            println!("[OK] {name}: no deviations");
            continue;
        }
        println!(
            "\n[DIFF] {name}: {} ZIP + {} XML = {} total",
            report.zip_diffs.len(),
            report.xml_diffs.len(),
            total
        );
        for d in &report.zip_diffs {
            println!("  ZIP: {d}");
        }
        for d in &report.xml_diffs {
            println!("  XML: {d}");
        }
        total_zip += report.zip_diffs.len();
        total_xml += report.xml_diffs.len();
    }

    println!("\n================================================================");
    println!(
        "Total: {total_zip} ZIP diffs + {total_xml} XML diffs = {}",
        total_zip + total_xml
    );
}
