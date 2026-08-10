//! OFD 合规性测试。
//!
//! 验证生成的 OFD 文件符合 GB/T 33190-2016 标准要求。
//! 测试重点：
//! 1. ZIP 结构合规（OFD.xml → DocRoot → Document.xml → Pages）
//! 2. XML 命名空间正确（ofd: 前缀）
//! 3. 必需元素存在（DocBody, DocRoot, Pages, PageArea）
//! 4. 写入-读取往返保真

use std::io::Cursor;

use easyofd_core::{ContentObject, ImageObject, OfdPage, PathObject, TextObject};
use easyofd_reader::OfdReader;
use easyofd_writer::OfdWriter;
use zip::read::ZipArchive;

/// 辅助：构建简单 OFD 并返回字节
fn build_simple_ofd() -> Vec<u8> {
    let mut writer = OfdWriter::new();
    let mut page = OfdPage::new(210.0, 297.0);
    page.add_text(TextObject::new(10.0, 20.0, "合规测试"));
    writer.add_page(page);
    writer.build().expect("build should succeed")
}

/// 辅助：构建多页 OFD
fn build_multi_page_ofd() -> Vec<u8> {
    let mut writer = OfdWriter::new();
    for i in 0..3 {
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_text(TextObject::new(10.0, 20.0, &format!("第{i}页")));
        writer.add_page(page);
    }
    writer.build().expect("build should succeed")
}

// ─── ZIP 结构合规 ────────────────────────────────────────────────────────────

#[test]
fn compliance_zip_contains_ofd_xml() {
    let data = build_simple_ofd();
    let cursor = Cursor::new(&data[..]);
    let mut archive = ZipArchive::new(cursor).expect("should be valid ZIP");
    // OFD.xml 必须存在
    assert!(archive.by_name("OFD.xml").is_ok(), "OFD.xml must exist");
}

#[test]
fn compliance_zip_contains_doc_root() {
    let data = build_simple_ofd();
    let cursor = Cursor::new(&data[..]);
    let mut archive = ZipArchive::new(cursor).expect("should be valid ZIP");
    assert!(
        archive.by_name("Doc_0/Document.xml").is_ok(),
        "Doc_0/Document.xml must exist"
    );
}

#[test]
fn compliance_zip_contains_pages() {
    let data = build_simple_ofd();
    let cursor = Cursor::new(&data[..]);
    let mut archive = ZipArchive::new(cursor).expect("should be valid ZIP");
    assert!(
        archive.by_name("Doc_0/Pages/Page_0.xml").is_ok(),
        "Doc_0/Pages/Page_0.xml must exist"
    );
}

#[test]
fn compliance_zip_contains_public_res() {
    let data = build_simple_ofd();
    let cursor = Cursor::new(&data[..]);
    let mut archive = ZipArchive::new(cursor).expect("should be valid ZIP");
    assert!(
        archive.by_name("Doc_0/PublicRes.xml").is_ok(),
        "Doc_0/PublicRes.xml must exist"
    );
}

// ─── XML 内容合规 ────────────────────────────────────────────────────────────

#[test]
fn compliance_ofd_xml_has_namespace() {
    let data = build_simple_ofd();
    let cursor = Cursor::new(&data[..]);
    let mut archive = ZipArchive::new(cursor).unwrap();
    let mut file = archive.by_name("OFD.xml").unwrap();
    let mut content = String::new();
    std::io::Read::read_to_string(&mut file, &mut content).unwrap();
    assert!(
        content.contains("http://www.ofdspec.org/2016"),
        "OFD.xml must contain OFD namespace"
    );
}

#[test]
fn compliance_ofd_xml_has_doc_body() {
    let data = build_simple_ofd();
    let cursor = Cursor::new(&data[..]);
    let mut archive = ZipArchive::new(cursor).unwrap();
    let mut file = archive.by_name("OFD.xml").unwrap();
    let mut content = String::new();
    std::io::Read::read_to_string(&mut file, &mut content).unwrap();
    assert!(content.contains("ofd:DocBody"), "OFD.xml must contain ofd:DocBody");
    assert!(content.contains("ofd:DocRoot"), "OFD.xml must contain ofd:DocRoot");
}

#[test]
fn compliance_document_xml_has_page_area() {
    let data = build_simple_ofd();
    let cursor = Cursor::new(&data[..]);
    let mut archive = ZipArchive::new(cursor).unwrap();
    let mut file = archive.by_name("Doc_0/Document.xml").unwrap();
    let mut content = String::new();
    std::io::Read::read_to_string(&mut file, &mut content).unwrap();
    assert!(
        content.contains("ofd:PageArea"),
        "Document.xml must contain ofd:PageArea"
    );
    assert!(content.contains("ofd:Pages"), "Document.xml must contain ofd:Pages");
}

#[test]
fn compliance_page_xml_has_content() {
    let data = build_simple_ofd();
    let cursor = Cursor::new(&data[..]);
    let mut archive = ZipArchive::new(cursor).unwrap();
    let mut file = archive.by_name("Doc_0/Pages/Page_0.xml").unwrap();
    let mut content = String::new();
    std::io::Read::read_to_string(&mut file, &mut content).unwrap();
    assert!(
        content.contains("ofd:Content"),
        "Page XML must contain ofd:Content"
    );
    assert!(
        content.contains("ofd:TextObject"),
        "Page XML must contain ofd:TextObject for text content"
    );
}

// ─── 写入-读取往返保真 ──────────────────────────────────────────────────────

#[test]
fn compliance_roundtrip_text_content() {
    let data = build_simple_ofd();
    let reader = OfdReader::from_bytes(&data).expect("should read back");
    assert_eq!(reader.page_count(), 1);
    let text = reader.extract_all_text();
    assert!(text.contains("合规测试"), "roundtrip text should match");
}

#[test]
fn compliance_roundtrip_multi_page() {
    let data = build_multi_page_ofd();
    let reader = OfdReader::from_bytes(&data).expect("should read back");
    assert_eq!(reader.page_count(), 3);
    let text = reader.extract_all_text();
    assert!(text.contains("第0页"), "page 0 text");
    assert!(text.contains("第1页"), "page 1 text");
    assert!(text.contains("第2页"), "page 2 text");
}

#[test]
fn compliance_roundtrip_path_object() {
    let mut writer = OfdWriter::new();
    let mut page = OfdPage::new(210.0, 297.0);
    page.add_path(PathObject::new(50.0, 50.0, "M 0 0 L 100 100"));
    writer.add_page(page);
    let data = writer.build().unwrap();

    let reader = OfdReader::from_bytes(&data).expect("should read back");
    assert_eq!(reader.page_count(), 1);
    // Path 对象应存在
    let page = &reader.pages()[0];
    assert!(
        page.content.iter().any(|c| matches!(c, ContentObject::Path(_))),
        "should contain path object"
    );
}

#[test]
fn compliance_roundtrip_page_dimensions() {
    let mut writer = OfdWriter::new();
    let page = OfdPage::new(148.5, 210.0); // A5
    writer.add_page(page);
    let data = writer.build().unwrap();

    let reader = OfdReader::from_bytes(&data).unwrap();
    let page = &reader.pages()[0];
    assert!((page.width - 148.5).abs() < 0.1, "width should be 148.5");
    assert!((page.height - 210.0).abs() < 0.1, "height should be 210.0");
}

// ─── GB/T 33190 版本声明 ────────────────────────────────────────────────────

#[test]
fn compliance_version_declaration() {
    let data = build_simple_ofd();
    let cursor = Cursor::new(&data[..]);
    let mut archive = ZipArchive::new(cursor).unwrap();
    let mut file = archive.by_name("OFD.xml").unwrap();
    let mut content = String::new();
    std::io::Read::read_to_string(&mut file, &mut content).unwrap();
    assert!(
        content.contains("Version=\"1.0\""),
        "OFD.xml must declare Version=\"1.0\""
    );
}
