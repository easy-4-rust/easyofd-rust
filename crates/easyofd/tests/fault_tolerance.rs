#![allow(clippy::needless_borrows_for_generic_args, clippy::single_match)]

//! Phase 3.3 容错测试：损坏 ZIP、截断、CRC 不一致、压缩炸弹、路径穿越等。
//!
//! 覆盖所有要求的异常输入场景，确保 `OfdReader` 和 `validate_archive`
//! 在面对恶意或损坏数据时返回错误而非 panic。

use std::io::{Cursor, Write};

use easyofd_core::{OfdError, OfdPage, TextObject};
use easyofd_package::{PackageLimits, validate_archive, validate_entry_name};
use easyofd_reader::OfdReader;
use easyofd_writer::OfdWriter;
use zip::write::{SimpleFileOptions, ZipWriter};

// ─── Helpers ──────────────────────────────────────────────────────────────────

/// 构建一个包含单页文本的合法 OFD 字节。
fn build_valid_ofd() -> Vec<u8> {
    let mut writer = OfdWriter::new();
    let mut page = OfdPage::new(210.0, 297.0);
    page.add_text(TextObject::new(10.0, 20.0, "容错测试"));
    writer.add_page(page);
    writer.build().expect("OfdWriter::build should succeed")
}

/// 创建一个包含指定条目的最小 ZIP（Stored 压缩）。
fn build_zip(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
    for (name, data) in entries {
        zip.start_file(*name, options).unwrap();
        zip.write_all(data).unwrap();
    }
    zip.finish().unwrap().into_inner()
}

/// 在 ZIP 原始字节中修补所有 local file header 和 central directory 的同一个
/// u32 字段。
///
/// - `local_off`: 字段相对 `PK\x03\x04` 签名的字节偏移
/// - `cd_off`:    字段相对 `PK\x01\x02` 签名的字节偏移
fn patch_zip_u32(bytes: &mut [u8], local_off: usize, cd_off: usize, value: u32) {
    let le = value.to_le_bytes();
    // Local file header(s)
    let mut i = 0;
    while i + local_off + 4 <= bytes.len() {
        if bytes[i..i + 4] == [b'P', b'K', 0x03, 0x04] {
            bytes[i + local_off..i + local_off + 4].copy_from_slice(&le);
        }
        i += 1;
    }
    // Central directory entry(ies)
    i = 0;
    while i + cd_off + 4 <= bytes.len() {
        if bytes[i..i + 4] == [b'P', b'K', 0x01, 0x02] {
            bytes[i + cd_off..i + cd_off + 4].copy_from_slice(&le);
        }
        i += 1;
    }
}

/// 篡改 ZIP 中所有 CRC-32 字段（local +14, central directory +16）。
fn corrupt_zip_crc(bytes: &mut [u8]) {
    patch_zip_u32(bytes, 14, 16, 0xDEAD_BEEF);
}

// ─── 1. 损坏 ZIP 头 ──────────────────────────────────────────────────────────

#[test]
fn corrupt_zip_header_returns_error() {
    let Err(err) = OfdReader::from_bytes(b"NOT_A_ZIP_FILE") else {
        panic!("non-ZIP input must fail");
    };
    assert!(
        matches!(err, OfdError::Zip(_)),
        "expected OfdError::Zip for non-ZIP input"
    );
}

// ─── 2. 截断 ZIP（中间字节缺失）────────────────────────────────────────────

#[test]
fn truncated_zip_returns_error() {
    let valid = build_valid_ofd();
    assert!(valid.len() > 20, "valid OFD must be >20 bytes");
    let truncated = &valid[..valid.len() - 10];
    let result = OfdReader::from_bytes(truncated);
    assert!(result.is_err(), "truncated ZIP must fail");
}

// ─── 3. ZIP CRC 不一致 ───────────────────────────────────────────────────────

#[test]
fn zip_crc_mismatch_returns_error() {
    let ofd_xml = b"<?xml version=\"1.0\"?>\
        <ofd:OFD xmlns:ofd=\"http://www.ofdspec.org/2016\">\
          <ofd:DocBody>\
            <ofd:DocRoot>Doc_0/Document.xml</ofd:DocRoot>\
          </ofd:DocBody>\
        </ofd:OFD>";
    let mut zip_bytes = build_zip(&[("OFD.xml", ofd_xml.as_ref())]);
    corrupt_zip_crc(&mut zip_bytes);
    let result = OfdReader::from_bytes(&zip_bytes);
    assert!(result.is_err(), "CRC mismatch must be rejected");
}

// ─── 4. 超大压缩比（zip bomb）───────────────────────────────────────────────

#[test]
fn compression_bomb_detected() {
    // 构造一个 stored 条目（2 KB），然后篡改 compressed_size 使 ratio > 1000:1。
    let mut zip_bytes = build_zip(&[("bomb.dat", &vec![0u8; 2048])]);
    // compressed_size 在 local header +18、central directory +20
    // 设为 2 → ratio = 2048 / 2 = 1024 > 1000
    patch_zip_u32(&mut zip_bytes, 18, 20, 2);
    let mut archive = zip::ZipArchive::new(Cursor::new(&zip_bytes)).unwrap();
    let result = validate_archive(&mut archive, PackageLimits::default());
    assert!(result.is_err(), "compression bomb must be rejected");
    assert!(
        matches!(result.unwrap_err(), OfdError::InvalidDocument(_)),
        "expected InvalidDocument for compression ratio violation"
    );
}

// ─── 5. 空字节 ───────────────────────────────────────────────────────────────

#[test]
fn empty_bytes_returns_error() {
    let result = OfdReader::from_bytes(&[]);
    assert!(result.is_err(), "empty input must fail");
}

// ─── 6. 仅含 EOCD 记录（无 entries）─────────────────────────────────────────

#[test]
fn eocd_only_returns_error() {
    // End of central directory record: 签名 PK\x05\x06 + 18 字节零填充
    let mut eocd = [0u8; 22];
    eocd[0] = b'P';
    eocd[1] = b'K';
    eocd[2] = 0x05;
    eocd[3] = 0x06;
    let result = OfdReader::from_bytes(&eocd);
    assert!(result.is_err(), "EOCD-only input must fail");
}

// ─── 7. 条目数超限（20001）─────────────────────────────────────────────────

#[test]
fn rejects_20001_entries() {
    let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
    for i in 0..20_001u32 {
        zip.start_file(format!("entry_{i:05}"), options).unwrap();
        zip.write_all(b"x").unwrap();
    }
    let bytes = zip.finish().unwrap().into_inner();
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
    let result = validate_archive(&mut archive, PackageLimits::default());
    assert!(result.is_err(), "20001 entries must be rejected");
    assert!(
        matches!(result.unwrap_err(), OfdError::InvalidDocument(_)),
        "expected InvalidDocument for entry count overflow"
    );
}

// ─── 8. 单条目 >256 MB ─────────────────────────────────────────────────────

#[test]
fn rejects_single_entry_over_256mb() {
    // 创建一个 1 字节 stored 条目，篡改 uncompressed_size 为 300 MB。
    let mut zip_bytes = build_zip(&[("big.dat", b"x")]);
    // uncompressed_size: local +22, central directory +24
    patch_zip_u32(&mut zip_bytes, 22, 24, 300_000_000);
    let mut archive = zip::ZipArchive::new(Cursor::new(&zip_bytes)).unwrap();
    let result = validate_archive(&mut archive, PackageLimits::default());
    assert!(result.is_err(), "300 MB entry must be rejected");
    assert!(
        matches!(result.unwrap_err(), OfdError::InvalidDocument(_)),
        "expected InvalidDocument for oversized entry"
    );
}

// ─── 9. 畸形 Document.xml（未闭合标签）─────────────────────────────────────

#[test]
fn malformed_document_xml_no_panic() {
    let ofd_xml = b"<?xml version=\"1.0\"?>\
        <ofd:OFD xmlns:ofd=\"http://www.ofdspec.org/2016\">\
          <ofd:DocBody>\
            <ofd:DocRoot>Doc_0/Document.xml</ofd:DocRoot>\
          </ofd:DocBody>\
        </ofd:OFD>";
    let zip_bytes = build_zip(&[
        ("OFD.xml", ofd_xml.as_ref()),
        ("Doc_0/Document.xml", b"<not_closed>"),
    ]);
    // 不 panic 即可；quick-xml 不校验 start/end 配对，所以返回 Ok(0 pages) 是预期行为。
    match OfdReader::from_bytes(&zip_bytes) {
        Ok(reader) => assert_eq!(
            reader.page_count(),
            0,
            "malformed Document.xml should yield 0 pages"
        ),
        Err(_) => {} // XML 错误也可以接受——关键是不 panic
    }
}

// ─── 10. 畸形 Content XML（未转义 <）────────────────────────────────────────

#[test]
fn malformed_content_xml_no_panic() {
    let ofd_xml = b"<?xml version=\"1.0\"?>\
        <ofd:OFD xmlns:ofd=\"http://www.ofdspec.org/2016\">\
          <ofd:DocBody>\
            <ofd:DocRoot>Doc_0/Document.xml</ofd:DocRoot>\
          </ofd:DocBody>\
        </ofd:OFD>";
    let document_xml = b"<?xml version=\"1.0\"?>\
        <ofd:Document xmlns:ofd=\"http://www.ofdspec.org/2016\">\
          <ofd:Pages>\
            <ofd:Page BaseLoc=\"Pages/Page_0.xml\"/>\
          </ofd:Pages>\
        </ofd:Document>";
    let page_xml = br#"<?xml version="1.0"?>
<ofd:Page xmlns:ofd="http://www.ofdspec.org/2016">
  <ofd:Content>
    <ofd:TextObject Boundary="0 0 100 20">
      <ofd:TextCode X="0" Y="15">bad < content</ofd:TextCode>
    </ofd:TextObject>
  </ofd:Content>
</ofd:Page>"#;
    let zip_bytes = build_zip(&[
        ("OFD.xml", ofd_xml.as_ref()),
        ("Doc_0/Document.xml", document_xml.as_ref()),
        ("Doc_0/Pages/Page_0.xml", page_xml.as_ref()),
    ]);
    // 不 panic 即可；返回 Err(OfdError::Xml) 是预期行为（未转义 < 违反 XML 规范）。
    let _ = OfdReader::from_bytes(&zip_bytes);
}

// ─── 11. 路径穿越 — validate_entry_name ──────────────────────────────────────

#[test]
fn path_traversal_rejected_by_entry_name() {
    // 必须拒绝的路径
    assert!(validate_entry_name("../escape.txt").is_err());
    assert!(validate_entry_name("Doc_0/../../etc/passwd").is_err());
    assert!(validate_entry_name("/absolute/path").is_err());
    assert!(validate_entry_name("\\backslash\\path").is_err());

    // 合法路径
    assert!(validate_entry_name("Doc_0/Pages/Page_0.xml").is_ok());
    assert!(validate_entry_name("OFD.xml").is_ok());
}

// ─── 12. 路径穿越 — validate_archive ─────────────────────────────────────────

#[test]
fn path_traversal_in_zip_rejected() {
    let zip_bytes = build_zip(&[("../escape.txt", b"escaped")]);
    let mut archive = zip::ZipArchive::new(Cursor::new(&zip_bytes)).unwrap();
    let result = validate_archive(&mut archive, PackageLimits::default());
    assert!(result.is_err(), "path traversal must be rejected");
    assert!(
        matches!(result.unwrap_err(), OfdError::InvalidDocument(_)),
        "expected InvalidDocument for path traversal"
    );
}
