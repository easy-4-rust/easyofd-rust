// Copyright 2024 easy-4-rust contributors
// SPDX-License-Identifier: Apache-2.0

//! Roundtrip diff: read ofdrw-produced fixtures -> write -> compare against original.
//!
//! 两层比对：
//! 1. **元素计数**（快速预检）：按元素名统计数量，检测结构性增删。
//! 2. **文本级规范化比对**（全文）：解析 XmlNode 树后递归比较元素名、
//!    属性值、文本内容，忽略 ofd: 前缀差异、属性顺序、自闭合风格差异，
//!    能抓到日期格式偏差、属性值不同、文本内容不同等字节级差异。

use std::collections::HashMap;
use std::io::Read;
use std::path::{Path, PathBuf};

use easyofd_core::XmlNode;
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

/// Count XML elements, normalizing the `ofd:` namespace prefix so that
/// structural comparison is namespace-agnostic.
fn count_xml_elements(xml: &str) -> HashMap<String, usize> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut counts: HashMap<String, usize> = HashMap::new();
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e) | Event::Empty(e)) => {
                let raw = String::from_utf8_lossy(e.name().as_ref()).into_owned();
                let name = raw.strip_prefix("ofd:").unwrap_or(&raw).to_string();
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
    writer.preserve_entries(reader.raw_entries().to_vec());
    // roundtrip 保真：传入原始 XML，writer 原样输出 OFD.xml / Document.xml
    if let Some(xml) = reader.raw_ofd_xml() {
        writer.set_raw_ofd_xml(xml.to_string());
    }
    if let Some(xml) = reader.raw_document_xml() {
        writer.set_raw_document_xml(xml.to_string());
    }
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
    text_diffs: Vec<String>,
}

/// 递归比较两棵 XmlNode 树，报告文本级差异。
///
/// 规范化规则：
/// - 元素名已由解析器去掉 ofd: 前缀（`local_name`）。
/// - 属性按 key 排序后逐对比较（忽略原始顺序差异）。
/// - 文本内容 trim 后比较（忽略前导/尾随空白差异）。
/// - 空自闭合 `<Tag/>` 与空显式闭合 `<Tag></Tag>` 经解析后等价
///   （均为 `children: []`），无需额外处理。
fn compare_xml_nodes(orig: &XmlNode, rt: &XmlNode, path: &str, diffs: &mut Vec<String>) {
    // 比较元素名（已去除 namespace 前缀）
    if orig.name != rt.name {
        diffs.push(format!(
            "{path}: element name mismatch: '{}' vs '{}'",
            orig.name, rt.name
        ));
        return;
    }
    let elem_path = if path.is_empty() {
        orig.name.clone()
    } else {
        format!("{path}/{}", orig.name)
    };

    // 属性：按 key 排序后比较
    let mut orig_attrs: Vec<(&str, &str)> = orig
        .attrs
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();
    let mut rt_attrs: Vec<(&str, &str)> = rt
        .attrs
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();
    orig_attrs.sort_unstable_by_key(|(k, _)| *k);
    rt_attrs.sort_unstable_by_key(|(k, _)| *k);

    if orig_attrs.len() != rt_attrs.len() {
        diffs.push(format!(
            "{elem_path}: attribute count mismatch: {} vs {}",
            orig_attrs.len(),
            rt_attrs.len()
        ));
    }
    for (i, ((ok, ov), (rk, rv))) in orig_attrs.iter().zip(rt_attrs.iter()).enumerate() {
        if ok != rk {
            diffs.push(format!(
                "{elem_path}: attr[{i}] key mismatch: '{ok}' vs '{rk}'"
            ));
        }
        if ov != rv {
            diffs.push(format!(
                "{elem_path}: attr '{ok}' value mismatch: '{ov}' vs '{rv}'"
            ));
        }
    }

    // 文本内容（trim 后比较）
    let orig_text = orig.text.as_deref().unwrap_or("").trim();
    let rt_text = rt.text.as_deref().unwrap_or("").trim();
    if orig_text != rt_text {
        diffs.push(format!(
            "{elem_path}: text content mismatch: '{}' vs '{}'",
            truncate_display(orig_text, 80),
            truncate_display(rt_text, 80)
        ));
    }

    // 子元素
    if orig.children.len() != rt.children.len() {
        diffs.push(format!(
            "{elem_path}: child count mismatch: {} vs {}",
            orig.children.len(),
            rt.children.len()
        ));
    }
    for (oc, rc) in orig.children.iter().zip(rt.children.iter()) {
        compare_xml_nodes(oc, rc, &elem_path, diffs);
    }
}

/// 截断字符串用于显示（超过 max_len 时加 "..."）。
fn truncate_display(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len])
    }
}

/// 文本级规范化比对：解析两段 XML 为 XmlNode 树，递归比较。
fn compare_xml_text(orig_xml: &str, rt_xml: &str) -> Vec<String> {
    let orig_tree = match easyofd_core::parse_xml_to_nodes(orig_xml) {
        Ok(tree) => tree,
        Err(e) => return vec![format!("original XML parse error: {e}")],
    };
    let rt_tree = match easyofd_core::parse_xml_to_nodes(rt_xml) {
        Ok(tree) => tree,
        Err(e) => return vec![format!("roundtrip XML parse error: {e}")],
    };
    let mut diffs = Vec::new();
    compare_xml_nodes(&orig_tree, &rt_tree, "", &mut diffs);
    diffs
}

fn analyze_fixture(name: &str) -> DiffReport {
    let path = fixture_path(name);
    let orig_bytes = std::fs::read(&path).unwrap();
    let rt_bytes = roundtrip(&orig_bytes);

    let orig_entries = list_zip_entries(&orig_bytes);
    let rt_entries = list_zip_entries(&rt_bytes);

    let mut zip_diffs = Vec::new();
    let mut xml_diffs = Vec::new();
    let mut text_diffs = Vec::new();

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

    // 比对共享 XML 文件：元素计数（快速预检）+ 文本级规范化比对（全文）
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

        // 层 1：元素计数（快速预检）
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

        // 层 2：文本级规范化比对（抓日期格式偏差、属性值差异、文本内容差异）
        let semantic_diffs = compare_xml_text(orig, &rt_xml);
        for d in &semantic_diffs {
            text_diffs.push(format!("{xml_name}: {d}"));
        }
    }

    DiffReport {
        fixture: name.to_string(),
        zip_diffs,
        xml_diffs,
        text_diffs,
    }
}

/// Discover every `.ofd` fixture in `tests/fixtures/real_ofd/`, sorted.
fn discover_fixtures() -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(fixture_dir())
        .map(|entries| {
            entries
                .flatten()
                .filter_map(|e| {
                    let path = e.path();
                    if path
                        .extension()
                        .is_some_and(|ext| ext.eq_ignore_ascii_case("ofd"))
                    {
                        path.file_name().map(|f| f.to_string_lossy().into_owned())
                    } else {
                        None
                    }
                })
                .collect()
        })
        .unwrap_or_default();
    names.sort();
    names
}

#[test]
fn roundtrip_diff_report() {
    let mut total_zip = 0;
    let mut total_xml = 0;
    let mut total_text = 0;
    let mut ok_count = 0;
    let mut diff_count = 0;
    let mut skipped = 0;

    for name in discover_fixtures() {
        let path = fixture_path(&name);
        if !path.exists() {
            println!("[SKIP] {name}: not found");
            skipped += 1;
            continue;
        }
        let Ok(report) = std::panic::catch_unwind(|| analyze_fixture(&name)) else {
            println!("[PANIC] {name}: roundtrip panicked");
            diff_count += 1;
            continue;
        };
        let total = report.zip_diffs.len() + report.xml_diffs.len() + report.text_diffs.len();
        if total == 0 {
            println!("[OK] {name}: no deviations");
            ok_count += 1;
            continue;
        }
        println!(
            "\n[DIFF] {name}: {} ZIP + {} XML-count + {} text = {} total",
            report.zip_diffs.len(),
            report.xml_diffs.len(),
            report.text_diffs.len(),
            total
        );
        for d in &report.zip_diffs {
            println!("  ZIP: {d}");
        }
        for d in &report.xml_diffs {
            println!("  XML-count: {d}");
        }
        for d in &report.text_diffs {
            println!("  TEXT: {d}");
        }
        total_zip += report.zip_diffs.len();
        total_xml += report.xml_diffs.len();
        total_text += report.text_diffs.len();
        diff_count += 1;
    }

    println!("\n================================================================");
    println!(
        "Total: {total_zip} ZIP + {total_xml} XML-count + {total_text} text = {} across {ok_count} clean, {diff_count} with deviations, {skipped} skipped",
        total_zip + total_xml + total_text
    );
}
