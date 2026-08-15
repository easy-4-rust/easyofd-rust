//! 09_archive_check —— OFD-A 归档规则检查。
//!
//! 演示使用 `easyofd_archive` 的合规规则引擎检查 OFD 文档是否符合
//! GB/T 33190 归档要求：DocType、Version、DocRoot、Pages、外部资源。
//! 产物写入 /tmp/easyofd_examples/09_archive_check/。
//!
//! 注意：本示例直接使用 easyofd_archive 子 crate，因为归档检查
//! 功能未通过 EasyOfd facade 暴露（facade 易用性缺口）。
//!
//! 运行：
//!   cargo run --example 09_archive_check

use easyofd::{EasyOfd, OfdPage, TextObject};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir = std::env::temp_dir()
        .join("easyofd_examples")
        .join("09_archive_check");
    std::fs::create_dir_all(&dir)?;

    // ── 创建合规的 OFD 文档 ──────────────────────────────────────
    let good_path = dir.join("compliant.ofd");
    let mut page = OfdPage::new(210.0, 297.0);
    page.add_text(
        TextObject::new(20.0, 30.0, "合规文档测试")
            .size(18.0)
            .bold(),
    );
    page.add_text(TextObject::new(
        20.0,
        55.0,
        "此文档由 easyofd-writer 生成，应通过全部合规规则。",
    ));

    EasyOfd::write_pages(good_path.to_string_lossy().into_owned())
        .metadata_title("合规测试文档")
        .metadata_author("easyofd-rust")
        .do_write(vec![page])?;

    let good_bytes = std::fs::read(&good_path)?;
    println!(
        "[合规文档] {} ({} bytes)",
        good_path.display(),
        good_bytes.len()
    );

    // ── 合规检查 ─────────────────────────────────────────────────
    println!("\n=== 合规规则检查（合规文档）===");
    let results = easyofd_archive::check_compliance(&good_bytes)?;
    for r in &results {
        let status = if r.passed { "PASS" } else { "FAIL" };
        println!("  [{status}] {}", r.message);
    }

    // ── 完整性校验 ───────────────────────────────────────────────
    println!("\n=== 完整性校验 ===");
    let integrity = easyofd_archive::verify_integrity(&good_bytes)?;
    println!("  校验通过: {}", integrity.passed);
    println!("  摘要不匹配: {} 个文件", integrity.failed_files.len());
    println!("  缺失文件: {} 个", integrity.missing_files.len());

    // ── 构造不合规的 OFD（缺少 DocType）──────────────────────────
    println!("\n=== 不合规文档测试 ===");
    let bad_bytes = build_non_compliant_zip();
    let bad_results = easyofd_archive::check_compliance(&bad_bytes)?;
    for r in &bad_results {
        let status = if r.passed { "PASS" } else { "FAIL" };
        println!("  [{status}] {}", r.message);
    }

    // 清理
    let _ = std::fs::remove_dir_all(&dir);
    println!("\n示例完成。");
    Ok(())
}

/// 构造一个不合规的 OFD ZIP（缺少 DocType 属性，版本错误）。
fn build_non_compliant_zip() -> Vec<u8> {
    use std::io::Write;

    let mut buf = std::io::Cursor::new(Vec::new());
    {
        let mut zip = zip::ZipWriter::new(&mut buf);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);

        zip.start_file("OFD.xml", options).expect("OFD.xml");
        zip.write_all(
            br#"<?xml version="1.0" encoding="UTF-8"?>
<ofd:OFD xmlns:ofd="http://www.ofdspec.org/2016" Version="1.0">
  <ofd:DocBody><ofd:DocRoot>Doc_0/Document.xml</ofd:DocRoot></ofd:DocBody>
</ofd:OFD>"#,
        )
        .expect("write OFD.xml");

        zip.start_file("Doc_0/Document.xml", options)
            .expect("Document.xml");
        zip.write_all(
            br#"<?xml version="1.0" encoding="UTF-8"?>
<ofd:Document xmlns:ofd="http://www.ofdspec.org/2016">
  <ofd:Pages><ofd:Page ID="1" BaseLoc="Pages/Page_0.xml"/></ofd:Pages>
</ofd:Document>"#,
        )
        .expect("write Document.xml");

        zip.start_file("Doc_0/Pages/Page_0.xml", options)
            .expect("Page_0.xml");
        zip.write_all(b"<ofd:Page/>").expect("write Page_0.xml");

        zip.finish().expect("finish zip");
    }
    buf.into_inner()
}
