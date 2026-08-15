//! 02_read_metadata —— 读取 OFD 元数据、页数、页面信息。
//!
//! 演示如何从 OFD 文件中提取文档标题、作者、创建者等元数据，
//! 以及逐页遍历页面尺寸和内容对象数量。
//! 产物写入 /tmp/easyofd_examples/02_read_metadata.ofd。
//!
//! 运行：
//!   cargo run --example 02_read_metadata

use easyofd::{ContentObject, EasyOfd, ImageObject, OfdPage, PathObject, TextObject};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir = std::env::temp_dir().join("easyofd_examples");
    std::fs::create_dir_all(&dir)?;
    let path = dir.join("02_read_metadata.ofd");

    // ── 创建带元数据的多页文档 ─────────────────────────────────────
    let mut page1 = OfdPage::new(210.0, 297.0); // A4
    page1.add_text(
        TextObject::new(20.0, 30.0, "第一页：文本内容")
            .size(18.0)
            .bold(),
    );
    page1.add_text(TextObject::new(20.0, 55.0, "包含纯文本和路径对象。"));
    page1.add_path(PathObject::hline(20.0, 70.0, 190.0));

    let mut page2 = OfdPage::new(297.0, 210.0); // A4 横向
    page2.add_text(TextObject::new(20.0, 30.0, "第二页：横向页面").size(18.0));
    page2.add_image(ImageObject::jpeg(
        20.0,
        50.0,
        40.0,
        40.0,
        vec![0xFF, 0xD8, 0xFF, 0xE0],
    ));

    EasyOfd::write_pages(path.to_string_lossy().into_owned())
        .metadata_title("元数据示例文档")
        .metadata_author("easyofd-rust")
        .metadata_creator("02_read_metadata 示例")
        .do_write(vec![page1, page2])?;

    println!(
        "[写入] 文件: {} ({} bytes)\n",
        path.display(),
        std::fs::metadata(&path)?.len()
    );

    // ── 读取元数据 ────────────────────────────────────────────────
    let reader = EasyOfd::read(&path)?;
    let meta = reader.metadata();

    println!("=== 文档元数据 ===");
    println!("  标题:   {}", meta.title.as_deref().unwrap_or("(无)"));
    println!("  作者:   {}", meta.author.as_deref().unwrap_or("(无)"));
    println!("  创建者: {}", meta.creator.as_deref().unwrap_or("(无)"));
    println!("  页数:   {}", reader.page_count());

    // ── 逐页信息 ─────────────────────────────────────────────────
    println!("\n=== 页面详情 ===");
    for (i, page) in reader.pages().iter().enumerate() {
        let text_count = page
            .content
            .iter()
            .filter(|o| matches!(o, ContentObject::Text(_)))
            .count();
        let image_count = page
            .content
            .iter()
            .filter(|o| matches!(o, ContentObject::Image(_)))
            .count();
        let path_count = page
            .content
            .iter()
            .filter(|o| matches!(o, ContentObject::Path(_)))
            .count();
        println!(
            "  第 {} 页: {:.0}×{:.0} mm, 文本={}, 图片={}, 路径={}",
            i + 1,
            page.width,
            page.height,
            text_count,
            image_count,
            path_count
        );
    }

    // 清理
    let _ = std::fs::remove_file(&path);
    println!("\n示例完成。");
    Ok(())
}
