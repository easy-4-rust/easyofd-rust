//! 创建含超链接（URI Action）的 OFD 文档。
//!
//! 本示例演示如何在 OFD 文档中嵌入 URI 超链接动作。
//! GB/T 33190 第 15 章定义了动作模型，其中 URI 动作用于打开外部链接。
//!
//! 流程：
//!   1. 创建包含文本内容的 OFD 页面
//!   2. 构造 URI 动作的 XML 描述
//!   3. 构造 Link 注释的 XML 描述
//!   4. 读取并验证内容
//!
//! 用法：
//!   cargo run --release --example action_uri

use easyofd::{
    AnnPage, Annot, AnnotType, Annotations, Appearance, CTAction, EasyOfd, EventType, OfdPage,
    PathObject, TextObject, URI,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir = std::env::temp_dir().join("easyofd_example_action_uri");
    std::fs::create_dir_all(&dir)?;
    let ofd_path = dir.join("action_uri.ofd");

    // ── Step 1: 创建 OFD 页面 ────────────────────────────────────────────
    let mut page = OfdPage::new(210.0, 297.0); // A4

    page.add_text(
        TextObject::new(20.0, 30.0, "OFD 超链接示例")
            .size(22.0)
            .bold(),
    );
    page.add_path(PathObject::hline(20.0, 45.0, 190.0));

    page.add_text(TextObject::new(
        20.0,
        60.0,
        "以下链接可在支持动作的 OFD 阅读器中点击：",
    ));
    page.add_text(
        TextObject::new(20.0, 80.0, "1. easyofd-rust 项目主页")
            .size(14.0)
            .color(0x00_66CC),
    );
    page.add_text(
        TextObject::new(20.0, 100.0, "2. GB/T 33190-2016 标准文档")
            .size(14.0)
            .color(0x00_66CC),
    );
    page.add_text(
        TextObject::new(20.0, 120.0, "3. Rust 官方网站")
            .size(14.0)
            .color(0x00_66CC),
    );

    page.add_path(PathObject::hline(20.0, 140.0, 190.0));
    page.add_text(TextObject::new(
        20.0,
        155.0,
        "上述链接对应 GB/T 33190 第 15 章定义的 URI 动作类型。",
    ));

    // ── Step 2: 构造 URI 动作 ────────────────────────────────────────────
    // URI 动作是 GB/T 33190 中最常用的动作类型之一，
    // 用于在文档被打开或用户交互时打开外部资源。

    let uri_github = URI::new("https://github.com/user/easyofd-rust");
    let uri_standard = URI::new("https://openstd.samr.gov.cn/");
    let uri_rust = URI::new("https://www.rust-lang.org");

    // 构造 CTAction（事件触发器）：PO_DocumentOpen 表示文档打开时触发。
    let mut doc_open_action = CTAction::new(EventType::PO_DocumentOpen);
    doc_open_action.add_action(Box::new(uri_github));

    // 构造按钮点击事件的动作。
    let mut button_click_action = CTAction::new(EventType::PO_ButtonClick);
    button_click_action.add_action(Box::new(uri_standard));
    button_click_action.add_action(Box::new(uri_rust));

    // 序列化为 XML 用于演示。
    println!("=== URI 动作 XML 示例 ===\n");
    println!("文档打开动作：\n{}\n", doc_open_action.to_xml_string());
    println!("按钮点击动作：\n{}\n", button_click_action.to_xml_string());

    // ── Step 3: 构造注释（Link 类型注释可关联 URI 动作）─────────────────
    let link_annot = Annot::new("annot_link_1", AnnotType::Link)
        .creator("easyofd-rust")
        .last_mod_date("2026-08-10T00:00:00")
        .location(20.0, 75.0, 80.0, 10.0) // 覆盖链接文本区域
        .add_appearance(Appearance::new("appearance_normal", "Normal"));

    let text_annot = Annot::new("annot_text_1", AnnotType::Text)
        .creator("easyofd-rust")
        .last_mod_date("2026-08-10T00:00:00")
        .location(150.0, 60.0, 50.0, 20.0)
        .add_appearance(Appearance::new("appearance_note", "Normal"));

    // 构造注释容器（AnnPage 用于文档级索引）。
    let annotations = Annotations::new()
        .doc_id("action-uri-demo")
        .add_page(AnnPage::new(0).annot_file("Doc_0/Annots/Annotations_0.xml"));

    println!("=== 注释容器 XML ===\n");
    println!("{}\n", annotations.to_xml_string());

    // 单独输出注释对象的 XML。
    println!("=== 注释对象 XML ===\n");
    println!("链接注释：\n{}\n", link_annot.to_xml_string());
    println!("文本注释：\n{}\n", text_annot.to_xml_string());

    // ── Step 4: 写入 OFD 并验证 ─────────────────────────────────────────
    EasyOfd::write_pages(ofd_path.to_string_lossy().into_owned())
        .metadata_title("URI 动作示例")
        .metadata_author("easyofd-rust")
        .metadata_creator("action_uri example")
        .do_write(vec![page])?;

    println!("已生成 OFD 文件: {}", ofd_path.display());
    println!("文件大小: {} bytes", std::fs::metadata(&ofd_path)?.len());

    // 读取并验证内容。
    println!("\n=== 读取验证 ===");
    let reader = EasyOfd::read(&ofd_path)?;
    println!("页数: {}", reader.page_count());
    println!("全文:\n{}", reader.extract_all_text());

    // 清理。
    let _ = std::fs::remove_dir_all(dir);
    println!("\n所有步骤完成。");
    Ok(())
}
