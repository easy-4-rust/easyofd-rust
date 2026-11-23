//! SES签章的原始印章保留和DER签名编码。
use crate::errors::SignError;
use crate::internal_helpers::compute_sm3;
use crate::timestamp_hook::TimeStampHook;
use der::{Decode, Encode, Tag, Tagged};
use easyofd_gm::ses::{encode_length, encode_octet_string, encode_oid, encode_sequence};
use sm2::dsa::signature::Signer;
fn tlv(tag: u8, data: &[u8], out: &mut Vec<u8>) {
    out.push(tag);
    encode_length(data.len(), out);
    out.extend_from_slice(data);
}
fn bit(data: &[u8], out: &mut Vec<u8>) {
    let mut v = vec![0];
    v.extend_from_slice(data);
    tlv(3, &v, out);
}
/// 按版本签章，保留印章原始DER，输出标准DER SM2签名。
pub(crate) fn sign(
    version: u64,
    key: &sm2::SecretKey,
    seal: &[u8],
    cert: &[u8],
    message: &[u8],
    property: &str,
    hook: Option<&dyn TimeStampHook>,
) -> Result<Vec<u8>, SignError> {
    let node = der::asn1::AnyRef::from_der(seal).map_err(|e| SignError::Decode(e.to_string()))?;
    if node.tag() != Tag::Sequence {
        return Err(SignError::Decode("印章应为SEQUENCE".into()));
    }
    if !property.is_ascii() {
        return Err(SignError::Encode("propertyInfo必须为IA5String".into()));
    }
    let oid = [1, 2, 156, 10_197, 1, 501];
    let mut inner = Vec::new();
    inner.extend_from_slice(
        &version
            .to_der()
            .map_err(|e| SignError::Encode(e.to_string()))?,
    );
    inner.extend_from_slice(seal);
    if version == 1 {
        bit(&super::sign_containers::local_time_utf8_bytes(), &mut inner);
    } else {
        tlv(
            0x18,
            super::sign_containers::generalized_time_now().as_bytes(),
            &mut inner,
        );
    }
    bit(&compute_sm3(message), &mut inner);
    tlv(0x16, property.as_bytes(), &mut inner);
    if version == 1 {
        encode_octet_string(cert, &mut inner);
        encode_oid(&oid, &mut inner);
    }
    let mut tbs = Vec::new();
    encode_sequence(&inner, &mut tbs);
    let signer = sm2::dsa::SigningKey::new("1234567812345678", key)
        .map_err(|e| SignError::Signing(e.to_string()))?;
    let signature: sm2::dsa::Signature = signer.sign(&tbs);
    let signature = signature.to_der();
    let mut outer = tbs;
    if version != 1 {
        encode_octet_string(cert, &mut outer);
        encode_oid(&oid, &mut outer);
    }
    bit(signature.as_bytes(), &mut outer);
    if let Some(hook) = hook {
        let ts = hook.apply(signature.as_bytes());
        if !ts.is_empty() {
            let mut timestamp = Vec::new();
            bit(&ts, &mut timestamp);
            tlv(0xa0, &timestamp, &mut outer);
        }
    }
    let mut out = Vec::new();
    encode_sequence(&outer, &mut out);
    Ok(out)
}
