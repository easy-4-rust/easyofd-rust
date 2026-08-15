//! 11_keyword_search —— 关键字定位（含跨 TextCode 匹配）。
//!
//! 演示使用 `KeywordExtractor` 在 OFD 文档中搜索关键字，
//! 返回匹配位置的页码和矩形区域。支持两种模式：
//!   - 简化模式：基于 OfdPage 模型搜索
//!   - 完整模式：基于 TextCodeEntry 列表，支持跨 TextCode 边界匹配
//!
//! 产物写入 /tmp/easyofd_examples/11_keyword_search/。
//!
//! 注意：本示例直接使用 easyofd_reader::KeywordExtractor，
//! 因为关键字搜索功能未通过 EasyOfd facade 暴露（facade 易用性缺口）。
//!
//! 运行：
//!   cargo run --example 11_keyword_search

use easyofd::{EasyOfd, OfdPage, TextObject};
use easyofd_core::ST_Box;
use easyofd_reader::{KeywordExtractor, TextCodeEntry};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir = std::env::temp_dir()
        .join("easyofd_examples")
        .join("11_keyword_search");
    std::fs::create_dir_all(&dir)?;
    let ofd_path = dir.join("search_demo.ofd");

    // ── 创建包含多个关键字的文档 ─────────────────────────────────
    let mut page1 = OfdPage::new(210.0, 297.0);
    page1.add_text(
        TextObject::new(20.0, 30.0, "OFD 文档格式标准")
            .size(18.0)
            .bold(),
    );
    page1.add_text(TextObject::new(
        20.0,
        55.0,
        "OFD 是中国的国家标准文档格式。",
    ));
    page1.add_text(TextObject::new(
        20.0,
        75.0,
        "OFD 格式支持文本、图片、路径等多种内容类型。",
    ));

    let mut page2 = OfdPage::new(210.0, 297.0);
    page2.add_text(
        TextObject::new(20.0, 30.0, "easyofd-rust 项目")
            .size(18.0)
            .bold(),
    );
    page2.add_text(TextObject::new(
        20.0,
        55.0,
        "easyofd-rust 是 OFD 格式的 Rust 实现。",
    ));
    page2.add_text(TextObject::new(
        20.0,
        75.0,
        "项目目标：零 unsafe 代码，完整支持 OFD 标准。",
    ));

    EasyOfd::write_pages(ofd_path.to_string_lossy().into_owned())
        .metadata_title("关键字搜索演示")
        .do_write(vec![page1, page2])?;

    println!("[文档] {}", ofd_path.display());

    // ── 模式 1: 简化模式搜索 ─────────────────────────────────────
    let reader = EasyOfd::read(&ofd_path)?;
    let doc_pages = reader.pages();

    println!("\n=== 简化模式: 搜索 'OFD' ===");
    let positions = KeywordExtractor::get_keyword_positions(doc_pages, "OFD");
    println!("  找到 {} 个匹配:", positions.len());
    for pos in &positions {
        println!(
            "    页={}, 位置=({:.1}, {:.1}), 区域={:.1}×{:.1} mm",
            pos.page,
            pos.x(),
            pos.y(),
            pos.width(),
            pos.height()
        );
    }

    println!("\n=== 简化模式: 搜索 'Rust' ===");
    let positions = KeywordExtractor::get_keyword_positions(doc_pages, "Rust");
    println!("  找到 {} 个匹配:", positions.len());
    for pos in &positions {
        println!("    页={}, 位置=({:.1}, {:.1})", pos.page, pos.x(), pos.y());
    }

    // ── 模式 2: 跨 TextCode 边界匹配 ────────────────────────────
    // 模拟关键字被 TextCode 边界切断的场景
    println!("\n=== 完整模式: 跨 TextCode 边界搜索 ===");
    let boundary = ST_Box::new(0.0, 0.0, 210.0, 297.0);

    let entries = vec![
        // "电子" 在第一个 TextCode
        TextCodeEntry::new("电子", 1, boundary.clone(), 3.0).coordinate(10.0, 50.0),
        // "印章" 在第二个 TextCode（关键字 "电子印章" 被切断）
        TextCodeEntry::new("印章", 1, boundary.clone(), 3.0).coordinate(30.0, 50.0),
        // 普通文本
        TextCodeEntry::new("是文档中的重要元素", 1, boundary.clone(), 3.0).coordinate(50.0, 50.0),
    ];

    let positions = KeywordExtractor::get_keyword_positions_from_text_codes(&entries, "电子印章");
    println!(
        "  搜索 '电子印章' (跨 TextCode): {} 个匹配",
        positions.len()
    );
    for pos in &positions {
        println!(
            "    页={}, 区域=({:.1}, {:.1}, {:.1}, {:.1})",
            pos.page, pos.rect.top_left_x, pos.rect.top_left_y, pos.rect.width, pos.rect.height
        );
    }

    // 验证普通匹配也能工作
    let positions_single =
        KeywordExtractor::get_keyword_positions_from_text_codes(&entries, "电子");
    println!(
        "\n  搜索 '电子' (单 TextCode): {} 个匹配",
        positions_single.len()
    );

    // 清理
    let _ = std::fs::remove_dir_all(&dir);
    println!("\n示例完成。");
    Ok(())
}
