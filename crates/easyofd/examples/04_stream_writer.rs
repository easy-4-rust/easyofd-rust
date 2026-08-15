//! 04_stream_writer —— 流式写入大文档。
//!
//! 演示使用 `EasyOfd::stream_writer` 逐页写入大文档，
//! 内存占用恒定（每页独立写入 ZIP），适合万页级场景。
//! 产物写入 /tmp/easyofd_examples/04_stream_writer.ofd。
//!
//! 运行：
//!   cargo run --example 04_stream_writer

use easyofd::{EasyOfd, OfdPage, TextObject};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir = std::env::temp_dir().join("easyofd_examples");
    std::fs::create_dir_all(&dir)?;
    let path = dir.join("04_stream_writer.ofd");

    let total_pages = 100;

    // ── 流式写入：逐页构建并写入 ZIP ─────────────────────────────
    let file = std::fs::File::create(&path)?;
    let mut writer = EasyOfd::stream_writer(file);

    for i in 1..=total_pages {
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_text(
            TextObject::new(10.0, 15.0, format!("第 {i} 页"))
                .size(16.0)
                .bold(),
        );
        page.add_text(TextObject::new(
            10.0,
            35.0,
            format!("这是流式写入文档的第 {i} 页，共 {total_pages} 页。"),
        ));
        page.add_text(TextObject::new(
            10.0,
            55.0,
            "流式写入器 (OfdStreamWriter) 适合生成大文档，内存占用恒定。",
        ));
        writer.write_page(page)?;
    }
    writer.finish()?;

    println!("[写入] 文件: {}", path.display());
    println!(
        "[写入] 大小: {} bytes ({} 页)",
        std::fs::metadata(&path)?.len(),
        total_pages
    );

    // ── 验证：读回前 3 页和最后 1 页 ─────────────────────────────
    let reader = EasyOfd::read(&path)?;
    println!("[读回] 总页数: {}", reader.page_count());

    let first_page_text = easyofd::OfdReader::extract_text(&reader);
    if let Some(text) = first_page_text.first() {
        println!("[读回] 第 1 页: {}", text.lines().next().unwrap_or(""));
    }
    if let Some(text) = first_page_text.last() {
        println!(
            "[读回] 第 {} 页: {}",
            first_page_text.len(),
            text.lines().next().unwrap_or("")
        );
    }

    // 清理
    let _ = std::fs::remove_file(&path);
    println!("\n示例完成。");
    Ok(())
}
