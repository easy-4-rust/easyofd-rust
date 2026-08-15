//! 03_text_image_page —— 图文混排页面。
//!
//! 演示在同一页面上混合放置文本、图片和路径对象，
//! 模拟发票/报表等常见文档布局。
//! 产物写入 /tmp/easyofd_examples/03_text_image_page.ofd。
//!
//! 运行：
//!   cargo run --example 03_text_image_page

use easyofd::{EasyOfd, ImageObject, OfdPage, PathObject, TextObject};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir = std::env::temp_dir().join("easyofd_examples");
    std::fs::create_dir_all(&dir)?;
    let path = dir.join("03_text_image_page.ofd");

    let mut page = OfdPage::new(210.0, 297.0); // A4

    // ── 标题区 ───────────────────────────────────────────────────
    page.add_text(TextObject::new(20.0, 25.0, "销 售 发 票").size(24.0).bold());
    page.add_path(PathObject::hline(20.0, 40.0, 190.0));

    // ── 发票信息 ─────────────────────────────────────────────────
    page.add_text(TextObject::new(20.0, 50.0, "发票编号: INV-2026-0042"));
    page.add_text(TextObject::new(120.0, 50.0, "日期: 2026-08-16"));

    // ── 表头 ─────────────────────────────────────────────────────
    page.add_path(PathObject::hline(20.0, 65.0, 190.0));
    page.add_text(TextObject::new(20.0, 72.0, "商品名称").bold());
    page.add_text(TextObject::new(90.0, 72.0, "数量").bold());
    page.add_text(TextObject::new(130.0, 72.0, "单价").bold());
    page.add_text(TextObject::new(170.0, 72.0, "金额").bold());
    page.add_path(PathObject::hline(20.0, 80.0, 190.0));

    // ── 表体 ─────────────────────────────────────────────────────
    page.add_text(TextObject::new(20.0, 90.0, "EasyWidget Pro"));
    page.add_text(TextObject::new(90.0, 90.0, "10"));
    page.add_text(TextObject::new(130.0, 90.0, "¥99.00"));
    page.add_text(TextObject::new(170.0, 90.0, "¥990.00"));

    page.add_text(TextObject::new(20.0, 105.0, "Rust 编程指南"));
    page.add_text(TextObject::new(90.0, 105.0, "3"));
    page.add_text(TextObject::new(130.0, 105.0, "¥68.00"));
    page.add_text(TextObject::new(170.0, 105.0, "¥204.00"));

    page.add_path(PathObject::hline(20.0, 118.0, 190.0));

    // ── 合计 ─────────────────────────────────────────────────────
    page.add_text(TextObject::new(130.0, 125.0, "合计:").bold());
    page.add_text(
        TextObject::new(170.0, 125.0, "¥1,194.00")
            .bold()
            .color(0xCC_0000),
    );

    // ── 印章图片（占位 JPEG）─────────────────────────────────────
    let placeholder_jpeg = vec![0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10];
    page.add_image(ImageObject::jpeg(
        140.0,
        180.0,
        50.0,
        50.0,
        placeholder_jpeg,
    ));
    page.add_text(TextObject::new(140.0, 235.0, "（公章）").size(10.0));

    // ── 底部矩形框 ──────────────────────────────────────────────
    page.add_path(PathObject::rect(15.0, 260.0, 180.0, 25.0).stroke_color(0x00_CCCC));
    page.add_text(TextObject::new(20.0, 268.0, "谢谢惠顾！").size(10.0));

    // ── 写入并验证 ───────────────────────────────────────────────
    EasyOfd::write_pages(path.to_string_lossy().into_owned())
        .metadata_title("销售发票 INV-2026-0042")
        .metadata_author("easyofd-rust")
        .do_write(vec![page])?;

    println!("[写入] 文件: {}", path.display());
    println!("[写入] 大小: {} bytes", std::fs::metadata(&path)?.len());

    let reader = EasyOfd::read(&path)?;
    println!("[读回] 页数: {}", reader.page_count());
    println!("[读回] 全文:\n{}", reader.extract_all_text());

    // 清理
    let _ = std::fs::remove_file(&path);
    println!("\n示例完成。");
    Ok(())
}
