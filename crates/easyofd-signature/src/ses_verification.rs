//! SES真实密码验签，使用原始TBS以保留未知的印章扩展。
use crate::errors::OfdVerifyException;
use der::asn1::AnyRef;
use der::{Decode, Encode, Reader, Tag, Tagged};
use sm2::dsa::signature::Verifier;
use sm3::Digest;
fn error(message: impl Into<String>) -> OfdVerifyException {
    OfdVerifyException::new(message)
}
fn fields(value: AnyRef<'_>) -> Result<Vec<AnyRef<'_>>, OfdVerifyException> {
    if value.tag() != Tag::Sequence {
        return Err(error("SES字段必须为SEQUENCE"));
    }
    let mut r = der::SliceReader::new(value.value()).map_err(|e| error(e.to_string()))?;
    let mut out = Vec::new();
    while !r.is_finished() {
        out.push(AnyRef::decode(&mut r).map_err(|e| error(e.to_string()))?);
    }
    Ok(out)
}
fn at<'a>(items: &[AnyRef<'a>], i: usize) -> Result<AnyRef<'a>, OfdVerifyException> {
    items.get(i).copied().ok_or_else(|| error("SES缺少字段"))
}
fn octets(value: AnyRef<'_>) -> Result<&[u8], OfdVerifyException> {
    match value.tag() {
        Tag::OctetString => Ok(value.value()),
        Tag::BitString => {
            if value.value().first() != Some(&0) {
                return Err(error("SES位串填充错误"));
            }
            Ok(&value.value()[1..])
        }
        _ => Err(error("SES字节字段类型错误")),
    }
}
/// 验证指定版本的原文摘要、算法和证书公钥签名，不替代证书信任认证。
pub(crate) fn validate(
    version: u64,
    message: &[u8],
    value: &[u8],
) -> Result<(), OfdVerifyException> {
    let outer = fields(AnyRef::from_der(value).map_err(|e| error(e.to_string()))?)?;
    let tbs = at(&outer, 0)?;
    let inner = fields(tbs)?;
    let ver = at(&inner, 0)?;
    if ver.tag() != Tag::Integer
        || ver.value() != [u8::try_from(version).map_err(|_| error("SES版本越界"))?]
    {
        return Err(error("SES版本不匹配"));
    }
    let (cert, oid, sig) = if version == 1 {
        (at(&inner, 5)?, at(&inner, 6)?, at(&outer, 1)?)
    } else if version == 4 || version == 5 {
        (at(&outer, 1)?, at(&outer, 2)?, at(&outer, 3)?)
    } else {
        return Err(error("不支持的SES版本"));
    };
    let expected: der::asn1::ObjectIdentifier = "1.2.156.10197.1.501"
        .parse()
        .map_err(|_| error("内部算法OID无效"))?;
    if oid.to_der().map_err(|e| error(e.to_string()))?
        != expected.to_der().map_err(|e| error(e.to_string()))?
    {
        return Err(error("非SM2WithSM3签名"));
    }
    if octets(at(&inner, 3)?)? != sm3::Sm3::digest(message).as_slice() {
        return Err(error("原文摘要不匹配"));
    }
    let certificate = x509_cert::Certificate::from_der(octets(cert)?)
        .map_err(|e| error(format!("证书DER: {e}")))?;
    let public = certificate
        .tbs_certificate()
        .subject_public_key_info()
        .subject_public_key
        .raw_bytes();
    let key = sm2::dsa::VerifyingKey::from_sec1_bytes("1234567812345678", public)
        .map_err(|e| error(format!("SM2公钥: {e}")))?;
    let bytes = octets(sig)?;
    // 接受DER互操作签名及历史Rust raw编码。
    let signature = sm2::dsa::Signature::from_der(bytes)
        .or_else(|_| sm2::dsa::Signature::from_slice(bytes))
        .map_err(|e| error(format!("SM2签名编码: {e}")))?;
    key.verify(&tbs.to_der().map_err(|e| error(e.to_string()))?, &signature)
        .map_err(|e| error(format!("SES密码验签失败: {e}")))
}
