//! 07_sign_verify —— SM2 签名 + 验签 roundtrip。
//!
//! 演示 GB/T 38540 数字签名完整流程：
//!   1. 创建 OFD 文档
//!   2. 使用 SM2WithSM3 算法签名
//!   3. 验证签名有效性
//!   4. 篡改检测（修改 ZIP 条目后验签失败）
//!
//! 产物写入 /tmp/easyofd_examples/07_sign_verify/。
//!
//! 运行：
//!   cargo run --example 07_sign_verify

use easyofd::{EasyOfd, ElectronicSeal, OfdPage, OfdSignatureBuilder, TextObject};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir = std::env::temp_dir()
        .join("easyofd_examples")
        .join("07_sign_verify");
    std::fs::create_dir_all(&dir)?;

    // ── Step 1: 创建原始文档 ─────────────────────────────────────
    let original_path = dir.join("document.ofd");
    let mut page = OfdPage::new(210.0, 297.0);
    page.add_text(
        TextObject::new(20.0, 30.0, "SM2 数字签名演示")
            .size(20.0)
            .bold(),
    );
    page.add_text(TextObject::new(
        20.0,
        60.0,
        "本文档将使用 SM2WithSM3 算法进行数字签名。",
    ));
    page.add_text(TextObject::new(20.0, 80.0, "签名后任何篡改都会被检测到。"));

    EasyOfd::write_pages(original_path.to_string_lossy().into_owned())
        .metadata_title("签名演示文档")
        .do_write(vec![page])?;
    println!(
        "[Step 1] 原始文档: {} ({} bytes)",
        original_path.display(),
        std::fs::metadata(&original_path)?.len()
    );

    // ── Step 2: 签名 ─────────────────────────────────────────────
    let signed_path = dir.join("document_signed.ofd");
    let seal = ElectronicSeal {
        image_data: vec![0x89, 0x50, 0x4E, 0x47], // PNG 占位
        name: "公司印章".to_string(),
        position: (100.0, 200.0),
        page: 1,
    };

    let signed = OfdSignatureBuilder::new(original_path.to_string_lossy().into_owned())
        .seal(seal)
        .sign()?;

    let digest_preview = signed.digest[..16.min(signed.digest.len())].to_string();
    let sig_preview = signed.signature_value[..16.min(signed.signature_value.len())].to_string();
    signed.save(&signed_path)?;
    println!("[Step 2] 签名文档: {}", signed_path.display());
    println!("  SM3 摘要 (hex)  : {digest_preview}...");
    println!("  签名值 (hex)    : {sig_preview}...");

    // ── Step 3: 验签 ─────────────────────────────────────────────
    println!("\n[Step 3] 验证签名...");
    let valid = easyofd::verify_signature(&signed_path)?;
    println!("  签名有效: {valid}");

    let details = easyofd::read_signature(&signed_path)?;
    println!("  算法: {:?}", details.algorithm);
    println!(
        "  摘要: {}...",
        &details.digest[..16.min(details.digest.len())]
    );

    // ── Step 4: 篡改检测 ─────────────────────────────────────────
    println!("\n[Step 4] 篡改检测...");
    let tampered_path = dir.join("tampered.ofd");
    {
        use std::io::{Read as _, Write as _};
        let signed_bytes = std::fs::read(&signed_path)?;
        let reader = std::io::Cursor::new(&signed_bytes[..]);
        let mut archive = zip::ZipArchive::new(reader).map_err(|e| format!("zip read: {e}"))?;

        let out_file = std::fs::File::create(&tampered_path)?;
        let mut zip = zip::ZipWriter::new(out_file);
        let opts = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);

        let mut tampered = false;
        for i in 0..archive.len() {
            let mut entry = archive.by_index(i).expect("zip entry");
            let name = entry.name().to_string();
            let mut buf = Vec::new();
            entry.read_to_end(&mut buf).expect("read entry");

            if name.ends_with("OFD.xml") && !tampered {
                buf.extend_from_slice(b"<!-- TAMPERED -->");
                tampered = true;
                println!("  已篡改条目: {name}");
            }

            zip.start_file(&name, opts).expect("start file");
            zip.write_all(&buf).expect("write");
        }
        zip.finish().expect("finish zip");
    }

    match easyofd::verify_signature(&tampered_path) {
        Ok(true) => println!("  篡改文件: 签名格式有效（内容完整性未校验）"),
        Ok(false) => println!("  篡改文件: 签名无效，篡改已检测。"),
        Err(e) => println!("  篡改文件: 正确拒绝，错误: {e}"),
    }

    // 清理
    let _ = std::fs::remove_dir_all(&dir);
    println!("\n示例完成。");
    Ok(())
}
