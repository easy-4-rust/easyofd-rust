//! 10_convert_pdf —— OFD 与 PDF 双向转换。
//!
//! 演示使用 `ofd_to_pdf` 和 `pdf_to_ofd` 进行格式互转，
//! 包括指定页面范围转换和转换结果验证。
//! 产物写入 /tmp/easyofd_examples/10_convert_pdf/。
//!
//! 运行：
//!   cargo run --example 10_convert_pdf

use easyofd::{ConvertOptions, EasyOfd, OfdPage, PathObject, TextObject, ofd_to_pdf, pdf_to_ofd};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir = std::env::temp_dir()
        .join("easyofd_examples")
        .join("10_convert_pdf");
    std::fs::create_dir_all(&dir)?;

    // ── Step 1: 创建 OFD 文档 ────────────────────────────────────
    let ofd_path = dir.join("input.ofd");

    let mut page1 = OfdPage::new(210.0, 297.0);
    page1.add_text(
        TextObject::new(20.0, 30.0, "OFD 转 PDF 演示")
            .size(24.0)
            .bold(),
    );
    page1.add_path(PathObject::hline(20.0, 45.0, 190.0));
    page1.add_text(TextObject::new(
        20.0,
        60.0,
        "OFD (Open Fixed-layout Document) 是中国国家标准 GB/T 33190-2016。",
    ));
    page1.add_text(TextObject::new(
        20.0,
        80.0,
        "PDF (Portable Document Format) 是国际通用的文档格式。",
    ));

    let mut page2 = OfdPage::new(210.0, 297.0);
    page2.add_text(TextObject::new(20.0, 30.0, "第二页").size(18.0).bold());
    page2.add_text(TextObject::new(20.0, 55.0, "多页文档也能正确转换。"));
    page2.add_path(PathObject::rect(20.0, 70.0, 170.0, 40.0));

    EasyOfd::write_pages(ofd_path.to_string_lossy().into_owned())
        .metadata_title("OFD 转 PDF 示例")
        .metadata_author("easyofd-rust")
        .do_write(vec![page1, page2])?;

    println!(
        "[Step 1] OFD 文件: {} ({} bytes)",
        ofd_path.display(),
        std::fs::metadata(&ofd_path)?.len()
    );

    // ── Step 2: OFD → PDF（全部页面）──────────────────────────────
    let pdf_path = dir.join("output.pdf");
    ofd_to_pdf(&ofd_path, &pdf_path, &ConvertOptions::default())?;
    println!(
        "\n[Step 2] OFD → PDF: {} ({} bytes)",
        pdf_path.display(),
        std::fs::metadata(&pdf_path)?.len()
    );

    // ── Step 3: OFD → PDF（仅第 1 页）────────────────────────────
    let pdf_page1_path = dir.join("output_page1.pdf");
    ofd_to_pdf(
        &ofd_path,
        &pdf_page1_path,
        &ConvertOptions {
            pages: 0..1,
            page_size: None,
        },
    )?;
    println!(
        "[Step 3] 部分转换 (第1页): {} bytes",
        std::fs::metadata(&pdf_page1_path)?.len()
    );

    // ── Step 4: PDF → OFD 反向转换 ───────────────────────────────
    let roundtrip_path = dir.join("roundtrip.ofd");
    pdf_to_ofd(&pdf_path, &roundtrip_path, &ConvertOptions::default())?;
    println!(
        "\n[Step 4] PDF → OFD: {} ({} bytes)",
        roundtrip_path.display(),
        std::fs::metadata(&roundtrip_path)?.len()
    );

    // ── Step 5: 验证反向转换 ─────────────────────────────────────
    let reader = EasyOfd::read(&roundtrip_path)?;
    println!("\n[Step 5] 验证反向转换:");
    println!("  页数: {}", reader.page_count());
    println!("  提取文本:\n{}", reader.extract_all_text());

    // 清理
    let _ = std::fs::remove_dir_all(&dir);
    println!("\n示例完成。");
    Ok(())
}
