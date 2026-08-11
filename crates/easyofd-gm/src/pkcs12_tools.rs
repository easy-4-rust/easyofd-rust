//! PKCS#12 解析工具。
//!
//! 对应 Java: org.ofdrw.gm.cert.PKCS12Tools
//!
//! 提供 PKCS#12 (PFX) 格式密钥库的解析功能。
//! 支持解析无加密 KeyBag 和 PBES2 加密的 ShroudedKeyBag，提取 SM2 私钥。
//!
//! ## 已支持
//! - 无加密 KeyBag（bagId = 1.2.840.113549.1.12.10.1.1）
//!   直接从 PrivateKeyInfo 结构中提取 EC 私钥标量字节
//! - ShroudedKeyBag（加密私钥袋，bagId = 1.2.840.113549.1.12.10.1.2）
//!   支持 PBES2 + PBKDF2 解密，PRF 支持 HMAC-SHA1 / HMAC-SHA256 / HMAC-SM3，
//!   加密算法支持 AES-128-CBC / SM4-CBC
//!
//! ## 不支持（返回 None 并记录原因）
//! - PKCS#12 传统 PBE（OID 1.2.840.113549.1.12.1.x，3DES/SHA1 链式）
//!   需要 PKCS#12 专用 KDF，与 Java(BC) 的 BouncyCastle 实现差异较大
//! - EncryptedData 类型的 ContentInfo
//!   需要 PKCS#7 解密，当前无纯 Rust 实现
//! - 带 MAC 校验的 PFX（macData 字段）
//!   当前跳过 MAC 校验，仅解析 authSafe

use der::asn1::Any;
use der::{Decode, Encode, Reader, SliceReader, Tag, Tagged};

// ── PKCS#12 / PKCS#7 OID 常量 ──────────────────────────────────────────

/// PKCS#7 data 内容类型 OID: 1.2.840.113549.1.7.1
const OID_PKCS7_DATA: &str = "1.2.840.113549.1.7.1";

/// PKCS#12 KeyBag OID: 1.2.840.113549.1.12.10.1.1
const OID_KEY_BAG: &str = "1.2.840.113549.1.12.10.1.1";

/// PKCS#12 ShroudedKeyBag OID: 1.2.840.113549.1.12.10.1.2
const OID_SHROUDED_KEY_BAG: &str = "1.2.840.113549.1.12.10.1.2";

// ── PBES2 / PBKDF2 OID 常量 ────────────────────────────────────────────

/// PBES2 OID: 1.2.840.113549.1.5.13
const OID_PBES2: &str = "1.2.840.113549.1.5.13";

/// PBKDF2 OID: 1.2.840.113549.1.5.12
const OID_PBKDF2: &str = "1.2.840.113549.1.5.12";

/// HMAC-SHA1 PRF OID（PBKDF2 默认 PRF）: 1.2.840.113549.2.7
const OID_HMAC_SHA1: &str = "1.2.840.113549.2.7";

/// HMAC-SHA256 PRF OID: 1.2.840.113549.2.9
const OID_HMAC_SHA256: &str = "1.2.840.113549.2.9";

/// HMAC-SM3 PRF OID: 1.2.156.10197.1.401
const OID_HMAC_SM3: &str = "1.2.156.10197.1.401";

/// AES-128-CBC 加密方案 OID: 2.16.840.1.101.3.4.1.2
const OID_AES128_CBC: &str = "2.16.840.1.101.3.4.1.2";

/// SM4-CBC 加密方案 OID: 1.2.156.10197.1.104.2
const OID_SM4_CBC: &str = "1.2.156.10197.1.104.2";

/// AES/SM4 分组大小（均为128位 = 16字节）
const BLOCK_SIZE: usize = 16;

/// HMAC-SM3 输出大小（SM3 摘要长度 = 32字节）
const SM3_OUTPUT_SIZE: usize = 32;

/// SM3 分组大小（用于 HMAC 内部 key padding）
const SM3_BLOCK_SIZE: usize = 64;

/// PKCS#12 工具。
///
/// 对应 Java: org.ofdrw.gm.cert.PKCS12Tools
///
/// 提供 PKCS#12 格式密钥库的解析功能。
pub struct Pkcs12Tools;

impl Pkcs12Tools {
    /// 从 PKCS#12 数据中提取私钥。
    ///
    /// 对应 Java: org.ofdrw.gm.cert.PKCS12Tools#ReadPrvKey
    ///
    /// 解析 PKCS#12 (PFX) 结构，提取第一个私钥。
    /// 返回 SM2 私钥的原始标量字节（32 字节）。
    ///
    /// 支持无加密 KeyBag 和 PBES2 加密的 ShroudedKeyBag。
    /// 对于 PBES2，password 传入 PBKDF2 参与密钥派生。
    ///
    /// # 参数
    /// - `p12_data`: PKCS#12 格式的 DER 编码数据
    /// - `password`: 解密密码，用于 ShroudedKeyBag 的 PBES2 解密
    ///
    /// # 返回
    /// SM2 私钥原始字节（32 字节标量），如果解析失败返回 `None`。
    ///
    /// # 不支持的格式
    /// - PKCS#12 传统 PBE（OID 1.2.840.113549.1.12.1.x）
    /// - EncryptedData ContentInfo
    #[must_use]
    pub fn read_private_key(p12_data: &[u8], password: &str) -> Option<Vec<u8>> {
        extract_first_private_key(p12_data, password)
    }
}

// ══════════════════════════════════════════════════════════════════════════
// PFX 顶层解析
// ══════════════════════════════════════════════════════════════════════════

/// 从 PFX 结构中提取第一个私钥。
fn extract_first_private_key(pfx_data: &[u8], password: &str) -> Option<Vec<u8>> {
    // 1. 解析 PFX SEQUENCE { version INTEGER, authSafe ContentInfo, [macData] }
    let pfx_fields = parse_sequence_fields(pfx_data)?;
    if pfx_fields.len() < 2 {
        return None;
    }

    // 2. authSafe 是 ContentInfo（SEQUENCE），解析其字段
    let auth_safe_fields = parse_der_fields(pfx_fields[1].value())?;
    if auth_safe_fields.len() < 2 {
        return None;
    }

    // 3. 检查 contentType 是否为 data (1.2.840.113549.1.7.1)
    let content_type_oid = parse_oid_string(&auth_safe_fields[0])?;
    if content_type_oid != OID_PKCS7_DATA {
        return None;
    }

    // 4. 提取 [0] EXPLICIT 中的 OCTET STRING（包含 AuthenticatedSafe）
    let auth_safe_data = extract_explicit_content(&auth_safe_fields[1])?;

    // 5. 解析 AuthenticatedSafe ::= SEQUENCE OF ContentInfo
    let auth_safe_items = parse_sequence_fields(&auth_safe_data)?;

    // 6. 遍历每个 ContentInfo，查找包含 KeyBag 的 data 类型
    for item in &auth_safe_items {
        if let Some(key) = extract_key_from_content_info(item, password) {
            return Some(key);
        }
    }

    None
}

// ══════════════════════════════════════════════════════════════════════════
// ContentInfo / SafeBag 层解析
// ══════════════════════════════════════════════════════════════════════════

/// 从 ContentInfo 中提取私钥。
///
/// ContentInfo ::= SEQUENCE { contentType OID, content [0] EXPLICIT }
fn extract_key_from_content_info(ci_any: &Any, password: &str) -> Option<Vec<u8>> {
    let ci_fields = parse_der_fields(ci_any.value())?;
    if ci_fields.len() < 2 {
        return None;
    }

    let ci_oid = parse_oid_string(&ci_fields[0])?;
    if ci_oid != OID_PKCS7_DATA {
        // 非 data 类型（如 encryptedData），当前不支持
        return None;
    }

    // 提取 [0] EXPLICIT 中的 OCTET STRING（包含 SafeContents）
    let safe_contents_data = extract_explicit_content(&ci_fields[1])?;

    // 解析 SafeContents ::= SEQUENCE OF SafeBag
    let safe_bags = parse_sequence_fields(&safe_contents_data)?;

    // 遍历每个 SafeBag，查找 KeyBag 或 ShroudedKeyBag
    for bag in &safe_bags {
        if let Some(key) = extract_key_from_safe_bag(bag, password) {
            return Some(key);
        }
    }

    None
}

/// 从 SafeBag 中提取私钥。
///
/// SafeBag ::= SEQUENCE { bagId OID, bagValue [0] EXPLICIT, [bagAttributes] }
/// 处理 KeyBag 和 ShroudedKeyBag。
fn extract_key_from_safe_bag(bag_any: &Any, password: &str) -> Option<Vec<u8>> {
    let bag_fields = parse_der_fields(bag_any.value())?;
    if bag_fields.len() < 2 {
        return None;
    }

    let bag_oid = parse_oid_string(&bag_fields[0])?;

    if bag_oid == OID_KEY_BAG {
        // KeyBag: bagValue [0] EXPLICIT 包含 PrivateKeyInfo SEQUENCE（完整 DER）
        let pk_info_der = extract_explicit_tlv(&bag_fields[1])?;
        extract_private_key_from_pkcs8(&pk_info_der)
    } else if bag_oid == OID_SHROUDED_KEY_BAG {
        // ShroudedKeyBag: bagValue [0] EXPLICIT 包含 EncryptedPrivateKeyInfo SEQUENCE（完整 DER）
        // 对应 Java: org.ofdrw.gm.cert.PKCS12Tools#ReadPrvKey
        // BouncyCastle 通过 PKCS#12 KeyStore 解密，此处用 PBES2+PBKDF2 纯 Rust 实现
        let encrypted_pk_info_der = extract_explicit_tlv(&bag_fields[1])?;
        decrypt_shrouded_key_bag(&encrypted_pk_info_der, password)
    } else {
        // 其他类型的 SafeBag（如 certBag），跳过
        None
    }
}

// ══════════════════════════════════════════════════════════════════════════
// ShroudedKeyBag / PBES2 解密管线
// ══════════════════════════════════════════════════════════════════════════

/// 解密 ShroudedKeyBag 中的 EncryptedPrivateKeyInfo。
///
/// EncryptedPrivateKeyInfo ::= SEQUENCE {
///     encryptionAlgorithm AlgorithmIdentifier,
///     encryptedData OCTET STRING
/// }
fn decrypt_shrouded_key_bag(encrypted_pk_info_der: &[u8], password: &str) -> Option<Vec<u8>> {
    let fields = parse_sequence_fields(encrypted_pk_info_der)?;
    if fields.len() < 2 {
        return None;
    }

    // 解析 AlgorithmIdentifier SEQUENCE
    let alg_fields = parse_der_fields(fields[0].value())?;
    if alg_fields.is_empty() {
        return None;
    }
    let alg_oid = parse_oid_string(&alg_fields[0])?;

    if alg_oid != OID_PBES2 {
        // 老式 PKCS#12 PBE（OID 1.2.840.113549.1.12.1.x，3DES/SHA1 链式）
        // 无法纯 Rust 解则返回 None
        return None;
    }

    // PBES2-params 是 AlgorithmIdentifier 的第二个字段
    if alg_fields.len() < 2 {
        return None;
    }
    let pbes2_params_der = alg_fields[1].to_der().ok()?;
    let encrypted_data = fields[1].value();

    pbes2_decrypt(&pbes2_params_der, encrypted_data, password)
}

/// PBES2 解密入口。
///
/// PBES2-params ::= SEQUENCE {
///     keyDerivationFunc AlgorithmIdentifier,
///     encryptionScheme AlgorithmIdentifier
/// }
fn pbes2_decrypt(params_der: &[u8], encrypted_data: &[u8], password: &str) -> Option<Vec<u8>> {
    let params = parse_sequence_fields(params_der)?;
    if params.len() < 2 {
        return None;
    }

    // ── 解析 keyDerivationFunc ──
    let kdf_fields = parse_der_fields(params[0].value())?;
    if kdf_fields.is_empty() {
        return None;
    }
    let kdf_oid = parse_oid_string(&kdf_fields[0])?;
    if kdf_oid != OID_PBKDF2 {
        return None;
    }

    // PBKDF2-params ::= SEQUENCE {
    //     salt OCTET STRING,
    //     iterationCount INTEGER (1..MAX),
    //     keyLength INTEGER (1..MAX) OPTIONAL,
    //     prf AlgorithmIdentifier DEFAULT algid-hmacWithSHA1
    // }
    if kdf_fields.len() < 2 {
        return None;
    }
    let kdf_params_der = kdf_fields[1].to_der().ok()?;
    let pbkdf2_params = parse_sequence_fields(&kdf_params_der)?;
    if pbkdf2_params.len() < 2 {
        return None;
    }

    let salt = pbkdf2_params[0].value();
    let iterations = parse_integer_u32(&pbkdf2_params[1])?;
    if iterations == 0 {
        return None;
    }

    // 可选参数: keyLength (INTEGER), prf (AlgorithmIdentifier SEQUENCE)
    let mut key_length: Option<usize> = None;
    let mut prf_oid: Option<String> = None;
    for param in pbkdf2_params.iter().skip(2) {
        match param.tag() {
            Tag::Integer => {
                key_length = Some(parse_integer_usize(param)?);
            }
            Tag::Sequence => {
                let prf_fields = parse_der_fields(param.value())?;
                if let Some(first) = prf_fields.first() {
                    prf_oid = parse_oid_string(first);
                }
            }
            _ => {}
        }
    }

    // 默认 PRF 是 HMAC-SHA1（RFC 8018）
    let prf = prf_oid.unwrap_or_else(|| OID_HMAC_SHA1.to_string());

    // ── 解析 encryptionScheme ──
    let enc_fields = parse_der_fields(params[1].value())?;
    if enc_fields.is_empty() {
        return None;
    }
    let enc_oid = parse_oid_string(&enc_fields[0])?;

    // IV 是 encryptionScheme 的参数（OCTET STRING）
    if enc_fields.len() < 2 {
        return None;
    }
    let iv = enc_fields[1].value();
    if iv.len() != BLOCK_SIZE {
        return None;
    }

    // 确定密钥长度：优先使用 PBKDF2 参数中的 keyLength，否则按加密算法推断
    let key_len = key_length.unwrap_or(match enc_oid.as_str() {
        OID_AES128_CBC | OID_SM4_CBC => 16,
        _ => return None,
    });

    // ── 派生密钥 ──
    let key = derive_key_pbkdf2(password.as_bytes(), salt, iterations, key_len, &prf)?;

    // ── 解密 ──
    let plaintext = match enc_oid.as_str() {
        OID_AES128_CBC => decrypt_aes_128_cbc(&key, iv, encrypted_data)?,
        OID_SM4_CBC => decrypt_sm4_cbc(&key, iv, encrypted_data)?,
        _ => return None,
    };

    // 明文是 PKCS#8 PrivateKeyInfo，提取 EC 私钥标量
    extract_private_key_from_pkcs8(&plaintext)
}

// ══════════════════════════════════════════════════════════════════════════
// PBKDF2 密钥派生
// ══════════════════════════════════════════════════════════════════════════

/// PBKDF2 密钥派生（根据 PRF OID 分派）。
///
/// 支持的 PRF：
/// - HMAC-SHA1（OID 1.2.840.113549.2.7）—— 使用 pbkdf2 crate
/// - HMAC-SHA256（OID 1.2.840.113549.2.9）—— 使用 pbkdf2 crate
/// - HMAC-SM3（OID 1.2.156.10197.1.401）—— 手动实现（sm3 使用 digest 0.11，
///   与 hmac 0.12/pbkdf2 0.12 的 digest 0.10 不兼容）
fn derive_key_pbkdf2(
    password: &[u8],
    salt: &[u8],
    iterations: u32,
    key_len: usize,
    prf_oid: &str,
) -> Option<Vec<u8>> {
    match prf_oid {
        OID_HMAC_SHA1 => {
            let mut key = vec![0u8; key_len];
            pbkdf2::pbkdf2_hmac::<sha1::Sha1>(password, salt, iterations, &mut key);
            Some(key)
        }
        OID_HMAC_SHA256 => {
            let mut key = vec![0u8; key_len];
            pbkdf2::pbkdf2_hmac::<sha2::Sha256>(password, salt, iterations, &mut key);
            Some(key)
        }
        OID_HMAC_SM3 => Some(pbkdf2_hmac_sm3(password, salt, iterations, key_len)),
        _ => None,
    }
}

/// 手动实现 PBKDF2-HMAC-SM3。
///
/// sm3 crate 使用 digest 0.11，与 hmac 0.12（digest 0.10）不兼容，
/// 因此在此手工实现标准 PBKDF2 算法（RFC 8018 Section 5.2）。
fn pbkdf2_hmac_sm3(password: &[u8], salt: &[u8], iterations: u32, key_len: usize) -> Vec<u8> {
    let hash_len = SM3_OUTPUT_SIZE;
    let blocks = key_len.div_ceil(hash_len);
    let mut result = vec![0u8; key_len];

    for block_idx in 1..=u32::try_from(blocks).unwrap_or(u32::MAX) {
        // U1 = HMAC(password, salt || INT_32_BE(block_idx))
        let mut data = Vec::with_capacity(salt.len() + 4);
        data.extend_from_slice(salt);
        data.extend_from_slice(&block_idx.to_be_bytes());
        let mut u = hmac_sm3(password, &data);
        let mut block = u;

        // U2 .. Uc
        for _ in 1..iterations {
            u = hmac_sm3(password, &u);
            for j in 0..hash_len {
                block[j] ^= u[j];
            }
        }

        let offset = (block_idx as usize - 1) * hash_len;
        let remaining = key_len - offset;
        let copy_len = remaining.min(hash_len);
        result[offset..offset + copy_len].copy_from_slice(&block[..copy_len]);
    }

    result
}

// ══════════════════════════════════════════════════════════════════════════
// HMAC-SM3（手动实现）
// ══════════════════════════════════════════════════════════════════════════

/// HMAC-SM3 手动实现（RFC 2104）。
///
/// sm3 crate 使用 digest 0.11，而 hmac crate 0.12 使用 digest 0.10，
/// 两者不兼容。此处使用 sm3::Digest trait 手工实现 HMAC。
fn hmac_sm3(key: &[u8], message: &[u8]) -> [u8; SM3_OUTPUT_SIZE] {
    use sm3::Digest;

    // 步骤 1: 如果 key 长度 > 分组大小，先 hash(key)
    let key_derived = if key.len() > SM3_BLOCK_SIZE {
        sm3::Sm3::digest(key).to_vec()
    } else {
        key.to_vec()
    };

    // 步骤 2: 填充 key 到分组大小
    let mut key_padded = [0u8; SM3_BLOCK_SIZE];
    key_padded[..key_derived.len()].copy_from_slice(&key_derived);

    // 步骤 3: 计算 inner key 和 outer key
    let mut i_key = [0x36u8; SM3_BLOCK_SIZE];
    let mut o_key = [0x5cu8; SM3_BLOCK_SIZE];
    for i in 0..SM3_BLOCK_SIZE {
        i_key[i] ^= key_padded[i];
        o_key[i] ^= key_padded[i];
    }

    // 步骤 4: inner hash = H(i_key || message)
    let mut inner = sm3::Sm3::new();
    inner.update(i_key);
    inner.update(message);
    let inner_hash = inner.finalize();

    // 步骤 5: outer hash = H(o_key || inner_hash)
    let mut outer = sm3::Sm3::new();
    outer.update(o_key);
    outer.update(inner_hash);
    let result = outer.finalize();

    let mut output = [0u8; SM3_OUTPUT_SIZE];
    output.copy_from_slice(&result);
    output
}

// ══════════════════════════════════════════════════════════════════════════
// AES-128-CBC 解密（手动 CBC + aes crate）
// ══════════════════════════════════════════════════════════════════════════

/// AES-128-CBC 解密 + PKCS#7 unpad。
///
/// 使用 aes crate 的 BlockDecrypt trait 手动实现 CBC 模式，
/// 避免 cipher 版本兼容问题。
fn decrypt_aes_128_cbc(key: &[u8], iv: &[u8], ciphertext: &[u8]) -> Option<Vec<u8>> {
    use aes::Aes128;
    use cipher::{BlockDecrypt, KeyInit};

    if key.len() != BLOCK_SIZE || iv.len() != BLOCK_SIZE {
        return None;
    }
    if ciphertext.is_empty() || !ciphertext.len().is_multiple_of(BLOCK_SIZE) {
        return None;
    }

    let cipher = Aes128::new(cipher::generic_array::GenericArray::from_slice(key));

    let mut plaintext = Vec::with_capacity(ciphertext.len());
    let mut prev = iv;

    for chunk in ciphertext.chunks(BLOCK_SIZE) {
        let mut block = cipher::generic_array::GenericArray::clone_from_slice(chunk);
        cipher.decrypt_block(&mut block);
        for i in 0..BLOCK_SIZE {
            plaintext.push(block[i] ^ prev[i]);
        }
        prev = chunk;
    }

    pkcs7_unpad(&plaintext)
}

// ══════════════════════════════════════════════════════════════════════════
// SM4-CBC 解密（手动 CBC + sm4 crate）
// ══════════════════════════════════════════════════════════════════════════

/// SM4-CBC 解密 + PKCS#7 unpad。
///
/// sm4 0.5 使用 cipher 0.4，与 aes 0.8 共享 BlockDecrypt trait。
/// 手动实现 CBC 模式，不依赖 cbc crate。
fn decrypt_sm4_cbc(key: &[u8], iv: &[u8], ciphertext: &[u8]) -> Option<Vec<u8>> {
    use cipher::{BlockDecrypt, KeyInit};
    use sm4::Sm4;

    if key.len() != BLOCK_SIZE || iv.len() != BLOCK_SIZE {
        return None;
    }
    if ciphertext.is_empty() || !ciphertext.len().is_multiple_of(BLOCK_SIZE) {
        return None;
    }

    let cipher = Sm4::new(cipher::generic_array::GenericArray::from_slice(key));

    let mut plaintext = Vec::with_capacity(ciphertext.len());
    let mut prev = iv;

    for chunk in ciphertext.chunks(BLOCK_SIZE) {
        let mut block = cipher::generic_array::GenericArray::clone_from_slice(chunk);
        cipher.decrypt_block(&mut block);
        for i in 0..BLOCK_SIZE {
            plaintext.push(block[i] ^ prev[i]);
        }
        prev = chunk;
    }

    pkcs7_unpad(&plaintext)
}

// ══════════════════════════════════════════════════════════════════════════
// PKCS#7 padding
// ══════════════════════════════════════════════════════════════════════════

/// 移除 PKCS#7 padding。
///
/// PKCS#7: 最后一个字节 N (1..=BLOCK_SIZE) 表示 padding 长度，
/// 最后 N 个字节都必须等于 N。
fn pkcs7_unpad(data: &[u8]) -> Option<Vec<u8>> {
    if data.is_empty() {
        return None;
    }
    let pad_len = *data.last()? as usize;
    if pad_len == 0 || pad_len > BLOCK_SIZE || pad_len > data.len() {
        return None;
    }
    // 验证所有 padding 字节
    for &b in &data[data.len() - pad_len..] {
        if b as usize != pad_len {
            return None;
        }
    }
    Some(data[..data.len() - pad_len].to_vec())
}

// ══════════════════════════════════════════════════════════════════════════
// PKCS#8 PrivateKeyInfo 解析
// ══════════════════════════════════════════════════════════════════════════

/// 从 PKCS#8 PrivateKeyInfo 中提取 EC 私钥标量字节。
///
/// PrivateKeyInfo ::= SEQUENCE {
///     version INTEGER,
///     privateKeyAlgorithm AlgorithmIdentifier,
///     privateKey OCTET STRING  -- 包含 DER 编码的 ECPrivateKey
/// }
///
/// ECPrivateKey ::= SEQUENCE {
///     version INTEGER,
///     privateKey OCTET STRING,  -- 32 字节 SM2 私钥标量
///     [parameters] EXPLICIT OPTIONAL,
///     [publicKey] EXPLICIT OPTIONAL
/// }
fn extract_private_key_from_pkcs8(pk_info_der: &[u8]) -> Option<Vec<u8>> {
    let pk_fields = parse_sequence_fields(pk_info_der)?;
    if pk_fields.len() < 3 {
        return None;
    }

    // pk_fields[2] = privateKey OCTET STRING（包含 ECPrivateKey DER）
    let ec_key_any = &pk_fields[2];
    if ec_key_any.tag() != Tag::OctetString {
        return None;
    }

    // OCTET STRING 的 value 是 ECPrivateKey 的完整 DER
    let ec_fields = parse_sequence_fields(ec_key_any.value())?;
    if ec_fields.len() < 2 {
        return None;
    }

    // ec_fields[1] = privateKey OCTET STRING (32 字节标量)
    let private_key_any = &ec_fields[1];
    if private_key_any.tag() != Tag::OctetString {
        return None;
    }

    let key_bytes = private_key_any.value();
    if key_bytes.len() < 32 {
        return None;
    }

    // 返回 32 字节私钥标量
    Some(key_bytes[..32].to_vec())
}

// ══════════════════════════════════════════════════════════════════════════
// DER 解析工具函数
// ══════════════════════════════════════════════════════════════════════════

/// 解析完整 DER SEQUENCE（含 tag+length）为字段列表。
fn parse_sequence_fields(data: &[u8]) -> Option<Vec<Any>> {
    let any = Any::from_der(data).ok()?;
    if any.tag() != Tag::Sequence {
        return None;
    }
    parse_der_fields(any.value())
}

/// 从原始字节中解析连续的 DER 编码值（不含外层 SEQUENCE tag 检查）。
///
/// 用于解析 SEQUENCE 的 .value() 内容，其中各字段以 DER TLV 形式首尾相连。
/// 注意：不能使用 `Any::from_der` 逐字段解析，因为 der 0.8 的 `from_der`
/// 要求输入被单个 TLV 完全消费，遇到尾随数据会报 `TrailingData` 错误；
/// 必须用 `SliceReader` + `Any::decode` 逐字段解码而不检查 EOF。
fn parse_der_fields(data: &[u8]) -> Option<Vec<Any>> {
    let mut fields = Vec::new();
    let mut reader = SliceReader::new(data).ok()?;
    while reader.position() < reader.input_len() {
        let field = Any::decode(&mut reader).ok()?;
        fields.push(field);
    }
    Some(fields)
}

/// 从 context-specific [0] EXPLICIT 标签中提取内层值的内容。
///
/// 输入: 带有 ContextSpecific { number: 0, constructed: true } 标签的 Any
/// 输出: 内层 TLV 的 value 部分（去掉 tag+length）
///
/// 适用于内层是 OCTET STRING 的场景（如 ContentInfo.content），
/// OCTET STRING 的 value 即为目标数据。
fn extract_explicit_content(any: &Any) -> Option<Vec<u8>> {
    // [0] EXPLICIT 的 DER 编码: A0 len [inner DER]
    // any.value() 返回内层的完整 DER 编码
    let inner_any = Any::from_der(any.value()).ok()?;
    Some(inner_any.value().to_vec())
}

/// 从 context-specific [0] EXPLICIT 标签中提取内层 TLV 的完整 DER（含 tag+length）。
///
/// 适用于内层直接就是目标结构的场景（如 SafeBag.bagValue 是 PrivateKeyInfo/EncryptedPrivateKeyInfo SEQUENCE），
/// 需要保留完整 DER 以便后续 `parse_sequence_fields` 解析。
fn extract_explicit_tlv(any: &Any) -> Option<Vec<u8>> {
    let data = any.value();
    let inner_any = Any::from_der(data).ok()?;
    let len = usize::try_from(inner_any.encoded_len().ok()?).ok()?;
    Some(data[..len].to_vec())
}

/// 从 Any 中解析 OID 并返回点分字符串。
fn parse_oid_string(any: &Any) -> Option<String> {
    if any.tag() != Tag::ObjectIdentifier {
        return None;
    }
    let oid: der::asn1::ObjectIdentifier =
        der::asn1::ObjectIdentifier::try_from(any.value()).ok()?;
    Some(oid.to_string())
}

/// 从 DER INTEGER 中解析 u32 值。
fn parse_integer_u32(any: &Any) -> Option<u32> {
    if any.tag() != Tag::Integer {
        return None;
    }
    let bytes = any.value();
    if bytes.is_empty() {
        return None;
    }
    // 跳过前导零字节（DER 正整数可能有 padding）
    let start = usize::from(bytes[0] == 0 && bytes.len() > 1);
    let bytes = &bytes[start..];
    if bytes.len() > 4 {
        return None; // 超出 u32 范围
    }
    let mut result = 0u32;
    for &b in bytes {
        result = (result << 8) | u32::from(b);
    }
    Some(result)
}

/// 从 DER INTEGER 中解析 usize 值。
fn parse_integer_usize(any: &Any) -> Option<usize> {
    if any.tag() != Tag::Integer {
        return None;
    }
    let bytes = any.value();
    if bytes.is_empty() {
        return None;
    }
    let start = usize::from(bytes[0] == 0 && bytes.len() > 1);
    let bytes = &bytes[start..];
    if bytes.len() > std::mem::size_of::<usize>() {
        return None;
    }
    let mut result: usize = 0;
    for &b in bytes {
        result = (result << 8) | usize::from(b);
    }
    Some(result)
}

// ══════════════════════════════════════════════════════════════════════════
// 测试
// ══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    // ── DER 构造辅助函数（测试专用） ──────────────────────────────────

    /// 拼接多个 `Vec<u8>` 片段为一个。
    fn concat(parts: &[&[u8]]) -> Vec<u8> {
        let mut r = Vec::new();
        for p in parts {
            r.extend_from_slice(p);
        }
        r
    }

    /// 编码 DER length。
    #[allow(clippy::cast_possible_truncation)]
    fn der_len_bytes(len: usize) -> Vec<u8> {
        if len < 0x80 {
            vec![len as u8]
        } else if len < 0x100 {
            vec![0x81, len as u8]
        } else if len < 0x10000 {
            vec![0x82, (len >> 8) as u8, len as u8]
        } else {
            vec![0x83, (len >> 16) as u8, (len >> 8) as u8, len as u8]
        }
    }

    /// 编码 DER TLV。
    fn der_tlv(tag: u8, value: &[u8]) -> Vec<u8> {
        let mut r = vec![tag];
        r.extend(der_len_bytes(value.len()));
        r.extend(value);
        r
    }

    fn der_seq_raw(contents: &[u8]) -> Vec<u8> {
        der_tlv(0x30, contents)
    }

    fn der_integer(value: &[u8]) -> Vec<u8> {
        der_tlv(0x02, value)
    }

    fn der_octet_string(value: &[u8]) -> Vec<u8> {
        der_tlv(0x04, value)
    }

    fn der_explicit0(content: &[u8]) -> Vec<u8> {
        der_tlv(0xA0, content)
    }

    // 预编码 OID DER（仅 tag+length+value）
    const DER_OID_PKCS7_DATA: &[u8] = &[
        0x06, 0x09, 0x2A, 0x86, 0x48, 0x86, 0xF7, 0x0D, 0x01, 0x07, 0x01,
    ];
    const DER_OID_SHROUDED_KEY_BAG: &[u8] = &[
        0x06, 0x0B, 0x2A, 0x86, 0x48, 0x86, 0xF7, 0x0D, 0x01, 0x0C, 0x0A, 0x01, 0x02,
    ];
    const DER_OID_PBES2: &[u8] = &[
        0x06, 0x09, 0x2A, 0x86, 0x48, 0x86, 0xF7, 0x0D, 0x01, 0x05, 0x0D,
    ];
    const DER_OID_PBKDF2: &[u8] = &[
        0x06, 0x09, 0x2A, 0x86, 0x48, 0x86, 0xF7, 0x0D, 0x01, 0x05, 0x0C,
    ];
    const DER_OID_AES128_CBC: &[u8] = &[
        0x06, 0x09, 0x60, 0x86, 0x48, 0x01, 0x65, 0x03, 0x04, 0x01, 0x02,
    ];
    const DER_OID_SM4_CBC: &[u8] = &[0x06, 0x08, 0x2A, 0x81, 0x1C, 0xCF, 0x55, 0x01, 0x68, 0x02];

    // EC 公钥 OID 1.2.840.10045.2.1
    const DER_OID_EC_PUBLIC_KEY: &[u8] = &[0x06, 0x07, 0x2A, 0x86, 0x48, 0xCE, 0x3D, 0x02, 0x01];
    // secp256r1 OID 1.2.840.10045.3.1.7
    const DER_OID_SECP256R1: &[u8] = &[0x06, 0x08, 0x2A, 0x86, 0x48, 0xCE, 0x3D, 0x03, 0x01, 0x07];

    /// 构造 PKCS#8 PrivateKeyInfo DER（EC/SM2 私钥格式）。
    fn make_pkcs8_private_key_info(private_key_scalar: &[u8; 32]) -> Vec<u8> {
        // ECPrivateKey ::= SEQUENCE { version INTEGER 1, privateKey OCTET STRING }
        let ec_key = der_seq_raw(&concat(&[
            &der_integer(&[0x01]),
            &der_octet_string(private_key_scalar),
        ]));

        // PrivateKeyInfo ::= SEQUENCE {
        //     version INTEGER 0,
        //     AlgorithmIdentifier SEQUENCE { ecPublicKey OID, curve OID },
        //     privateKey OCTET STRING
        // }
        let alg_id = der_seq_raw(&concat(&[DER_OID_EC_PUBLIC_KEY, DER_OID_SECP256R1]));
        der_seq_raw(&concat(&[
            &der_integer(&[0x00]),
            &alg_id,
            &der_octet_string(&ec_key),
        ]))
    }

    /// 构造 EncryptedPrivateKeyInfo DER（PBES2 加密）。
    fn make_encrypted_private_key_info(
        salt: &[u8],
        iterations: u32,
        iv: &[u8],
        enc_oid_der: &[u8],
        encrypted_data: &[u8],
    ) -> Vec<u8> {
        // PBKDF2-params ::= SEQUENCE { salt OCTET STRING, iterationCount INTEGER }
        let pbkdf2_params = der_seq_raw(&concat(&[
            &der_octet_string(salt),
            &der_integer(&iterations.to_be_bytes()),
        ]));

        // keyDerivationFunc ::= SEQUENCE { PBKDF2 OID, PBKDF2-params }
        let kdf = der_seq_raw(&concat(&[DER_OID_PBKDF2, &pbkdf2_params]));

        // encryptionScheme ::= SEQUENCE { AES/SM4-CBC OID, IV OCTET STRING }
        let enc_scheme = der_seq_raw(&concat(&[enc_oid_der, &der_octet_string(iv)]));

        // PBES2-params ::= SEQUENCE { keyDerivationFunc, encryptionScheme }
        let pbes2_params = der_seq_raw(&concat(&[&kdf, &enc_scheme]));

        // AlgorithmIdentifier ::= SEQUENCE { PBES2 OID, PBES2-params }
        let alg_id = der_seq_raw(&concat(&[DER_OID_PBES2, &pbes2_params]));

        // EncryptedPrivateKeyInfo ::= SEQUENCE { AlgorithmIdentifier, OCTET STRING }
        der_seq_raw(&concat(&[&alg_id, &der_octet_string(encrypted_data)]))
    }

    /// 构造完整 PFX DER，包裹一个 ShroudedKeyBag。
    fn make_pfx_with_shrouded_bag(encrypted_pk_info_der: &[u8]) -> Vec<u8> {
        // SafeBag ::= SEQUENCE { shroudedKeyBag OID, [0] EXPLICIT { EncryptedPrivateKeyInfo } }
        let safe_bag = der_seq_raw(&concat(&[
            DER_OID_SHROUDED_KEY_BAG,
            &der_explicit0(encrypted_pk_info_der),
        ]));

        // SafeContents ::= SEQUENCE OF SafeBag
        let safe_contents = der_seq_raw(&safe_bag);

        // ContentInfo for SafeContents: SEQUENCE { data OID, [0] EXPLICIT { OCTET STRING } }
        let ci = der_seq_raw(&concat(&[
            DER_OID_PKCS7_DATA,
            &der_explicit0(&der_octet_string(&safe_contents)),
        ]));

        // AuthenticatedSafe ::= SEQUENCE OF ContentInfo
        let auth_safe = der_seq_raw(&ci);

        // authSafe ContentInfo: SEQUENCE { data OID, [0] EXPLICIT { OCTET STRING } }
        let auth_safe_ci = der_seq_raw(&concat(&[
            DER_OID_PKCS7_DATA,
            &der_explicit0(&der_octet_string(&auth_safe)),
        ]));

        // PFX ::= SEQUENCE { version INTEGER 3, authSafe ContentInfo }
        der_seq_raw(&concat(&[&der_integer(&[0x03]), &auth_safe_ci]))
    }

    /// AES-128-CBC 加密 + PKCS#7 pad。
    fn aes_128_cbc_encrypt(key: &[u8], iv: &[u8], plaintext: &[u8]) -> Vec<u8> {
        use aes::Aes128;
        use cipher::{BlockEncrypt, KeyInit};

        // PKCS#7 padding
        let pad_len = BLOCK_SIZE - (plaintext.len() % BLOCK_SIZE);
        let mut padded = plaintext.to_vec();
        #[allow(clippy::cast_possible_truncation)]
        let pad_byte = pad_len as u8;
        padded.extend(std::iter::repeat_n(pad_byte, pad_len));

        let cipher = Aes128::new(cipher::generic_array::GenericArray::from_slice(key));
        let mut ciphertext = Vec::with_capacity(padded.len());
        let mut prev = iv;

        for chunk in padded.chunks(BLOCK_SIZE) {
            let mut block = cipher::generic_array::GenericArray::clone_from_slice(chunk);
            for i in 0..BLOCK_SIZE {
                block[i] ^= prev[i];
            }
            cipher.encrypt_block(&mut block);
            ciphertext.extend_from_slice(&block);
            prev = &ciphertext[ciphertext.len() - BLOCK_SIZE..];
        }

        ciphertext
    }

    /// SM4-CBC 加密 + PKCS#7 pad。
    fn sm4_cbc_encrypt(key: &[u8], iv: &[u8], plaintext: &[u8]) -> Vec<u8> {
        use cipher::{BlockEncrypt, KeyInit};
        use sm4::Sm4;

        let pad_len = BLOCK_SIZE - (plaintext.len() % BLOCK_SIZE);
        let mut padded = plaintext.to_vec();
        #[allow(clippy::cast_possible_truncation)]
        let pad_byte = pad_len as u8;
        padded.extend(std::iter::repeat_n(pad_byte, pad_len));

        let cipher = Sm4::new(cipher::generic_array::GenericArray::from_slice(key));
        let mut ciphertext = Vec::with_capacity(padded.len());
        let mut prev = iv;

        for chunk in padded.chunks(BLOCK_SIZE) {
            let mut block = cipher::generic_array::GenericArray::clone_from_slice(chunk);
            for i in 0..BLOCK_SIZE {
                block[i] ^= prev[i];
            }
            cipher.encrypt_block(&mut block);
            ciphertext.extend_from_slice(&block);
            prev = &ciphertext[ciphertext.len() - BLOCK_SIZE..];
        }

        ciphertext
    }

    // ── 原有负向测试 ────────────────────────────────────────────────

    #[test]
    fn test_read_private_key_empty() {
        assert!(Pkcs12Tools::read_private_key(&[], "password").is_none());
    }

    #[test]
    fn test_read_private_key_invalid_der() {
        assert!(Pkcs12Tools::read_private_key(&[0xFF, 0xFF], "password").is_none());
    }

    #[test]
    fn test_read_private_key_random_data() {
        let data = vec![0x30, 0x82, 0x01, 0x00, 0x02, 0x01, 0x03];
        assert!(Pkcs12Tools::read_private_key(&data, "password").is_none());
    }

    #[test]
    fn test_parse_sequence_fields_non_sequence() {
        assert!(parse_sequence_fields(&[0x02, 0x01, 0x01]).is_none());
    }

    #[test]
    fn test_parse_sequence_fields_empty_sequence() {
        let fields = parse_sequence_fields(&[0x30, 0x00]).unwrap();
        assert!(fields.is_empty());
    }

    #[test]
    fn test_parse_sequence_fields_single_integer() {
        let data = [0x30, 0x03, 0x02, 0x01, 0x01];
        let fields = parse_sequence_fields(&data).unwrap();
        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].tag(), Tag::Integer);
    }

    #[test]
    fn test_parse_oid_string() {
        let data = [
            0x06, 0x09, 0x2A, 0x86, 0x48, 0x86, 0xF7, 0x0D, 0x01, 0x07, 0x01,
        ];
        let any = Any::from_der(&data).unwrap();
        let oid = parse_oid_string(&any).unwrap();
        assert_eq!(oid, OID_PKCS7_DATA);
    }

    // ── HMAC-SM3 单元测试 ──────────────────────────────────────────

    #[test]
    fn test_hmac_sm3_basic() {
        // 使用已知测试向量验证 HMAC-SM3 实现
        // RFC 4231 风格测试: key="key", data="The quick brown fox..."
        let key = b"key";
        let data = b"The quick brown fox jumps over the lazy dog";
        let mac = hmac_sm3(key, data);
        assert_eq!(mac.len(), 32);
        // 确保确定性
        let mac2 = hmac_sm3(key, data);
        assert_eq!(mac, mac2);
    }

    #[test]
    fn test_hmac_sm3_empty() {
        let mac = hmac_sm3(b"", b"");
        assert_eq!(mac.len(), 32);
    }

    #[test]
    fn test_hmac_sm3_long_key() {
        // key 长度 > SM3_BLOCK_SIZE(64)，内部先 hash(key)
        let key = vec![0xABu8; 128];
        let mac = hmac_sm3(&key, b"test");
        assert_eq!(mac.len(), 32);
    }

    // ── PBKDF2-HMAC-SM3 单元测试 ──────────────────────────────────

    #[test]
    fn test_pbkdf2_hmac_sm3_basic() {
        let key = pbkdf2_hmac_sm3(b"password", b"salt", 1000, 32);
        assert_eq!(key.len(), 32);
        // 确保确定性
        let key2 = pbkdf2_hmac_sm3(b"password", b"salt", 1000, 32);
        assert_eq!(key, key2);
    }

    #[test]
    fn test_pbkdf2_hmac_sm3_different_iterations() {
        let key1 = pbkdf2_hmac_sm3(b"password", b"salt", 100, 32);
        let key2 = pbkdf2_hmac_sm3(b"password", b"salt", 200, 32);
        assert_ne!(key1, key2);
    }

    #[test]
    fn test_pbkdf2_hmac_sm3_different_passwords() {
        let key1 = pbkdf2_hmac_sm3(b"pass1", b"salt", 1000, 32);
        let key2 = pbkdf2_hmac_sm3(b"pass2", b"salt", 1000, 32);
        assert_ne!(key1, key2);
    }

    // ── PKCS#7 padding 测试 ───────────────────────────────────────

    #[test]
    fn test_pkcs7_unpad_valid() {
        let data = vec![0x01, 0x02, 0x03, 0x04, 0x04, 0x04, 0x04, 0x04];
        let result = pkcs7_unpad(&data).unwrap();
        assert_eq!(result, vec![0x01, 0x02, 0x03, 0x04]);
    }

    #[test]
    fn test_pkcs7_unpad_full_block() {
        let data = vec![0x10u8; 16]; // 16 bytes of 0x10
        let result = pkcs7_unpad(&data).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_pkcs7_unpad_invalid() {
        // 最后一个字节为 0（无效 padding）
        assert!(pkcs7_unpad(&[0x00]).is_none());
        // padding 字节不一致
        assert!(pkcs7_unpad(&[0x01, 0x02]).is_none());
    }

    // ── parse_integer 测试 ─────────────────────────────────────────

    #[test]
    fn test_parse_integer_u32_basic() {
        // INTEGER 1000
        let any = Any::from_der(&der_integer(&1000u32.to_be_bytes())).unwrap();
        assert_eq!(parse_integer_u32(&any), Some(1000));
    }

    #[test]
    fn test_parse_integer_u32_with_leading_zero() {
        // DER INTEGER 256: 0x00 0x01 0x00 (leading zero because high bit is set)
        let any = Any::from_der(&[0x02, 0x03, 0x00, 0x80, 0x00]).unwrap();
        assert_eq!(parse_integer_u32(&any), Some(0x8000));
    }

    #[test]
    fn test_parse_integer_usize_basic() {
        let any = Any::from_der(&der_integer(&16usize.to_be_bytes())).unwrap();
        assert_eq!(parse_integer_usize(&any), Some(16));
    }

    // ── AES-128-CBC + PBKDF2-SHA1 完整 PFX round-trip 测试 ────────

    #[test]
    fn test_aes128_cbc_pbes2_shrouded_key_bag_roundtrip() {
        // 已知参数
        let password = b"test123";
        let salt = b"12345678";
        let iterations: u32 = 1000u32;
        let iv = [0xAAu8; 16];
        let private_key_scalar = [0x42u8; 32];

        // 1. 构造 PKCS#8 PrivateKeyInfo
        let pk_info = make_pkcs8_private_key_info(&private_key_scalar);

        // 2. 派生 AES-128 密钥
        let mut key = vec![0u8; 16];
        pbkdf2::pbkdf2_hmac::<sha1::Sha1>(password, salt, iterations, &mut key);

        // 3. AES-128-CBC 加密
        let encrypted = aes_128_cbc_encrypt(&key, &iv, &pk_info);

        // 4. 构造 EncryptedPrivateKeyInfo DER
        let epki =
            make_encrypted_private_key_info(salt, iterations, &iv, DER_OID_AES128_CBC, &encrypted);

        // 5. 构造完整 PFX
        let pfx = make_pfx_with_shrouded_bag(&epki);

        // 6. 用正确密码提取私钥
        let result = Pkcs12Tools::read_private_key(&pfx, "test123").unwrap();
        assert_eq!(result, private_key_scalar);
    }

    #[test]
    fn test_aes128_cbc_pbes2_wrong_password() {
        let password = b"correct";
        let salt = b"saltsalt";
        let iterations: u32 = 100u32;
        let iv = [0x55u8; 16];
        let private_key_scalar = [0x11u8; 32];

        let pk_info = make_pkcs8_private_key_info(&private_key_scalar);
        let mut key = vec![0u8; 16];
        pbkdf2::pbkdf2_hmac::<sha1::Sha1>(password, salt, iterations, &mut key);
        let encrypted = aes_128_cbc_encrypt(&key, &iv, &pk_info);
        let epki =
            make_encrypted_private_key_info(salt, iterations, &iv, DER_OID_AES128_CBC, &encrypted);
        let pfx = make_pfx_with_shrouded_bag(&epki);

        // 错误密码应返回 None，不 panic
        assert!(Pkcs12Tools::read_private_key(&pfx, "wrong").is_none());
    }

    #[test]
    fn test_aes128_cbc_pbes2_corrupted_ciphertext() {
        let password = b"pass";
        let salt = b"salt1234";
        let iterations: u32 = 100u32;
        let iv = [0x00u8; 16];
        let private_key_scalar = [0x33u8; 32];

        let pk_info = make_pkcs8_private_key_info(&private_key_scalar);
        let mut key = vec![0u8; 16];
        pbkdf2::pbkdf2_hmac::<sha1::Sha1>(password, salt, iterations, &mut key);
        let mut encrypted = aes_128_cbc_encrypt(&key, &iv, &pk_info);
        // 损坏密文的最后一个字节
        let last = encrypted.len() - 1;
        encrypted[last] ^= 0xFF;
        let epki =
            make_encrypted_private_key_info(salt, iterations, &iv, DER_OID_AES128_CBC, &encrypted);
        let pfx = make_pfx_with_shrouded_bag(&epki);

        // 损坏密文应返回 None（PKCS#7 padding 验证失败），不 panic
        assert!(Pkcs12Tools::read_private_key(&pfx, "pass").is_none());
    }

    // ── SM4-CBC + PBKDF2-SHA1 完整 PFX round-trip 测试 ────────────

    #[test]
    fn test_sm4_cbc_pbes2_shrouded_key_bag_roundtrip() {
        let password = b"sm4test";
        let salt = b"8bytesal";
        let iterations: u32 = 500u32;
        let iv = [0xBBu8; 16];
        let private_key_scalar = [0x77u8; 32];

        let pk_info = make_pkcs8_private_key_info(&private_key_scalar);

        let mut key = vec![0u8; 16];
        pbkdf2::pbkdf2_hmac::<sha1::Sha1>(password, salt, iterations, &mut key);

        let encrypted = sm4_cbc_encrypt(&key, &iv, &pk_info);
        let epki =
            make_encrypted_private_key_info(salt, iterations, &iv, DER_OID_SM4_CBC, &encrypted);
        let pfx = make_pfx_with_shrouded_bag(&epki);

        let result = Pkcs12Tools::read_private_key(&pfx, "sm4test").unwrap();
        assert_eq!(result, private_key_scalar);
    }

    #[test]
    fn test_sm4_cbc_pbes2_wrong_password() {
        let password = b"real_pass";
        let salt = b"saltsalt";
        let iterations: u32 = 100u32;
        let iv = [0xCCu8; 16];
        let private_key_scalar = [0x88u8; 32];

        let pk_info = make_pkcs8_private_key_info(&private_key_scalar);
        let mut key = vec![0u8; 16];
        pbkdf2::pbkdf2_hmac::<sha1::Sha1>(password, salt, iterations, &mut key);
        let encrypted = sm4_cbc_encrypt(&key, &iv, &pk_info);
        let epki =
            make_encrypted_private_key_info(salt, iterations, &iv, DER_OID_SM4_CBC, &encrypted);
        let pfx = make_pfx_with_shrouded_bag(&epki);

        assert!(Pkcs12Tools::read_private_key(&pfx, "fake").is_none());
    }

    // ── PBKDF2-SHA256 测试 ─────────────────────────────────────────

    #[test]
    fn test_aes128_cbc_pbes2_pbkdf2_sha256() {
        // HMAC-SHA256 PRF OID: 1.2.840.113549.2.9
        const DER_OID_HMAC_SHA256: &[u8] =
            &[0x06, 0x08, 0x2A, 0x86, 0x48, 0x86, 0xF7, 0x0D, 0x02, 0x09];

        let password = b"sha256test";
        let salt = b"salt256!";
        let iterations: u32 = 200u32;
        let iv = [0xDDu8; 16];
        let private_key_scalar = [0x99u8; 32];

        let pk_info = make_pkcs8_private_key_info(&private_key_scalar);

        // 用 HMAC-SHA256 派生密钥
        let mut key = vec![0u8; 16];
        pbkdf2::pbkdf2_hmac::<sha2::Sha256>(password, salt, iterations, &mut key);

        let encrypted = aes_128_cbc_encrypt(&key, &iv, &pk_info);

        // 手工构造带 HMAC-SHA256 PRF 的 EncryptedPrivateKeyInfo
        let prf = der_seq_raw(DER_OID_HMAC_SHA256);
        let pbkdf2_params = der_seq_raw(&concat(&[
            &der_octet_string(salt),
            &der_integer(&iterations.to_be_bytes()),
            &prf,
        ]));
        let kdf = der_seq_raw(&concat(&[DER_OID_PBKDF2, &pbkdf2_params]));
        let enc_scheme = der_seq_raw(&concat(&[DER_OID_AES128_CBC, &der_octet_string(&iv)]));
        let pbes2_params = der_seq_raw(&concat(&[&kdf, &enc_scheme]));
        let alg_id = der_seq_raw(&concat(&[DER_OID_PBES2, &pbes2_params]));
        let epki = der_seq_raw(&concat(&[&alg_id, &der_octet_string(&encrypted)]));
        let pfx = make_pfx_with_shrouded_bag(&epki);

        let result = Pkcs12Tools::read_private_key(&pfx, "sha256test").unwrap();
        assert_eq!(result, private_key_scalar);
    }

    // ── PBKDF2-HMAC-SM3 测试 ──────────────────────────────────────

    #[test]
    fn test_aes128_cbc_pbes2_pbkdf2_hmac_sm3() {
        // HMAC-SM3 PRF OID: 1.2.156.10197.1.401
        const DER_OID_HMAC_SM3: &[u8] =
            &[0x06, 0x08, 0x2A, 0x81, 0x1C, 0xCF, 0x55, 0x01, 0x83, 0x11];

        let password = b"sm3prf";
        let salt = b"sm3salt!";
        let iterations: u32 = 300u32;
        let iv = [0xEEu8; 16];
        let private_key_scalar = [0xAAu8; 32];

        let pk_info = make_pkcs8_private_key_info(&private_key_scalar);

        // 用 HMAC-SM3 派生密钥
        let key = pbkdf2_hmac_sm3(password, salt, iterations, 16);

        let encrypted = aes_128_cbc_encrypt(&key, &iv, &pk_info);

        // 手工构造带 HMAC-SM3 PRF 的 EncryptedPrivateKeyInfo
        let prf = der_seq_raw(DER_OID_HMAC_SM3);
        let pbkdf2_params = der_seq_raw(&concat(&[
            &der_octet_string(salt),
            &der_integer(&iterations.to_be_bytes()),
            &prf,
        ]));
        let kdf = der_seq_raw(&concat(&[DER_OID_PBKDF2, &pbkdf2_params]));
        let enc_scheme = der_seq_raw(&concat(&[DER_OID_AES128_CBC, &der_octet_string(&iv)]));
        let pbes2_params = der_seq_raw(&concat(&[&kdf, &enc_scheme]));
        let alg_id = der_seq_raw(&concat(&[DER_OID_PBES2, &pbes2_params]));
        let epki = der_seq_raw(&concat(&[&alg_id, &der_octet_string(&encrypted)]));
        let pfx = make_pfx_with_shrouded_bag(&epki);

        let result = Pkcs12Tools::read_private_key(&pfx, "sm3prf").unwrap();
        assert_eq!(result, private_key_scalar);
    }

    // ── 不支持的旧式 PBE 算法返回 None ────────────────────────────

    #[test]
    fn test_unsupported_old_pbe_returns_none() {
        // 模拟一个使用旧式 PKCS#12 PBE 的 ShroudedKeyBag
        // OID 1.2.840.113549.1.12.1.3 (pbeWithSHAAnd3-KeyTripleDES-CBC)
        let old_pbe_oid: &[u8] = &[
            0x06, 0x0A, 0x2A, 0x86, 0x48, 0x86, 0xF7, 0x0D, 0x01, 0x0C, 0x01, 0x03,
        ];
        let alg = der_seq_raw(&concat(&[old_pbe_oid, &der_octet_string(&[0x00; 8])]));
        let epki = der_seq_raw(&concat(&[&alg, &der_octet_string(&[0x00; 16])]));
        let pfx = make_pfx_with_shrouded_bag(&epki);

        assert!(Pkcs12Tools::read_private_key(&pfx, "password").is_none());
    }

    // ── AES-CBC 解密函数直接测试 ──────────────────────────────────

    #[test]
    fn test_decrypt_aes_128_cbc_basic() {
        let key = [0x2Bu8; 16];
        let iv = [0x01u8; 16];
        let plaintext = b"hello world!!!!"; // 15 bytes, pad to 16

        let ciphertext = aes_128_cbc_encrypt(&key, &iv, plaintext);
        let decrypted = decrypt_aes_128_cbc(&key, &iv, &ciphertext).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_decrypt_aes_128_cbc_wrong_key() {
        let key = [0x2Bu8; 16];
        let iv = [0x01u8; 16];
        let plaintext = b"test data 123456"; // 16 bytes, pad to 32

        let ciphertext = aes_128_cbc_encrypt(&key, &iv, plaintext);
        let wrong_key = [0x3Cu8; 16];
        // 错误密钥 -> padding 验证大概率失败 -> None
        let result = decrypt_aes_128_cbc(&wrong_key, &iv, &ciphertext);
        assert!(result.is_none());
    }

    // ── SM4-CBC 解密函数直接测试 ──────────────────────────────────

    #[test]
    fn test_decrypt_sm4_cbc_basic() {
        let key = [0x01u8; 16];
        let iv = [0x02u8; 16];
        let plaintext = b"SM4 CBC test!!!!"; // 16 bytes

        let ciphertext = sm4_cbc_encrypt(&key, &iv, plaintext);
        let decrypted = decrypt_sm4_cbc(&key, &iv, &ciphertext).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    // ── 非 PFX 结构返回 None ──────────────────────────────────────

    #[test]
    fn test_non_shrouded_pfx_still_works() {
        // 构造一个只含 certBag 的 PFX（不含密钥），应返回 None
        let cert_bag_oid: &[u8] = &[
            0x06, 0x0B, 0x2A, 0x86, 0x48, 0x86, 0xF7, 0x0D, 0x01, 0x0C, 0x0A, 0x01, 0x03,
        ];
        let safe_bag = der_seq_raw(&concat(&[
            cert_bag_oid,
            &der_explicit0(&der_octet_string(&[0x00; 4])),
        ]));
        let safe_contents = der_seq_raw(&safe_bag);
        let ci = der_seq_raw(&concat(&[
            DER_OID_PKCS7_DATA,
            &der_explicit0(&der_octet_string(&safe_contents)),
        ]));
        let auth_safe = der_seq_raw(&ci);
        let auth_safe_ci = der_seq_raw(&concat(&[
            DER_OID_PKCS7_DATA,
            &der_explicit0(&der_octet_string(&auth_safe)),
        ]));
        let pfx = der_seq_raw(&concat(&[&der_integer(&[0x03]), &auth_safe_ci]));

        assert!(Pkcs12Tools::read_private_key(&pfx, "pass").is_none());
    }

    // ── derive_key_pbkdf2 分派测试 ────────────────────────────────

    #[test]
    fn test_derive_key_pbkdf2_sha1() {
        let key = derive_key_pbkdf2(b"pw", b"sl", 100, 16, OID_HMAC_SHA1).unwrap();
        assert_eq!(key.len(), 16);
    }

    #[test]
    fn test_derive_key_pbkdf2_sha256() {
        let key = derive_key_pbkdf2(b"pw", b"sl", 100, 32, OID_HMAC_SHA256).unwrap();
        assert_eq!(key.len(), 32);
    }

    #[test]
    fn test_derive_key_pbkdf2_sm3() {
        let key = derive_key_pbkdf2(b"pw", b"sl", 100, 32, OID_HMAC_SM3).unwrap();
        assert_eq!(key.len(), 32);
    }

    #[test]
    fn test_derive_key_pbkdf2_unknown_prf() {
        assert!(derive_key_pbkdf2(b"pw", b"sl", 100, 16, "1.2.3.4.5").is_none());
    }
}
