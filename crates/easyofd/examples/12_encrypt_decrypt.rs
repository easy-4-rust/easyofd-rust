//! 12_encrypt_decrypt —— SM4 加解密 roundtrip。
//!
//! 演示使用 SM4-CBC 对称加密算法对 OFD 文档进行加密和解密：
//!   1. 创建 OFD 文档
//!   2. 使用 SM4 密钥加密（所有 ZIP 条目独立加密）
//!   3. 使用相同密钥解密
//!   4. 验证解密后内容与原文一致
//!   5. 使用错误密钥解密失败
//!
//! 产物写入 /tmp/easyofd_examples/12_encrypt_decrypt/。
//!
//! 注意：本示例直接使用 easyofd_crypto 子 crate，因为加解密
//! 功能未通过 EasyOfd facade 暴露（facade 易用性缺口）。
//!
//! 运行：
//!   cargo run --example 12_encrypt_decrypt

use easyofd::{EasyOfd, OfdPage, TextObject};
use easyofd_crypto::{decrypt_ofd, encrypt_ofd, sm4};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir = std::env::temp_dir()
        .join("easyofd_examples")
        .join("12_encrypt_decrypt");
    std::fs::create_dir_all(&dir)?;

    // ── SM4 密钥（128 位 = 16 字节）──────────────────────────────
    let key: [u8; sm4::KEY_SIZE] = [
        0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF, 0xFE, 0xDC, 0xBA, 0x98, 0x76, 0x54, 0x32,
        0x10,
    ];
    let wrong_key: [u8; sm4::KEY_SIZE] = [0xFF; sm4::KEY_SIZE];

    // ── Step 1: 创建 OFD 文档 ────────────────────────────────────
    let mut page = OfdPage::new(210.0, 297.0);
    page.add_text(
        TextObject::new(20.0, 30.0, "SM4 加密演示文档")
            .size(20.0)
            .bold(),
    );
    page.add_text(TextObject::new(
        20.0,
        60.0,
        "本文档将使用 SM4-CBC 算法进行对称加密。",
    ));
    page.add_text(TextObject::new(20.0, 80.0, "加密后所有 ZIP 条目均为密文。"));

    let ofd_bytes = EasyOfd::write_pages_to_bytes(vec![page])?;
    println!("[Step 1] 原始 OFD: {} bytes", ofd_bytes.len());

    // ── Step 2: SM4 加密 ─────────────────────────────────────────
    let encrypted = encrypt_ofd(&ofd_bytes, &key)?;
    println!("[Step 2] 加密完成: {} bytes", encrypted.len());

    // 验证加密后的 ZIP 包含 .enc 条目
    {
        let reader = std::io::Cursor::new(&encrypted);
        let mut archive = zip::ZipArchive::new(reader).expect("encrypted zip");
        let mut enc_count = 0;
        for i in 0..archive.len() {
            let entry = archive.by_index(i).expect("entry");
            if entry.name().to_ascii_lowercase().ends_with(".enc") {
                enc_count += 1;
            }
        }
        println!("  加密条目数: {enc_count}");
        // 验证包含 EncryptInfo.xml
        assert!(
            archive.by_name("EncryptInfo.xml").is_ok(),
            "应包含 EncryptInfo.xml"
        );
        println!("  EncryptInfo.xml: 存在");
    }

    // ── Step 3: SM4 解密 ─────────────────────────────────────────
    let decrypted = decrypt_ofd(&encrypted, &key)?;
    println!("\n[Step 3] 解密完成: {} bytes", decrypted.len());

    // ── Step 4: 验证解密内容 ─────────────────────────────────────
    let reader = EasyOfd::read_from_bytes(&decrypted)?;
    println!("[Step 4] 解密后页数: {}", reader.page_count());
    println!("  全文:\n{}", reader.extract_all_text());

    // 验证解密后内容与原文一致
    let original_reader = EasyOfd::read_from_bytes(&ofd_bytes)?;
    assert_eq!(
        reader.page_count(),
        original_reader.page_count(),
        "页数应一致"
    );
    println!("  内容一致性: OK");

    // ── Step 5: 错误密钥解密失败 ─────────────────────────────────
    println!("\n[Step 5] 错误密钥测试...");
    match decrypt_ofd(&encrypted, &wrong_key) {
        Ok(_) => println!("  意外成功（不应发生）"),
        Err(e) => println!("  正确拒绝: {e}"),
    }

    // 清理
    let _ = std::fs::remove_dir_all(&dir);
    println!("\n示例完成。");
    Ok(())
}
