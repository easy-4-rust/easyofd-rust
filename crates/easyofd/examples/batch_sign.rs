//! 批量签章示例（多文档）。
//!
//! 本示例演示如何对多个 OFD 文档进行批量数字签名，使用 GB/T 38540 标准。
//!
//! 流程：
//!   1. 创建多个 OFD 文档
//!   2. 对每个文档进行 SM2WithSM3 签名
//!   3. 验证每个签名的有效性
//!   4. 展示多签章（单文档多签名）模式
//!
//! 用法：
//!   cargo run --release --example batch_sign

use easyofd::{
    EasyOfd, ElectronicSeal, OfdPage, OfdSignatureBuilder, TextObject, verify_signature,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir = std::env::temp_dir().join("easyofd_example_batch_sign");
    std::fs::create_dir_all(&dir)?;

    // ── Step 1: 批量创建 OFD 文档 ───────────────────────────────────────
    let document_count = 3;
    let mut doc_paths = Vec::new();

    for i in 1..=document_count {
        let path = dir.join(format!("document_{i}.ofd"));
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_text(
            TextObject::new(20.0, 30.0, format!("文档 {i} - 批量签章测试"))
                .size(20.0)
                .bold(),
        );
        page.add_text(TextObject::new(
            20.0,
            60.0,
            "本文档将使用 SM2WithSM3 算法进行数字签名。",
        ));
        page.add_text(TextObject::new(
            20.0,
            80.0,
            format!("文档编号: DOC-2026-{i:04}"),
        ));
        page.add_text(TextObject::new(20.0, 100.0, "签名时间: 2026-08-10"));

        EasyOfd::write_pages(path.to_string_lossy().into_owned())
            .metadata_title(format!("批量签章文档 {i}"))
            .metadata_author("easyofd-rust")
            .metadata_creator("batch_sign example")
            .do_write(vec![page])?;

        doc_paths.push(path);
    }

    println!("=== 已创建 {document_count} 个 OFD 文档 ===");
    for path in &doc_paths {
        println!(
            "  - {} ({} bytes)",
            path.display(),
            std::fs::metadata(path)?.len()
        );
    }

    // ── Step 2: 批量签名 ─────────────────────────────────────────────────
    println!("\n=== 批量签名 ===");

    let seal_data = vec![0x89, 0x50, 0x4E, 0x47]; // PNG 占位数据
    let mut signed_paths = Vec::new();

    for (i, doc_path) in doc_paths.iter().enumerate() {
        let signed_path = dir.join(format!("signed_{}.ofd", i + 1));

        let seal = ElectronicSeal {
            image_data: seal_data.clone(),
            name: format!("公司印章_{}", i + 1),
            position: (100.0, 200.0),
            page: 1,
        };

        let signed = OfdSignatureBuilder::new(doc_path.to_string_lossy().into_owned())
            .seal(seal)
            .sign()?;

        let digest_preview = signed.digest[..16.min(signed.digest.len())].to_string();
        signed.save(&signed_path)?;

        println!("  文档 {}: 签名完成, digest={digest_preview}...", i + 1);
        signed_paths.push(signed_path);
    }

    // ── Step 3: 批量验证 ─────────────────────────────────────────────────
    println!("\n=== 批量验证签名 ===");

    for (i, signed_path) in signed_paths.iter().enumerate() {
        match verify_signature(signed_path) {
            Ok(valid) => println!("  文档 {}: 签名有效 = {valid}", i + 1),
            Err(e) => println!("  文档 {}: 验证错误 = {e}", i + 1),
        }
    }

    // ── Step 4: 多签章模式（单文档多签名）────────────────────────────────
    println!("\n=== 多签章模式（单文档多个独立签名）===");

    let multi_doc_path = dir.join("multi_sign.ofd");
    let mut page = OfdPage::new(210.0, 297.0);
    page.add_text(TextObject::new(20.0, 30.0, "多签章文档").size(20.0).bold());
    page.add_text(TextObject::new(
        20.0,
        60.0,
        "本文档将由多个签名人独立签名。",
    ));

    EasyOfd::write_pages(multi_doc_path.to_string_lossy().into_owned())
        .metadata_title("多签章文档")
        .metadata_author("easyofd-rust")
        .do_write(vec![page])?;

    // 使用 sign_multiple 进行多签章。
    use sm2::elliptic_curve::Generate;

    let alpha_key = sm2::SecretKey::generate();
    let beta_key = sm2::SecretKey::generate();

    let signed_multi = OfdSignatureBuilder::new(multi_doc_path.to_string_lossy().into_owned())
        .add_signature(
            alpha_key,
            vec![ElectronicSeal {
                image_data: vec![0x89, 0x50],
                name: "签名人A印章".to_string(),
                position: (80.0, 200.0),
                page: 0,
            }],
        )
        .add_signature(
            beta_key,
            vec![ElectronicSeal {
                image_data: vec![0x89, 0x50],
                name: "签名人B印章".to_string(),
                position: (140.0, 200.0),
                page: 0,
            }],
        )
        .sign_multiple()?;

    let multi_signed_path = dir.join("multi_signed.ofd");
    signed_multi.save(&multi_signed_path)?;

    println!(
        "  多签章完成: {} ({} bytes)",
        multi_signed_path.display(),
        std::fs::metadata(&multi_signed_path)?.len()
    );

    // 验证多签章文档。
    match verify_signature(&multi_signed_path) {
        Ok(valid) => println!("  多签章验证: 签名有效 = {valid}"),
        Err(e) => println!("  多签章验证: {e}"),
    }

    // 清理。
    let _ = std::fs::remove_dir_all(dir);
    println!("\n所有步骤完成。");
    Ok(())
}
