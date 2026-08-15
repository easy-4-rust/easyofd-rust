//! 05_template_fill —— 模板占位符填充。
//!
//! 演示如何创建带 `{placeholder}` 占位符的 OFD 模板，
//! 然后用 `EasyOfd::fill_template` 批量填充生成正式文档。
//! 产物写入 /tmp/easyofd_examples/05_template_fill_*.ofd。
//!
//! 运行：
//!   cargo run --example 05_template_fill

use std::collections::HashMap;

use easyofd::{EasyOfd, OfdPage, TextObject};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir = std::env::temp_dir().join("easyofd_examples");
    std::fs::create_dir_all(&dir)?;

    // ── Step 1: 创建模板 OFD（含占位符）────────────────────────────
    let template_path = dir.join("05_template_fill_template.ofd");
    let mut page = OfdPage::new(210.0, 297.0);
    page.add_text(TextObject::new(20.0, 25.0, "劳 动 合 同").size(22.0).bold());
    page.add_text(TextObject::new(20.0, 55.0, "甲方（用人单位）: {company}"));
    page.add_text(TextObject::new(20.0, 75.0, "乙方（劳动者）  : {employee}"));
    page.add_text(TextObject::new(
        20.0,
        95.0,
        "合同期限          : {start_date} 至 {end_date}",
    ));
    page.add_text(TextObject::new(20.0, 115.0, "月薪资            : {salary}"));
    page.add_text(TextObject::new(20.0, 145.0, "备注: {notes}"));

    EasyOfd::write_pages(template_path.to_string_lossy().into_owned()).do_write(vec![page])?;
    println!("[模板] 已创建: {}", template_path.display());

    // ── Step 2: 批量填充 ─────────────────────────────────────────
    let contracts = vec![
        (
            "Alice",
            hashmap(vec![
                ("company".into(), "北京科技有限公司".into()),
                ("employee".into(), "张三".into()),
                ("start_date".into(), "2026-01-01".into()),
                ("end_date".into(), "2028-12-31".into()),
                ("salary".into(), "¥15,000".into()),
                ("notes".into(), "试用期三个月".into()),
            ]),
        ),
        (
            "Bob",
            hashmap(vec![
                ("company".into(), "上海智能科技有限公司".into()),
                ("employee".into(), "李四".into()),
                ("start_date".into(), "2026-03-15".into()),
                ("end_date".into(), "2029-03-14".into()),
                ("salary".into(), "¥20,000".into()),
                ("notes".into(), "含五险一金".into()),
            ]),
        ),
    ];

    for (label, data) in &contracts {
        let output_path = dir.join(format!("05_template_fill_{label}.ofd"));
        EasyOfd::fill_template(&template_path, data)?.save(&output_path)?;

        println!("\n[填充] {label}: {}", output_path.display());
        let reader = EasyOfd::read(&output_path)?;
        println!("[填充] 内容:\n{}", reader.extract_all_text());
        let _ = std::fs::remove_file(&output_path);
    }

    // 清理
    let _ = std::fs::remove_file(&template_path);
    println!("\n示例完成。");
    Ok(())
}

/// 辅助函数：快速创建 HashMap。
fn hashmap(pairs: Vec<(String, String)>) -> HashMap<String, String> {
    pairs.into_iter().collect()
}
