// Copyright 2024 easy-4-rust contributors
// SPDX-License-Identifier: Apache-2.0

//! 元数据保留验证测试：read -> set_metadata -> write -> re-read -> 逐字段断言。
//!
//! roundtrip_diff.rs 只断言"字节级 0 ZIP + 0 XML 偏差"，本测试文件单独验证
//! Creator / CreatorVersion / ModDate / Author / Title / Keywords / Subject / DocID
//! 等 CT_DocInfo 字段在 roundtrip 中被精确保留。
//!
//! ## DocID 行为文档化
//!
//! Java ofdrw 的 `BareOFDDoc` 会生成新 DocID，但 Rust 实现中
//! `OfdMetadata.doc_id` 由 reader 解析后通过 `set_metadata` 原样传递给 writer，
//! 因此 roundtrip 会保留原始 DocID。本测试对此行为做显式断言。

use std::path::{Path, PathBuf};

use easyofd_core::OfdMetadata;
use easyofd_reader::OfdReader;
use easyofd_writer::OfdWriter;

/// 测试用例所在 crate 的 manifest 目录。
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

/// 读取 fixture -> set_metadata -> write -> re-read -> 返回 roundtrip 后的元数据。
///
/// 对应 ofdrw 的 read-write-read 链路，验证元数据在 writer 输出中被正确保留。
fn roundtrip_metadata(fixture_name: &str) -> OfdMetadata {
    let path = fixture_path(fixture_name);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("读取 fixture 失败: {e}"));

    // 第一次读取：提取原始元数据
    let reader = OfdReader::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("首次读取 {fixture_name} 失败: {e}"));
    let original_metadata = reader.metadata().clone();

    // 写入：set_metadata + preserve_entries + add_page
    let mut writer = OfdWriter::new();
    writer.set_metadata(original_metadata);
    writer.preserve_entries(reader.raw_entries().to_vec());
    for page in reader.pages() {
        writer.add_page(page.clone());
    }
    let output = writer
        .build()
        .unwrap_or_else(|e| panic!("写入 {fixture_name} 失败: {e}"));

    // 第二次读取：从 roundtrip 产物中提取元数据
    let reader2 = OfdReader::from_bytes(&output)
        .unwrap_or_else(|e| panic!("二次读取 {fixture_name} 失败: {e}"));
    reader2.metadata().clone()
}

// ─── 测试 1: 全字段保留（ofdrw_n.ofd：含 Creator/Author/ModDate/Keywords/DocUsage） ───

/// ofdrw_n.ofd 包含最丰富的元数据：Creator、Author、ModDate、DocID、
/// Keywords（空标签）、DocUsage、CreatorVersion、CreationDate。
///
/// 本测试断言 roundtrip 后所有字段值与原始值一致。
#[test]
fn metadata_roundtrip_ofdrw_n_all_fields() {
    let rt = roundtrip_metadata("ofdrw_n.ofd");

    // 对应 ofdrw CT_DocInfo 字段：Creator
    assert_eq!(
        rt.creator.as_deref(),
        Some("iOFD\u{00ae} 2.1.1 \u{00a9}2019-2020 dms360.cn"),
        "Creator 应被保留"
    );
    // 对应 ofdrw CT_DocInfo 字段：CreatorVersion
    assert_eq!(
        rt.creator_version.as_deref(),
        Some("2.1.1"),
        "CreatorVersion 应被保留"
    );
    // 对应 ofdrw CT_DocInfo 字段：Author
    assert_eq!(
        rt.author.as_deref(),
        Some("iOFD\u{00ae} 2.1.1 \u{00a9}2019-2020 dms360.cn"),
        "Author 应被保留"
    );
    // 对应 ofdrw CT_DocInfo 字段：ModDate（原始为 "2020-01-25"，解析为 NaiveDateTime）
    assert!(rt.mod_date.is_some(), "ModDate 应被保留");
    let mod_date = rt.mod_date.unwrap();
    assert_eq!(mod_date.format("%Y-%m-%d").to_string(), "2020-01-25");
    // 对应 ofdrw CT_DocInfo 字段：CreationDate
    assert!(rt.creation_date.is_some(), "CreationDate 应被保留");
    let creation_date = rt.creation_date.unwrap();
    assert_eq!(creation_date.format("%Y-%m-%d").to_string(), "2020-01-25");
    // 对应 ofdrw CT_DocInfo 字段：DocUsage
    assert_eq!(rt.doc_usage.as_deref(), Some("Normal"), "DocUsage 应被保留");
    // 对应 ofdrw CT_DocInfo 字段：Keywords（原始为空标签 <Keywords/>，
    // find_optional_text_deep 返回 Some("")，roundtrip 应保留）
    assert_eq!(
        rt.keywords.as_deref(),
        Some(""),
        "Keywords（空标签）应被保留为 Some(\"\")"
    );
}

// ─── 测试 2: 含 Title 的文档保留（ofdrw_z.ofd） ───

/// ofdrw_z.ofd 含 Title（"11.4 字型变换"）、中文 Author、Creator、ModDate。
///
/// 验证 Title 和中文文本在 roundtrip 中不丢失。
#[test]
fn metadata_roundtrip_ofdrw_z_with_title() {
    let rt = roundtrip_metadata("ofdrw_z.ofd");

    // 对应 ofdrw CT_DocInfo 字段：Title
    assert_eq!(rt.title.as_deref(), Some("11.4 字型变换"), "Title 应被保留");
    // 对应 ofdrw CT_DocInfo 字段：Author（中文）
    assert_eq!(
        rt.author.as_deref(),
        Some("\u{6731}\u{6587}\u{8363}"),
        "Author 应被保留"
    );
    // 对应 ofdrw CT_DocInfo 字段：Creator
    assert_eq!(
        rt.creator.as_deref(),
        Some("suwell ofd maker"),
        "Creator 应被保留"
    );
    // 对应 ofdrw CT_DocInfo 字段：CreatorVersion
    assert_eq!(
        rt.creator_version.as_deref(),
        Some("0.8"),
        "CreatorVersion 应被保留"
    );
    // 对应 ofdrw CT_DocInfo 字段：ModDate
    assert!(rt.mod_date.is_some(), "ModDate 应被保留");
    assert_eq!(
        rt.mod_date.unwrap().format("%Y-%m-%d").to_string(),
        "2012-06-05"
    );
    // 对应 ofdrw CT_DocInfo 字段：CreationDate
    assert!(rt.creation_date.is_some(), "CreationDate 应被保留");
    assert_eq!(
        rt.creation_date.unwrap().format("%Y-%m-%d").to_string(),
        "2012-06-05"
    );
}

// ─── 测试 3: 中文标题 + 中文 Author（ofdrw_发票监制章-数科.ofd） ───

/// ofdrw_发票监制章-数科.ofd 含 Title="未命名"、Author="Administrator"。
///
/// 验证中文标题和 ASCII Author 均被保留。
#[test]
fn metadata_roundtrip_chinese_title_and_author() {
    let rt = roundtrip_metadata("ofdrw_发票监制章-数科.ofd");

    assert_eq!(
        rt.title.as_deref(),
        Some("\u{672a}\u{547d}\u{540d}"),
        "中文 Title 应被保留"
    );
    assert_eq!(
        rt.author.as_deref(),
        Some("Administrator"),
        "Author 应被保留"
    );
    assert_eq!(
        rt.creator.as_deref(),
        Some("Suwell-pdf2ofd"),
        "Creator 应被保留"
    );
    assert_eq!(
        rt.creator_version.as_deref(),
        Some("1.0.19.0910"),
        "CreatorVersion 应被保留"
    );
    assert!(rt.mod_date.is_some(), "ModDate 应被保留");
    assert_eq!(
        rt.mod_date.unwrap().format("%Y-%m-%d").to_string(),
        "2020-10-29"
    );
}

// ─── 测试 4: 缺字段不被填充为空值（ofdrw_keyword.ofd：无 Creator/ModDate/Title） ───

/// ofdrw_keyword.ofd 只有 DocID、Author、CreationDate、CreatorVersion，
/// 无 Creator、ModDate、Title、Keywords。
///
/// 验证原始为 None 的字段在 roundtrip 后仍为 None，不会被错误填充。
#[test]
fn metadata_roundtrip_missing_fields_stay_none() {
    let rt = roundtrip_metadata("ofdrw_keyword.ofd");

    // 应存在的字段
    assert_eq!(rt.author.as_deref(), Some("admin"), "Author 应被保留");
    assert_eq!(
        rt.creator_version.as_deref(),
        Some("V1.0"),
        "CreatorVersion 应被保留"
    );
    assert!(rt.creation_date.is_some(), "CreationDate 应被保留");

    // 应不存在的字段：roundtrip 不应凭空产生
    assert!(
        rt.creator.is_none(),
        "原始无 Creator，roundtrip 后应仍为 None"
    );
    assert!(
        rt.mod_date.is_none(),
        "原始无 ModDate，roundtrip 后应仍为 None"
    );
    assert!(rt.title.is_none(), "原始无 Title，roundtrip 后应仍为 None");
    assert!(
        rt.keywords.is_none(),
        "原始无 Keywords，roundtrip 后应仍为 None"
    );
    assert!(
        rt.doc_usage.is_none(),
        "原始无 DocUsage，roundtrip 后应仍为 None"
    );
}

// ─── 测试 5: DocID 行为文档化 ───

/// DocID 行为断言。
///
/// Java ofdrw 的 `BareOFDDoc` 在合并/生成时会产生新 DocID（UUID），
/// 但 Rust 实现中 reader 将 DocID 解析为 `OfdMetadata.doc_id: Option<String>`，
/// writer 原样输出，因此 roundtrip 保留原始 DocID。
///
/// 本测试断言：roundtrip 后 DocID 与原始值一致（非空、合法 UUID 格式）。
#[test]
fn metadata_roundtrip_doc_id_preserved() {
    let path = fixture_path("ofdrw_n.ofd");
    let bytes = std::fs::read(&path).unwrap();
    let reader = OfdReader::from_bytes(&bytes).unwrap();
    let original_doc_id = reader.metadata().doc_id.clone();

    // 原始 DocID 应存在
    assert!(original_doc_id.is_some(), "ofdrw_n.ofd 应含 DocID");
    let original_doc_id = original_doc_id.unwrap();
    assert_eq!(
        original_doc_id.len(),
        32,
        "DocID 应为 32 字符 UUID（无连字符）"
    );
    assert!(
        original_doc_id.chars().all(|c| c.is_ascii_hexdigit()),
        "DocID 应仅含十六进制字符"
    );

    // Roundtrip 后 DocID 应被保留
    let rt = roundtrip_metadata("ofdrw_n.ofd");
    assert_eq!(
        rt.doc_id.as_deref(),
        Some(original_doc_id.as_str()),
        "DocID 在 roundtrip 中应被原样保留"
    );
}

// ─── 测试 6: DocID 在多 fixture 中均被保留 ───

/// 对多个含 DocID 的 fixture 做 roundtrip，断言 DocID 全部保留。
///
/// 覆盖不同来源的 OFD 文件：ofdrw 生成、第三方工具生成。
#[test]
fn metadata_roundtrip_doc_id_multi_fixture() {
    let fixtures_with_doc_id = [
        "ofdrw_z.ofd",
        "ofdrw_发票监制章-数科.ofd",
        "ofdrw_keyword.ofd",
        "ofdrw_testPathFillOpacity.ofd",
    ];

    for name in &fixtures_with_doc_id {
        let path = fixture_path(name);
        let bytes = std::fs::read(&path).unwrap();
        let reader =
            OfdReader::from_bytes(&bytes).unwrap_or_else(|e| panic!("读取 {name} 失败: {e}"));
        let original_doc_id = reader.metadata().doc_id.clone();

        let rt = roundtrip_metadata(name);
        assert_eq!(
            rt.doc_id, original_doc_id,
            "{name}: DocID 在 roundtrip 中应被保留"
        );
    }
}

// ─── 测试 7: Subject 字段 roundtrip（CT_DocInfo 补全） ───

/// Subject 是 CT_DocInfo 标准字段，当前 fixtures 均不含 Subject。
///
/// 本测试通过合成元数据验证 Subject 字段在 writer -> reader 链路中被正确保留。
#[test]
fn metadata_roundtrip_subject_synthetic() {
    let path = fixture_path("ofdrw_keyword.ofd");
    let bytes = std::fs::read(&path).unwrap();
    let reader = OfdReader::from_bytes(&bytes).unwrap();

    // 合成含 Subject 的元数据
    let mut meta = reader.metadata().clone();
    meta.subject = Some("OFD 文档主题测试".to_string());

    let mut writer = OfdWriter::new();
    writer.set_metadata(meta);
    writer.preserve_entries(reader.raw_entries().to_vec());
    for page in reader.pages() {
        writer.add_page(page.clone());
    }
    let output = writer.build().unwrap();

    let reader2 = OfdReader::from_bytes(&output).unwrap();
    assert_eq!(
        reader2.metadata().subject.as_deref(),
        Some("OFD 文档主题测试"),
        "Subject 字段应在 roundtrip 中被保留"
    );
}

// ─── 测试 8: 空 Keywords 标签不被丢弃 ───

/// ofdrw_n.ofd 含 `<ofd:Keywords/>`（空自闭合标签）。
///
/// `find_optional_text_deep` 对空标签返回 `Some("")`，
/// writer 输出时应保留该字段（输出为 `<ofd:Keywords></ofd:Keywords>`）。
///
/// 本测试断言 roundtrip 后 Keywords 仍为 `Some("")` 而非 `None`。
#[test]
fn metadata_roundtrip_empty_keywords_not_lost() {
    let path = fixture_path("ofdrw_n.ofd");
    let bytes = std::fs::read(&path).unwrap();
    let reader = OfdReader::from_bytes(&bytes).unwrap();

    // 原始应为 Some("")
    assert_eq!(
        reader.metadata().keywords.as_deref(),
        Some(""),
        "原始 Keywords 应为 Some(\"\")（空标签）"
    );

    let rt = roundtrip_metadata("ofdrw_n.ofd");
    assert_eq!(
        rt.keywords.as_deref(),
        Some(""),
        "roundtrip 后 Keywords 应仍为 Some(\"\")，不被丢弃为 None"
    );
}
