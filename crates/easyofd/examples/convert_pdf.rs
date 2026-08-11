//! OFD 到 PDF 转换示例。
//!
//! 本示例演示如何将 OFD 文档转换为 PDF 格式，以及反向转换。
//! 使用 easyofd-convert 模块提供的 `ofd_to_pdf` 和 `pdf_to_ofd` 函数。
//!
//! 流程：
//!   1. 创建 OFD 文档（文本 + 路径）
//!   2. OFD → PDF 转换
//!   3. PDF → OFD 反向转换
//!   4. 读取并验证转换结果
//!
//! 用法：
//!   cargo run --release --example convert_pdf

use easyofd::{ConvertOptions, EasyOfd, OfdPage, PathObject, TextObject, ofd_to_pdf, pdf_to_ofd};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir = std::env::temp_dir().join("easyofd_example_convert_pdf");
    std::fs::create_dir_all(&dir)?;

    // ── Step 1: 创建 OFD 文档 ────────────────────────────────────────────
    let ofd_path = dir.join("input.ofd");

    let mut page1 = OfdPage::new(210.0, 297.0); // A4
    page1.add_text(
        TextObject::new(20.0, 30.0, "OFD 转 PDF 示例文档")
            .size(24.0)
            .bold(),
    );
    page1.add_path(PathObject::hline(20.0, 45.0, 190.0));
    page1.add_text(TextObject::new(
        20.0,
        60.0,
        "本文档演示 OFD 与 PDF 之间的双向转换功能。",
    ));
    page1.add_text(TextObject::new(
        20.0,
        80.0,
        "OFD (Open Fixed-layout Document) 是中国国家标准 GB/T 33190-2016。",
    ));
    page1.add_text(TextObject::new(
        20.0,
        100.0,
        "PDF (Portable Document Format) 是国际通用的文档格式。",
    ));
    page1.add_path(PathObject::hline(20.0, 120.0, 190.0));
    page1.add_text(TextObject::new(
        20.0,
        135.0,
        "转换器支持文本和路径对象的映射。",
    ));

    let mut page2 = OfdPage::new(210.0, 297.0);
    page2.add_text(
        TextObject::new(20.0, 30.0, "第二页 - 更多内容")
            .size(18.0)
            .bold(),
    );
    page2.add_text(TextObject::new(20.0, 55.0, "多页文档也能正确转换。"));
    page2.add_path(PathObject::rect(20.0, 70.0, 170.0, 40.0));
    page2.add_text(TextObject::new(
        30.0,
        90.0,
        "矩形框内的文本也能保留位置信息。",
    ));

    EasyOfd::write_pages(ofd_path.to_string_lossy().into_owned())
        .metadata_title("OFD 转 PDF 示例")
        .metadata_author("easyofd-rust")
        .metadata_creator("convert_pdf example")
        .do_write(vec![page1, page2])?;

    println!("Step 1: 已创建 OFD 文件: {}", ofd_path.display());
    println!("  文件大小: {} bytes", std::fs::metadata(&ofd_path)?.len());

    // ── Step 2: OFD → PDF 转换 ───────────────────────────────────────────
    let pdf_path = dir.join("output.pdf");
    let options = ConvertOptions::default(); // 转换所有页面

    ofd_to_pdf(&ofd_path, &pdf_path, &options)?;

    println!("\nStep 2: OFD → PDF 转换完成");
    println!("  PDF 文件: {}", pdf_path.display());
    println!("  文件大小: {} bytes", std::fs::metadata(&pdf_path)?.len());

    // 转换指定页面范围。
    let pdf_page1_path = dir.join("output_page1.pdf");
    let partial_options = ConvertOptions {
        pages: 0..1, // 只转换第 1 页
        page_size: None,
    };
    ofd_to_pdf(&ofd_path, &pdf_page1_path, &partial_options)?;
    println!(
        "  部分转换 (第1页): {} bytes",
        std::fs::metadata(&pdf_page1_path)?.len()
    );

    // ── Step 3: PDF → OFD 反向转换 ───────────────────────────────────────
    let roundtrip_ofd_path = dir.join("roundtrip.ofd");

    pdf_to_ofd(&pdf_path, &roundtrip_ofd_path, &ConvertOptions::default())?;

    println!("\nStep 3: PDF → OFD 反向转换完成");
    println!("  OFD 文件: {}", roundtrip_ofd_path.display());
    println!(
        "  文件大小: {} bytes",
        std::fs::metadata(&roundtrip_ofd_path)?.len()
    );

    // ── Step 4: 验证转换结果 ─────────────────────────────────────────────
    println!("\nStep 4: 验证转换结果");

    // 读取原始 OFD。
    let original_reader = EasyOfd::read(&ofd_path)?;
    println!("  原始 OFD: {} 页", original_reader.page_count());

    // 读取反向转换的 OFD。
    let roundtrip_reader = EasyOfd::read(&roundtrip_ofd_path)?;
    println!("  反向转换 OFD: {} 页", roundtrip_reader.page_count());
    println!("  提取文本:\n{}", roundtrip_reader.extract_all_text());

    // 使用 Markdown 转换验证内容可读性。
    let md_result = EasyOfd::to_markdown(&ofd_path).do_convert()?;
    println!("\n=== Markdown 输出 ===");
    println!("{}", md_result.markdown);
    println!("转换页数: {}", md_result.report.pages_converted);

    // 清理。
    let _ = std::fs::remove_dir_all(dir);
    println!("\n所有步骤完成。");
    Ok(())
}
