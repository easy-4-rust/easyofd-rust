//! SM4 对称加密/解密实现（SM4-CBC 模式，PKCS#7 填充）。
//!
//! 对应 Java: `org.ofdrw.crypto.encryt.SM4Util`
//!
//! SM4 是中国国家密码标准（GB/T 32907-2016），分组长度 128 位，密钥长度 128 位。
//! 本模块使用 CBC（Cipher Block Chaining）模式配合 PKCS#7 填充，适用于 OFD 文档
//! 中 ZIP 条目的逐条加密。
//!
//! 密文格式：`[16 字节 IV || 密文数据]`，IV 在加密时随机生成并前置于密文。

use cbc::cipher::block_padding::Pkcs7;
use cbc::cipher::{BlockModeDecrypt, BlockModeEncrypt, KeyIvInit};
use easyofd_core::{OfdError, OfdResult};
use rand::RngCore;

/// SM4 分组长度（字节）。
pub const BLOCK_SIZE: usize = 16;

/// SM4 密钥长度（字节）。
pub const KEY_SIZE: usize = 16;

/// SM4-CBC 加密器类型别名。
type Sm4CbcEnc = cbc::Encryptor<sm4::Sm4>;

/// SM4-CBC 解密器类型别名。
type Sm4CbcDec = cbc::Decryptor<sm4::Sm4>;

/// 生成 16 字节随机 IV。
#[must_use]
pub fn generate_iv() -> [u8; BLOCK_SIZE] {
    let mut iv = [0u8; BLOCK_SIZE];
    rand::thread_rng().fill_bytes(&mut iv);
    iv
}

/// SM4-CBC 加密（PKCS#7 填充）。
///
/// 返回格式：`[16 字节 IV || 密文]`。
///
/// 对应 Java: `SM4Util.encrypt(String key, byte[] data)` 及
/// `SM4Util.encrypt(String key, String iv, byte[] data)`
///
/// # 错误
///
/// - 输入为空时返回 [`OfdError::InvalidDocument`]。
/// - 内部加密失败时返回 [`OfdError::Conversion`]。
pub fn encrypt(key: &[u8; KEY_SIZE], plaintext: &[u8]) -> OfdResult<Vec<u8>> {
    if plaintext.is_empty() {
        return Err(OfdError::InvalidDocument("SM4 加密：明文不能为空".into()));
    }

    let iv = generate_iv();
    let encryptor = Sm4CbcEnc::new(key.into(), &iv.into());

    // 额外空间用于 PKCS#7 填充（最多 BLOCK_SIZE 字节）
    let mut buf = vec![0u8; plaintext.len() + BLOCK_SIZE];
    buf[..plaintext.len()].copy_from_slice(plaintext);

    let ct = encryptor
        .encrypt_padded::<Pkcs7>(&mut buf, plaintext.len())
        .map_err(|e| OfdError::Conversion(format!("SM4 加密失败: {e}")))?;

    // 输出 = IV + 密文
    let mut output = Vec::with_capacity(BLOCK_SIZE + ct.len());
    output.extend_from_slice(&iv);
    output.extend_from_slice(ct);
    Ok(output)
}

/// SM4-CBC 加密（使用指定 IV）。
///
/// 用于需要指定 IV 的场景（如测试或文档重加密）。
///
/// 返回格式：`[16 字节 IV || 密文]`。
///
/// # 错误
///
/// - 输入为空时返回 [`OfdError::InvalidDocument`]。
/// - 内部加密失败时返回 [`OfdError::Conversion`]。
pub fn encrypt_with_iv(
    key: &[u8; KEY_SIZE],
    iv: &[u8; BLOCK_SIZE],
    plaintext: &[u8],
) -> OfdResult<Vec<u8>> {
    if plaintext.is_empty() {
        return Err(OfdError::InvalidDocument("SM4 加密：明文不能为空".into()));
    }

    let encryptor = Sm4CbcEnc::new(key.into(), iv.into());
    let mut buf = vec![0u8; plaintext.len() + BLOCK_SIZE];
    buf[..plaintext.len()].copy_from_slice(plaintext);

    let ct = encryptor
        .encrypt_padded::<Pkcs7>(&mut buf, plaintext.len())
        .map_err(|e| OfdError::Conversion(format!("SM4 加密失败: {e}")))?;

    let mut output = Vec::with_capacity(BLOCK_SIZE + ct.len());
    output.extend_from_slice(iv);
    output.extend_from_slice(ct);
    Ok(output)
}

/// SM4-CBC 解密（PKCS#7 去填充）。
///
/// 输入格式：`[16 字节 IV || 密文]`。
///
/// 对应 Java: `SM4Util.decrypt(String key, byte[] data)` 及
/// `SM4Util.decrypt(String key, String iv, byte[] data)`
///
/// # 错误
///
/// - 输入长度不足 16 字节时返回 [`OfdError::InvalidDocument`]。
/// - 填充无效或解密失败时返回 [`OfdError::Conversion`]。
pub fn decrypt(key: &[u8; KEY_SIZE], ciphertext: &[u8]) -> OfdResult<Vec<u8>> {
    if ciphertext.len() < BLOCK_SIZE {
        return Err(OfdError::InvalidDocument(
            "SM4 解密：密文长度不足（至少需要 16 字节 IV）".into(),
        ));
    }

    let (iv, ct) = ciphertext.split_at(BLOCK_SIZE);
    decrypt_with_iv(key, iv.try_into().expect("iv is BLOCK_SIZE bytes"), ct)
}

/// SM4-CBC 解密（使用指定 IV）。
///
/// 用于已知 IV 的场景。
///
/// # 错误
///
/// - 输入为空时返回 [`OfdError::InvalidDocument`]。
/// - 填充无效或解密失败时返回 [`OfdError::Conversion`]。
pub fn decrypt_with_iv(
    key: &[u8; KEY_SIZE],
    iv: &[u8; BLOCK_SIZE],
    ciphertext: &[u8],
) -> OfdResult<Vec<u8>> {
    if ciphertext.is_empty() {
        return Err(OfdError::InvalidDocument("SM4 解密：密文不能为空".into()));
    }

    let decryptor = Sm4CbcDec::new(key.into(), iv.into());
    let mut buf = ciphertext.to_vec();

    let pt = decryptor
        .decrypt_padded::<Pkcs7>(&mut buf)
        .map_err(|e| OfdError::Conversion(format!("SM4 解密失败（填充无效）: {e}")))?;

    Ok(pt.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_KEY: [u8; KEY_SIZE] = [
        0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF, 0xFE, 0xDC, 0xBA, 0x98, 0x76, 0x54, 0x32,
        0x10,
    ];

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let plaintext = b"Hello, OFD encryption with SM4-CBC!";
        let ct = encrypt(&TEST_KEY, plaintext).unwrap();
        let pt = decrypt(&TEST_KEY, &ct).unwrap();
        assert_eq!(pt, plaintext);
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip_with_iv() {
        let iv = [0xABu8; BLOCK_SIZE];
        let plaintext = b"OFD document content for SM4-CBC testing.";
        let ct = encrypt_with_iv(&TEST_KEY, &iv, plaintext).unwrap();
        // IV is prepended; extract it from ct
        let pt = decrypt(&TEST_KEY, &ct).unwrap();
        assert_eq!(pt, plaintext);
    }

    #[test]
    fn test_decrypt_wrong_key() {
        let plaintext = b"secret data for wrong key test";
        let ct = encrypt(&TEST_KEY, plaintext).unwrap();

        let wrong_key = [0xFFu8; KEY_SIZE];
        let result = decrypt(&wrong_key, &ct);
        assert!(result.is_err());
    }

    #[test]
    fn test_encrypt_empty_input() {
        let result = encrypt(&TEST_KEY, b"");
        assert!(result.is_err());
        assert!(matches!(result, Err(OfdError::InvalidDocument(_))));
    }

    #[test]
    fn test_decrypt_too_short() {
        let result = decrypt(&TEST_KEY, &[0u8; 8]);
        assert!(result.is_err());
        assert!(matches!(result, Err(OfdError::InvalidDocument(_))));
    }

    #[test]
    fn test_decrypt_empty_after_iv() {
        // Valid IV but empty ciphertext
        let short_data = [0u8; BLOCK_SIZE]; // just an IV with no ciphertext
        let result = decrypt(&TEST_KEY, &short_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_decrypt_with_iv_empty_input() {
        let iv = [0u8; BLOCK_SIZE];
        let result = decrypt_with_iv(&TEST_KEY, &iv, &[]);
        assert!(result.is_err());
        assert!(matches!(result, Err(OfdError::InvalidDocument(_))));
    }

    #[test]
    fn test_encrypt_decrypt_long_data() {
        // Data longer than one block
        let plaintext = vec![0x42u8; 256];
        let ct = encrypt(&TEST_KEY, &plaintext).unwrap();
        let pt = decrypt(&TEST_KEY, &ct).unwrap();
        assert_eq!(pt, plaintext);
    }

    #[test]
    fn test_ciphertext_includes_iv() {
        let plaintext = b"test iv prefix";
        let ct = encrypt(&TEST_KEY, plaintext).unwrap();
        // 密文 = IV(16) + 至少一个加密块
        assert!(ct.len() >= BLOCK_SIZE + BLOCK_SIZE);
    }

    #[test]
    fn test_different_ivs_produce_different_ciphertext() {
        let plaintext = b"same plaintext, different IVs";
        let ct1 = encrypt(&TEST_KEY, plaintext).unwrap();
        let ct2 = encrypt(&TEST_KEY, plaintext).unwrap();
        // 两次加密使用不同随机 IV，密文应不同
        assert_ne!(ct1, ct2);
        // 但都能正确解密
        assert_eq!(decrypt(&TEST_KEY, &ct1).unwrap(), plaintext);
        assert_eq!(decrypt(&TEST_KEY, &ct2).unwrap(), plaintext);
    }

    #[test]
    fn test_encrypt_decrypt_single_byte() {
        let plaintext = [0x42u8]; // 1 字节，需要 15 字节填充
        let ct = encrypt(&TEST_KEY, &plaintext).unwrap();
        let pt = decrypt(&TEST_KEY, &ct).unwrap();
        assert_eq!(pt, plaintext);
    }

    #[test]
    fn test_encrypt_decrypt_exact_block_size() {
        // 恰好 16 字节 = 一个完整分组，PKCS#7 会追加一整块填充
        let plaintext = [0xAAu8; BLOCK_SIZE];
        let ct = encrypt(&TEST_KEY, &plaintext).unwrap();
        // 密文 = IV(16) + 2 blocks(32) = 48 字节
        assert_eq!(ct.len(), BLOCK_SIZE + 2 * BLOCK_SIZE);
        let pt = decrypt(&TEST_KEY, &ct).unwrap();
        assert_eq!(pt, plaintext);
    }

    #[test]
    fn test_generate_iv_randomness() {
        let iv1 = generate_iv();
        let iv2 = generate_iv();
        // 两次生成的 IV 应不同（概率上几乎必然）
        assert_ne!(iv1, iv2);
    }
}
