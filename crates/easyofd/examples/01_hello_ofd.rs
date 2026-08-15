//! 01_hello_ofd —— 最小写入 + 读回 roundtrip。
//!
//! 演示最简单的 OFD 操作：创建一页文档、写入文件、读回验证。
//! 产物写入 /tmp/easyofd_examples/01_hello_ofd.ofd。
//!
//! 运行：
//!   cargo run --example 01_hello_ofd

use easyofd::{EasyOfd, OfdPage, TextObject};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir = std::env::temp_dir().join("easyofd_examples");
    std::fs::create_dir_all(&dir)?;
    let path = dir.join("01_hello_ofd.ofd");

    // ── 写入：一页 A4，包含一行文本 ─────────────────────────────────
    let mut page = OfdPage::new(210.0, 297.0);
    page.add_text(
        TextObject::new(20.0, 30.0, "你好，OFD 世界！")
            .size(24.0)
            .bold(),
    );
    page.add_text(TextObject::new(
        20.0,
        60.0,
        "这是 easyofd-rust 的第一个示例。",
    ));

    EasyOfd::write_pages(path.to_string_lossy().into_owned())
        .metadata_title("Hello OFD")
        .metadata_author("easyofd-rust")
        .do_write(vec![page])?;

    println!("[写入] 文件: {}", path.display());
    println!("[写入] 大小: {} bytes", std::fs::metadata(&path)?.len());

    // ── 读回：验证内容 ─────────────────────────────────────────────
    let reader = EasyOfd::read(&path)?;
    println!("[读回] 页数: {}", reader.page_count());
    println!("[读回] 全文:\n{}", reader.extract_all_text());

    // 清理
    let _ = std::fs::remove_file(&path);
    println!("\n示例完成。");
    Ok(())
}
