//! PKCS#12密码完整性校验；按RFC7292在提取密钥前验证原始AuthenticatedSafe字节。
use crate::pkcs12_tools::{parse_der_fields, parse_integer_u32, parse_oid_string};
use der::asn1::Any;
use der::{Tag, Tagged};
use hmac::{Hmac, Mac};
use sha1::Digest;

/// 攻击者可控的密码派生工作量上限；大于该预算拒绝导入。
pub(crate) const MAX_KDF_ITERATIONS: u32 = 2_000_000;
fn hash(oid: &str, data: &[u8]) -> Option<Vec<u8>> {
    Some(match oid {
        "1.3.14.3.2.26" => sha1::Sha1::digest(data).to_vec(),
        "2.16.840.1.101.3.4.2.4" => sha2::Sha224::digest(data).to_vec(),
        "2.16.840.1.101.3.4.2.1" => sha2::Sha256::digest(data).to_vec(),
        "2.16.840.1.101.3.4.2.2" => sha2::Sha384::digest(data).to_vec(),
        "2.16.840.1.101.3.4.2.3" => sha2::Sha512::digest(data).to_vec(),
        "2.16.840.1.101.3.4.2.5" => sha2::Sha512_224::digest(data).to_vec(),
        "2.16.840.1.101.3.4.2.6" => sha2::Sha512_256::digest(data).to_vec(),
        _ => return None,
    })
}
fn key(oid: &str, password: &[u8], salt: &[u8], iterations: u32) -> Option<Vec<u8>> {
    let block = if matches!(
        oid,
        "1.3.14.3.2.26" | "2.16.840.1.101.3.4.2.4" | "2.16.840.1.101.3.4.2.1"
    ) {
        64
    } else {
        128
    };
    // MAC密钥长度等于摘要长度，只需RFC7292 B.2的首个输出块。
    let mut seed = vec![3; block];
    for input in [salt, password] {
        if !input.is_empty() {
            seed.extend(
                input
                    .iter()
                    .cycle()
                    .take(input.len().div_ceil(block) * block),
            );
        }
    }
    let mut result = hash(oid, &seed)?;
    for _ in 1..iterations {
        result = hash(oid, &result)?;
    }
    Some(result)
}
fn check(oid: &str, key: &[u8], data: &[u8], expected: &[u8]) -> Option<()> {
    // verify_slice由hmac crate以恒时比较验证标签，不自行用普通字节相等比较。
    macro_rules! verify {
        ($hash:ty) => {{
            let mut mac = Hmac::<$hash>::new_from_slice(key).ok()?;
            mac.update(data);
            mac.verify_slice(expected).ok()
        }};
    }
    match oid {
        "1.3.14.3.2.26" => verify!(sha1::Sha1),
        "2.16.840.1.101.3.4.2.4" => verify!(sha2::Sha224),
        "2.16.840.1.101.3.4.2.1" => verify!(sha2::Sha256),
        "2.16.840.1.101.3.4.2.2" => verify!(sha2::Sha384),
        "2.16.840.1.101.3.4.2.3" => verify!(sha2::Sha512),
        "2.16.840.1.101.3.4.2.5" => verify!(sha2::Sha512_224),
        "2.16.840.1.101.3.4.2.6" => verify!(sha2::Sha512_256),
        _ => None,
    }
}
/// 参数为MacData、原始安全内容和口令；完整性、算法或预算不满足时返回None。
pub(crate) fn verify(mac: &Any, data: &[u8], password: &str) -> Option<()> {
    if mac.tag() != Tag::Sequence {
        return None;
    }
    let fields = parse_der_fields(mac.value())?;
    if !(2..=3).contains(&fields.len())
        || fields[0].tag() != Tag::Sequence
        || fields[1].tag() != Tag::OctetString
    {
        return None;
    }
    let info = parse_der_fields(fields[0].value())?;
    if info.len() != 2 || info[0].tag() != Tag::Sequence || info[1].tag() != Tag::OctetString {
        return None;
    }
    let algorithm = parse_der_fields(info[0].value())?;
    if algorithm.is_empty()
        || algorithm.len() > 2
        || (algorithm.len() == 2 && algorithm[1].tag() != Tag::Null)
    {
        return None;
    }
    let oid = parse_oid_string(&algorithm[0])?;
    let iterations = if fields.len() == 3 {
        parse_integer_u32(&fields[2])?
    } else {
        1
    };
    let salt = fields[1].value();
    if !(1..=MAX_KDF_ITERATIONS).contains(&iterations)
        || salt.len() > 65536
        || password.len() > 4096
    {
        return None;
    }
    let bmp: Vec<u8> = password
        .encode_utf16()
        .chain(std::iter::once(0))
        .flat_map(u16::to_be_bytes)
        .collect();
    check(
        &oid,
        &key(&oid, &bmp, salt, iterations)?,
        data,
        info[1].value(),
    )
    .or_else(|| {
        // BC旧版的空口令使用空字节，仍必须经过真实MAC校验。
        if password.is_empty() {
            check(
                &oid,
                &key(&oid, &[], salt, iterations)?,
                data,
                info[1].value(),
            )
        } else {
            None
        }
    })
}
