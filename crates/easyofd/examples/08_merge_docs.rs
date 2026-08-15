//! 08_merge_docs —— 多文档合并。
//!
//! 演示将多个 OFD 文档的页面合并到一个新文档中：
//!   1. 分别创建 3 个独立的 OFD 文档
//!   2. 逐个读取并收集所有页面
//!   3. 写入合并后的新文档
//!
//! 注意：OfdEditor 的 save() 在多源合并场景下会产生 ZIP 条目冲突，
//! 因此本示例使用读取页面 + 写入新文档的方式实现合并。
//! 产物写入 /tmp/easyofd_examples/08_merge_docs/。
//!
//! 运行：
//!   cargo run --example 08_merge_docs

use easyofd::{EasyOfd, ImageObject, OfdPage, TextObject};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir = std::env::temp_dir()
        .join("easyofd_examples")
        .join("08_merge_docs");
    std::fs::create_dir_all(&dir)?;

    // ── 创建 3 个独立文档 ────────────────────────────────────────
    let doc_paths: Vec<_> = (1..=3)
        .map(|i| {
            let path = dir.join(format!("doc_{i}.ofd"));
            let mut page = OfdPage::new(210.0, 297.0);
            page.add_text(
                TextObject::new(20.0, 30.0, format!("文档 {i} 标题"))
                    .size(18.0)
                    .bold(),
            );
            page.add_text(TextObject::new(
                20.0,
                55.0,
                format!("这是第 {i} 个文档的内容。"),
            ));
            // 每个文档都有图片（不同数据，验证合并后资源正确保留）
            page.add_image(ImageObject::jpeg(
                150.0,
                30.0,
                30.0,
                30.0,
                vec![
                    0xFF,
                    0xD8,
                    0xFF,
                    0xE0,
                    0x00,
                    u8::try_from(i).unwrap_or(0),
                    0x01,
                    0x02,
                ],
            ));
            EasyOfd::write_pages(path.to_string_lossy().into_owned())
                .metadata_title(format!("文档 {i}"))
                .do_write(vec![page])
                .expect("写入文档");
            println!(
                "[创建] doc_{i}.ofd ({} bytes)",
                std::fs::metadata(&path).expect("metadata").len()
            );
            path
        })
        .collect();

    // ── 逐个读取并收集所有页面 ───────────────────────────────────
    // 注意：合并多源文档时，需要清除图片的 res_name，避免不同文档的
    // 同名资源（如 Image_0.jpeg）在合并 ZIP 中冲突。
    let mut all_pages: Vec<OfdPage> = Vec::new();
    for (i, path) in doc_paths.iter().enumerate() {
        let reader = EasyOfd::read(path)?;
        let count = reader.page_count();
        for page in reader.pages() {
            let mut p = page.clone();
            // 清除原始路径和资源名，避免多源文档合并时 ZIP 条目冲突
            p.base_path = None;
            for obj in &mut p.content {
                if let easyofd::ContentObject::Image(img) = obj {
                    img.res_name = None;
                }
            }
            all_pages.push(p);
        }
        println!(
            "[读取] doc_{}: {} 页, 累计 {} 页",
            i + 1,
            count,
            all_pages.len()
        );
    }

    // ── 写入合并后的新文档 ───────────────────────────────────────
    let merged_path = dir.join("merged.ofd");
    EasyOfd::write_pages(merged_path.to_string_lossy().into_owned())
        .metadata_title("合并文档")
        .metadata_author("easyofd-rust")
        .do_write(all_pages.clone())?;

    println!("\n[结果] 合并文档: {}", merged_path.display());
    println!(
        "[结果] 大小: {} bytes",
        std::fs::metadata(&merged_path)?.len()
    );

    // ── 验证合并结果 ─────────────────────────────────────────────
    let reader = EasyOfd::read(&merged_path)?;
    println!("[验证] 总页数: {}", reader.page_count());
    for (i, text) in reader.extract_text().iter().enumerate() {
        let preview = text.lines().next().unwrap_or("");
        println!("[验证] 第 {} 页: {}", i + 1, preview);
    }

    // ── 验证图片资源保留 ─────────────────────────────────────────
    let merged_bytes = std::fs::read(&merged_path)?;
    let cursor = std::io::Cursor::new(&merged_bytes);
    let archive = zip::ZipArchive::new(cursor);
    if let Ok(mut archive) = archive {
        let image_count = (0..archive.len())
            .filter(|&i| {
                archive
                    .by_index(i)
                    .is_ok_and(|e| e.name().contains("Image_"))
            })
            .count();
        println!("[验证] 图片资源数: {image_count}");
    }

    // 清理
    let _ = std::fs::remove_dir_all(&dir);
    println!("\n示例完成。");
    Ok(())
}
