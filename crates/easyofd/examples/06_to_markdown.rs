//! 06_to_markdown —— OFD 转 Markdown。
//!
//! 演示使用 `EasyOfd::to_markdown` 将 OFD 文档转换为 Markdown 格式，
//! 包含转换报告（损失、警告）和流式输出到文件。
//! 产物写入 /tmp/easyofd_examples/06_to_markdown.md。
//!
//! 运行：
//!   cargo run --example 06_to_markdown

use easyofd::{EasyOfd, OfdPage, PathObject, TextObject};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir = std::env::temp_dir().join("easyofd_examples");
    std::fs::create_dir_all(&dir)?;
    let ofd_path = dir.join("06_to_markdown_src.ofd");

    // ── 创建示例文档 ─────────────────────────────────────────────
    let mut page1 = OfdPage::new(210.0, 297.0);
    page1.add_text(TextObject::new(20.0, 25.0, "项目周报").size(22.0).bold());
    page1.add_path(PathObject::hline(20.0, 40.0, 190.0));
    page1.add_text(TextObject::new(20.0, 55.0, "日期: 2026-08-16"));
    page1.add_text(TextObject::new(20.0, 75.0, "本周完成:"));
    page1.add_text(TextObject::new(25.0, 90.0, "- OFD 读写器核心功能"));
    page1.add_text(TextObject::new(25.0, 105.0, "- 数字签名 SM2 集成"));
    page1.add_text(TextObject::new(25.0, 120.0, "- Markdown 导出功能"));

    let mut page2 = OfdPage::new(210.0, 297.0);
    page2.add_text(TextObject::new(20.0, 25.0, "下周计划").size(18.0).bold());
    page2.add_path(PathObject::hline(20.0, 40.0, 190.0));
    page2.add_text(TextObject::new(20.0, 55.0, "- PDF 转换优化"));
    page2.add_text(TextObject::new(20.0, 70.0, "- 归档合规检查"));

    EasyOfd::write_pages(ofd_path.to_string_lossy().into_owned()).do_write(vec![page1, page2])?;
    println!("[源文件] {}", ofd_path.display());

    // ── 转换到 Markdown（内存模式）────────────────────────────────
    println!("\n=== 内存模式转换 ===");
    let result = EasyOfd::to_markdown(&ofd_path).do_convert()?;
    println!("转换页数: {}", result.report.pages_converted);
    println!("损失数量: {}", result.report.losses.len());
    println!("警告数量: {}", result.report.warnings.len());
    println!("\n--- Markdown 输出 ---\n{}", result.markdown);

    // ── 转换到 Markdown（流式写文件）──────────────────────────────
    let md_path = dir.join("06_to_markdown.md");
    println!("=== 流式写文件 ===");
    let report = EasyOfd::to_markdown(&ofd_path).convert_to(std::fs::File::create(&md_path)?)?;
    println!("已写入: {}", md_path.display());
    println!("转换页数: {}", report.pages_converted);
    if !report.losses.is_empty() {
        println!("转换损失:");
        for loss in &report.losses {
            println!(
                "  - [{}] 页={}, 策略={}",
                loss.feature, loss.page, loss.policy
            );
        }
    }

    // 清理
    let _ = std::fs::remove_file(&ofd_path);
    let _ = std::fs::remove_file(&md_path);
    println!("\n示例完成。");
    Ok(())
}
