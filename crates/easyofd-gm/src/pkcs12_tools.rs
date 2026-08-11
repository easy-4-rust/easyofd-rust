//! PKCS#12 解析工具。
//!
//! 对应 Java: org.ofdrw.gm.cert.PKCS12Tools
//!
//! 提供 PKCS#12 (PFX) 格式密钥库的解析功能。
//! 支持解析无加密的 KeyBag，提取 SM2 私钥。
//!
//! ## 已支持
//! - 无加密 KeyBag（bagId = 1.2.840.113549.1.12.10.1.1）
//!   直接从 PrivateKeyInfo 结构中提取 EC 私钥标量字节
//!
//! ## 不支持（返回 None 并记录原因）
//! - ShroudedKeyBag（加密私钥袋，bagId = 1.2.840.113549.1.12.10.1.2）
//!   需要 PKCS#5 v2 (PBES2+PBKDF2) 解密，当前无纯 Rust 实现
//! - EncryptedData 类型的 ContentInfo
//!   需要 PKCS#7 解密，当前无纯 Rust 实现
//! - 带 MAC 校验的 PFX（macData 字段）
//!   当前跳过 MAC 校验，仅解析 authSafe

use der::asn1::Any;
use der::{Decode, Encode, Tag, Tagged};

/// PKCS#12 工具。
///
/// 对应 Java: org.ofdrw.gm.cert.PKCS12Tools
///
/// 提供 PKCS#12 格式密钥库的解析功能。
pub struct Pkcs12Tools;

// ── PKCS#12 / PKCS#7 OID 常量 ──────────────────────────────────────────

/// PKCS#7 data 内容类型 OID: 1.2.840.113549.1.7.1
const OID_PKCS7_DATA: &str = "1.2.840.113549.1.7.1";

/// PKCS#12 KeyBag OID: 1.2.840.113549.1.12.10.1.1
const OID_KEY_BAG: &str = "1.2.840.113549.1.12.10.1.1";

/// PKCS#12 ShroudedKeyBag OID: 1.2.840.113549.1.12.10.1.2
const OID_SHROUDED_KEY_BAG: &str = "1.2.840.113549.1.12.10.1.2";

impl Pkcs12Tools {
    /// 从 PKCS#12 数据中提取私钥。
    ///
    /// 对应 Java: org.ofdrw.gm.cert.PKCS12Tools#ReadPrvKey
    ///
    /// 解析 PKCS#12 (PFX) 结构，提取第一个私钥。
    /// 返回 SM2 私钥的原始标量字节（32 字节）。
    ///
    /// 当前仅支持无加密的 KeyBag。对于加密的 ShroudedKeyBag，
    /// 返回 `None` 并在文档中说明不支持的原因。
    ///
    /// # 参数
    /// - `p12_data`: PKCS#12 格式的 DER 编码数据
    /// - `password`: 解密密码（当前未使用，预留接口）
    ///
    /// # 返回
    /// SM2 私钥原始字节（32 字节标量），如果解析失败返回 `None`。
    ///
    /// # 不支持的格式
    /// - ShroudedKeyBag（需要 PBES2+PBKDF2 解密，当前无纯 Rust SM4/AES 实现）
    /// - EncryptedData ContentInfo（需要 PKCS#7 解密）
    #[must_use]
    pub fn read_private_key(p12_data: &[u8], _password: &str) -> Option<Vec<u8>> {
        extract_first_private_key(p12_data)
    }
}

/// 从 PFX 结构中提取第一个私钥。
fn extract_first_private_key(pfx_data: &[u8]) -> Option<Vec<u8>> {
    // 1. 解析 PFX SEQUENCE { version INTEGER, authSafe ContentInfo, [macData] }
    let pfx_fields = parse_sequence_fields(pfx_data)?;
    if pfx_fields.len() < 2 {
        return None;
    }

    // 2. 解析 authSafe ContentInfo { contentType OID, content [0] EXPLICIT }
    let auth_safe_fields = parse_sequence_fields(pfx_fields[1].value())?;
    if auth_safe_fields.len() < 2 {
        return None;
    }

    // 3. 检查 contentType 是否为 data (1.2.840.113549.1.7.1)
    let content_type_oid = parse_oid_string(&auth_safe_fields[0])?;
    if content_type_oid != OID_PKCS7_DATA {
        // 非 data 类型的 ContentInfo，当前不支持
        return None;
    }

    // 4. 提取 [0] EXPLICIT 中的 OCTET STRING（包含 AuthenticatedSafe）
    let auth_safe_data = extract_explicit_content(&auth_safe_fields[1])?;

    // 5. 解析 AuthenticatedSafe ::= SEQUENCE OF ContentInfo
    let auth_safe_items = parse_sequence_fields(&auth_safe_data)?;

    // 6. 遍历每个 ContentInfo，查找包含 KeyBag 的 data 类型
    for item in &auth_safe_items {
        if let Some(key) = extract_key_from_content_info(item) {
            return Some(key);
        }
    }

    None
}

/// 从 ContentInfo 中提取私钥。
///
/// ContentInfo ::= SEQUENCE { contentType OID, content [0] EXPLICIT }
/// 如果 contentType 是 data，解析 SafeContents 并查找 KeyBag。
fn extract_key_from_content_info(ci_any: &Any) -> Option<Vec<u8>> {
    let ci_fields = parse_sequence_fields(ci_any.value())?;
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

    // 遍历每个 SafeBag，查找 KeyBag
    for bag in &safe_bags {
        if let Some(key) = extract_key_from_safe_bag(bag) {
            return Some(key);
        }
    }

    None
}

/// 从 SafeBag 中提取私钥。
///
/// SafeBag ::= SEQUENCE { bagId OID, bagValue [0] EXPLICIT, [bagAttributes] }
/// 仅处理 KeyBag (1.2.840.113549.1.12.10.1.1)。
fn extract_key_from_safe_bag(bag_any: &Any) -> Option<Vec<u8>> {
    let bag_fields = parse_sequence_fields(bag_any.value())?;
    if bag_fields.len() < 2 {
        return None;
    }

    let bag_oid = parse_oid_string(&bag_fields[0])?;

    if bag_oid == OID_KEY_BAG {
        // KeyBag: bagValue 是 PrivateKeyInfo
        let pk_info_der = extract_explicit_content(&bag_fields[1])?;
        extract_private_key_from_pkcs8(&pk_info_der)
    } else if bag_oid == OID_SHROUDED_KEY_BAG {
        // ShroudedKeyBag: 需要解密，当前不支持
        // Java 使用 BouncyCastle 的 PKCS#12 KeyStore 实现来解密
        // Rust 端需要 PBES2+PBKDF2+SM4/AES 解密，暂无纯 Rust 实现
        None
    } else {
        // 其他类型的 SafeBag（如 certBag），跳过
        None
    }
}

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

    // pk_fields[0] = version INTEGER
    // pk_fields[1] = privateKeyAlgorithm AlgorithmIdentifier
    // pk_fields[2] = privateKey OCTET STRING (包含 ECPrivateKey DER)
    let ec_key_any = &pk_fields[2];
    if ec_key_any.tag() != Tag::OctetString {
        return None;
    }

    // 解析 ECPrivateKey SEQUENCE
    let ec_fields = parse_sequence_fields(ec_key_any.value())?;
    if ec_fields.len() < 2 {
        return None;
    }

    // ec_fields[0] = version INTEGER
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

/// 从 context-specific [0] EXPLICIT 标签中提取内层 DER 值。
///
/// 输入: 带有 ContextSpecific { number: 0, constructed: true } 标签的 Any
/// 输出: 内层值的 DER 字节
fn extract_explicit_content(any: &Any) -> Option<Vec<u8>> {
    // [0] EXPLICIT 的 DER 编码: A0 len [inner DER]
    // any.value() 返回内层的完整 DER 编码
    let inner_any = Any::from_der(any.value()).ok()?;
    Some(inner_any.value().to_vec())
}

/// 从 Any 中解析 OID 并返回点分字符串。
fn parse_oid_string(any: &Any) -> Option<String> {
    if any.tag() != Tag::ObjectIdentifier {
        return None;
    }
    // 使用 ObjectIdentifier::try_from 解析 OID BER/DER 字节
    let oid: der::asn1::ObjectIdentifier =
        der::asn1::ObjectIdentifier::try_from(any.value()).ok()?;
    Some(oid.to_string())
}

/// 解析 DER SEQUENCE 的字段列表。
///
/// 输入: 包含 SEQUENCE DER 编码的字节（包括 SEQUENCE tag+length）
/// 输出: SEQUENCE 内各字段的 Any 列表
fn parse_sequence_fields(data: &[u8]) -> Option<Vec<Any>> {
    let any = Any::from_der(data).ok()?;
    if any.tag() != Tag::Sequence {
        return None;
    }
    let value = any.value();
    let mut fields = Vec::new();
    let mut offset = 0;
    while offset < value.len() {
        let field = Any::from_der(&value[offset..]).ok()?;
        let field_len = usize::try_from(field.encoded_len().ok()?).ok()?;
        fields.push(field);
        offset += field_len;
    }
    Some(fields)
}

#[cfg(test)]
mod tests {
    use super::*;

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
        // 随机数据不应解析成功
        let data = vec![0x30, 0x82, 0x01, 0x00, 0x02, 0x01, 0x03];
        assert!(Pkcs12Tools::read_private_key(&data, "password").is_none());
    }

    #[test]
    fn test_parse_sequence_fields_non_sequence() {
        // 非 SEQUENCE 数据应返回 None
        assert!(parse_sequence_fields(&[0x02, 0x01, 0x01]).is_none());
    }

    #[test]
    fn test_parse_sequence_fields_empty_sequence() {
        // 空 SEQUENCE 应返回空 Vec
        let fields = parse_sequence_fields(&[0x30, 0x00]).unwrap();
        assert!(fields.is_empty());
    }

    #[test]
    fn test_parse_sequence_fields_single_integer() {
        // SEQUENCE { INTEGER 1 }
        let data = [0x30, 0x03, 0x02, 0x01, 0x01];
        let fields = parse_sequence_fields(&data).unwrap();
        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].tag(), Tag::Integer);
    }

    #[test]
    fn test_parse_oid_string() {
        // OID 1.2.840.113549.1.7.1
        let data = [
            0x06, 0x09, 0x2A, 0x86, 0x48, 0x86, 0xF7, 0x0D, 0x01, 0x07, 0x01,
        ];
        let any = Any::from_der(&data).unwrap();
        let oid = parse_oid_string(&any).unwrap();
        assert_eq!(oid, OID_PKCS7_DATA);
    }
}
