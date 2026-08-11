//! 创建含注释的 OFD 文档。
//!
//! 本示例演示如何使用 GB/T 33190 第 16 章定义的注释模型
//! 在 OFD 文档中添加各类注释：文本注释、高亮注释、印章注释。
//!
//! 流程：
//!   1. 创建包含内容的 OFD 页面
//!   2. 构造多种类型的注释对象
//!   3. 组装注释容器（Annotations XML）
//!   4. 写入 OFD 并验证
//!
//! 用法：
//!   cargo run --release --example annotation

use easyofd::{
    AnnPage, Annot, AnnotType, Annotations, Appearance, EasyOfd, OfdPage, PathObject, TextObject,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir = std::env::temp_dir().join("easyofd_example_annotation");
    std::fs::create_dir_all(&dir)?;
    let ofd_path = dir.join("annotation.ofd");

    // ── Step 1: 创建 OFD 页面 ────────────────────────────────────────────
    let mut page = OfdPage::new(210.0, 297.0); // A4

    page.add_text(
        TextObject::new(20.0, 30.0, "OFD 注释功能演示")
            .size(22.0)
            .bold(),
    );
    page.add_path(PathObject::hline(20.0, 45.0, 190.0));

    page.add_text(TextObject::new(
        20.0,
        60.0,
        "GB/T 33190 第 16 章定义了 OFD 文档的注释模型。",
    ));
    page.add_text(TextObject::new(
        20.0,
        80.0,
        "注释类型包括：文本(Text)、高亮(Highlight)、印章(Stamp)、手写(Handwritten)等。",
    ));

    page.add_path(PathObject::hline(20.0, 100.0, 190.0));

    page.add_text(TextObject::new(20.0, 115.0, "以下区域可以添加注释："));
    page.add_text(
        TextObject::new(20.0, 135.0, "重要段落：此段落需要审阅。")
            .bold()
            .color(0xCC_0000),
    );
    page.add_text(TextObject::new(
        20.0,
        155.0,
        "参考链接：请参阅 GB/T 33190-2016 标准全文。",
    ));

    // ── Step 2: 构造各类注释 ─────────────────────────────────────────────

    // 文本注释（便签）：用户可以在指定位置留下批注。
    let text_annot = Annot::new("annot_text_001", AnnotType::Text)
        .creator("审阅者A")
        .flags(0x01) // 可打印
        .last_mod_date("2026-08-10T10:30:00")
        .location(150.0, 130.0, 40.0, 20.0)
        .add_appearance(Appearance::new("text_app_normal", "Normal"));

    // 高亮注释：标记重要段落。
    let highlight_annot = Annot::new("annot_hl_001", AnnotType::Highlight)
        .creator("审阅者A")
        .flags(0x01)
        .last_mod_date("2026-08-10T10:35:00")
        .location(20.0, 130.0, 120.0, 10.0)
        .add_appearance(Appearance::new("hl_app_normal", "Normal"));

    // 印章注释：用于审批盖章。
    let stamp_annot = Annot::new("annot_stamp_001", AnnotType::Stamp)
        .creator("审批系统")
        .flags(0x05) // 可打印 + 锁定
        .last_mod_date("2026-08-10T11:00:00")
        .location(140.0, 200.0, 50.0, 50.0)
        .add_appearance(Appearance::new("stamp_app_normal", "Normal"));

    // 链接注释：关联外部资源。
    let link_annot = Annot::new("annot_link_001", AnnotType::Link)
        .creator("easyofd-rust")
        .last_mod_date("2026-08-10T11:05:00")
        .location(20.0, 150.0, 100.0, 10.0)
        .add_appearance(Appearance::new("link_app_normal", "Normal"));

    // 手写注释：模拟签名或手写批注。
    let handwritten_annot = Annot::new("annot_hw_001", AnnotType::Handwritten)
        .creator("签名人")
        .last_mod_date("2026-08-10T11:10:00")
        .location(20.0, 220.0, 80.0, 30.0)
        .add_appearance(Appearance::new("hw_app_normal", "Normal"));

    // ── Step 3: 组装注释容器 ─────────────────────────────────────────────

    // AnnPage：文档级注释索引，描述某一页关联的注释文件。
    let ann_page = AnnPage::new(0).annot_file("Doc_0/Annots/Annotations_0.xml");

    // Annotations：文档级注释容器。
    let annotations = Annotations::new()
        .doc_id("annotation-demo-doc")
        .add_page(ann_page);

    // 输出注释 XML 用于演示。
    println!("=== 注释容器 XML ===\n");
    println!("{}\n", annotations.to_xml_string());

    // 输出各类注释的详细信息。
    let all_annots = [
        &text_annot,
        &highlight_annot,
        &stamp_annot,
        &link_annot,
        &handwritten_annot,
    ];

    println!("=== 注释摘要（共 {} 个）===", all_annots.len());
    for ann in &all_annots {
        println!(
            "  [{}] ID={}, 位置=({:.0},{:.0},{:.0},{:.0}), 创建者={}",
            ann.annot_type.as_str(),
            ann.id,
            ann.location[0],
            ann.location[1],
            ann.location[2],
            ann.location[3],
            ann.creator.as_deref().unwrap_or("(无)")
        );
    }

    // 输出各注释的 XML。
    println!("\n=== 注释对象 XML ===\n");
    for ann in &all_annots {
        println!("{}\n", ann.to_xml_string());
    }

    // ── Step 4: 写入 OFD 并验证 ─────────────────────────────────────────
    EasyOfd::write_pages(ofd_path.to_string_lossy().into_owned())
        .metadata_title("注释功能演示")
        .metadata_author("easyofd-rust")
        .metadata_creator("annotation example")
        .do_write(vec![page])?;

    println!("已生成 OFD 文件: {}", ofd_path.display());
    println!("文件大小: {} bytes", std::fs::metadata(&ofd_path)?.len());

    // 读取并验证。
    println!("\n=== 读取验证 ===");
    let reader = EasyOfd::read(&ofd_path)?;
    println!("页数: {}", reader.page_count());
    println!("文本内容:\n{}", reader.extract_all_text());

    // 清理。
    let _ = std::fs::remove_dir_all(dir);
    println!("\n所有步骤完成。");
    Ok(())
}
