//! GB35275与CMS SM2签名验证，证书列表作为显式签名证书pin，不冒充完整PKI认证。
use crate::errors::OfdVerifyException;
use crate::verify_containers::SignedDataValidateContainer;
use der::asn1::{AnyRef, ObjectIdentifier};
use der::{Decode, Encode, Reader, Tag, Tagged};
use sm3::Digest;
/// GB35275验证容器。对应Java: org.ofdrw.sign.verify.container.GBT35275ValidateContainer。
pub struct Gbt35275ValidateContainer {
    trust_certs: Vec<Vec<u8>>,
}
impl Gbt35275ValidateContainer {
    /// 参数为明确允许的签名者证书DER（pin），返回容器；不是CA链/吊销认证接口。
    #[must_use]
    pub fn new(trust_certs: Vec<Vec<u8>>) -> Self {
        Self { trust_certs }
    }
    /// 返回允许的签名者证书DER列表。
    #[must_use]
    pub fn trust_certs(&self) -> &[Vec<u8>] {
        &self.trust_certs
    }
}
fn error(s: impl Into<String>) -> OfdVerifyException {
    OfdVerifyException::new(s)
}
fn fields(value: AnyRef<'_>) -> Result<Vec<AnyRef<'_>>, OfdVerifyException> {
    let mut r = der::SliceReader::new(value.value()).map_err(|e| error(e.to_string()))?;
    let mut out = Vec::new();
    while !r.is_finished() {
        if out.len() >= 4096 {
            return Err(error("CMS字段数量超限"));
        }
        out.push(AnyRef::decode(&mut r).map_err(|e| error(e.to_string()))?);
    }
    Ok(out)
}
fn sequence(v: AnyRef<'_>) -> Result<Vec<AnyRef<'_>>, OfdVerifyException> {
    if v.tag() != Tag::Sequence {
        return Err(error("CMS应为SEQUENCE"));
    }
    fields(v)
}
fn oid(v: AnyRef<'_>) -> Result<String, OfdVerifyException> {
    let v = if v.tag() == Tag::Sequence {
        let parts = sequence(v)?;
        if parts.is_empty() || parts.len() > 2 || (parts.len() == 2 && parts[1].tag() != Tag::Null)
        {
            return Err(error("算法参数不支持"));
        }
        parts[0]
    } else {
        v
    };
    if v.tag() != Tag::ObjectIdentifier {
        return Err(error("CMS缺少OID"));
    }
    ObjectIdentifier::from_der(&v.to_der().map_err(|e| error(e.to_string()))?)
        .map(|v: ObjectIdentifier| v.to_string())
        .map_err(|e| error(e.to_string()))
}
fn explicit(v: AnyRef<'_>) -> Result<AnyRef<'_>, OfdVerifyException> {
    if v.tag() == der::TagNumber(0).context_specific(true) {
        let parts = fields(v)?;
        if parts.len() != 1 {
            return Err(error("显式content字段数量错误"));
        }
        Ok(parts[0])
    } else {
        Ok(v)
    } // 历史Rust直接content编码仍只在真实验签后接受。
}
fn integer(v: AnyRef<'_>, expected: u8) -> Result<(), OfdVerifyException> {
    if v.tag() != Tag::Integer || v.value() != [expected] {
        return Err(error("CMS版本不支持"));
    }
    Ok(())
}
fn bytes(v: AnyRef<'_>) -> Result<&[u8], OfdVerifyException> {
    if v.tag() != Tag::OctetString {
        return Err(error("CMS字节字段不是OCTET STRING"));
    }
    Ok(v.value())
}
fn normalized_integer(data: &[u8]) -> &[u8] {
    let mut i = 0;
    while i + 1 < data.len() && data[i] == 0 {
        i += 1;
    }
    &data[i..]
}
impl SignedDataValidateContainer for Gbt35275ValidateContainer {
    fn validate(&self, message: &[u8], value: &[u8]) -> Result<(), OfdVerifyException> {
        if self.trust_certs.is_empty() || value.len() > 32 * 1024 * 1024 {
            return Err(error("缺少签名证书pin或签名长度超限"));
        }
        let root = AnyRef::from_der(value).map_err(|e| error(e.to_string()))?;
        let outer = sequence(root)?;
        if outer.len() != 2 {
            return Err(error("ContentInfo字段错误"));
        }
        let kind = oid(outer[0])?;
        let gm = kind == "1.2.156.10197.6.1.4.2.2";
        if !gm && kind != "1.2.840.113549.1.7.2" {
            return Err(error("非signedData类型"));
        }
        let legacy_gm = gm && outer[1].tag() == Tag::Sequence;
        let signed = sequence(explicit(outer[1])?)?;
        if signed.len() < 4 {
            return Err(error("SignedData字段缺失"));
        }
        integer(signed[0], 1)?;
        if signed[1].tag() != Tag::Set {
            return Err(error("摘要算法必须是SET"));
        }
        let algorithms = fields(signed[1])?;
        if algorithms.is_empty()
            || algorithms
                .iter()
                .any(|a| oid(*a).ok().as_deref() != Some("1.2.156.10197.1.401"))
        {
            return Err(error("不支持的摘要算法"));
        }
        let content = sequence(signed[2])?;
        if content.is_empty() || content.len() > 2 {
            return Err(error("内部ContentInfo字段错误"));
        }
        let expected_type = if gm {
            "1.2.156.10197.6.1.4.2.1"
        } else {
            "1.2.840.113549.1.7.1"
        };
        if oid(content[0])? != expected_type {
            return Err(error("签名内容类型错误"));
        }
        let digest = sm3::Sm3::digest(message);
        let payload = if gm { digest.as_slice() } else { message };
        if content.len() == 2 && bytes(explicit(content[1])?)? != payload {
            return Err(error("签名内容与原文不匹配"));
        }
        let mut i = 3;
        let mut certs = Vec::new();
        if signed
            .get(i)
            .is_some_and(|v| v.tag() == der::TagNumber(0).context_specific(true))
        {
            certs = fields(signed[i])?;
            i += 1;
        }
        if signed
            .get(i)
            .is_some_and(|v| v.tag() == der::TagNumber(1).context_specific(true))
        {
            i += 1;
        } // CRL不在此密码验签接口的认证范围。
        if i + 1 != signed.len() || signed[i].tag() != Tag::Set {
            return Err(error("SignerInfo集合错误"));
        }
        let signers = fields(signed[i])?;
        if signers.is_empty() || signers.len() > 64 {
            return Err(error("签名者为空或数量超限"));
        }
        for signer in signers {
            let parts = sequence(signer)?;
            if parts.len() < 5 {
                return Err(error("SignerInfo字段缺失"));
            }
            integer(parts[0], 1)?;
            let sid = sequence(parts[1])?;
            if sid.len() != 2 || sid[1].tag() != Tag::Integer {
                return Err(error("不支持的签名者标识"));
            }
            if oid(parts[2])? != "1.2.156.10197.1.401" {
                return Err(error("签名者摘要算法错误"));
            }
            let mut p = 3;
            let plaintext = if parts[p].tag() == der::TagNumber(0).context_specific(true) {
                let a = parts[p];
                p += 1;
                // 标准IMPLICIT SET和历史Rust嵌套SET两种属性编码，保留签名覆盖的原始字节。
                let mut attrs = fields(a)?;
                let nested_set = attrs.len() == 1 && attrs[0].tag() == Tag::Set;
                let encoded = if nested_set {
                    let encoded = attrs[0].to_der().map_err(|e| error(e.to_string()))?;
                    attrs = fields(attrs[0])?;
                    encoded
                } else {
                    let mut encoded = a.to_der().map_err(|e| error(e.to_string()))?;
                    encoded[0] = 0x31;
                    encoded
                };
                let mut found = false;
                let mut content_type_found = false;
                let mut names = std::collections::HashSet::new();
                for attr in attrs {
                    let pair = sequence(attr)?;
                    if pair.len() != 2 || pair[1].tag() != Tag::Set {
                        return Err(error("认证属性结构错误"));
                    }
                    let name = oid(pair[0])?;
                    if !names.insert(name.clone()) {
                        return Err(error("认证属性OID重复"));
                    }
                    if name == "1.2.840.113549.1.9.4" {
                        let vals = fields(pair[1])?;
                        if found || vals.len() != 1 || bytes(vals[0])? != digest.as_slice() {
                            return Err(error("认证摘要缺失/重复/不匹配"));
                        }
                        found = true;
                    } else if name == "1.2.840.113549.1.9.3" {
                        content_type_found = true;
                        let vals = fields(pair[1])?;
                        if vals.len() != 1 || oid(vals[0])? != expected_type {
                            return Err(error("认证内容类型不匹配"));
                        }
                    }
                }
                if !found {
                    return Err(error("缺少认证messageDigest"));
                }
                // 仅旧Rust GB直接content加嵌套SET编码保留缺失contentType的兼容性。
                if !(content_type_found || legacy_gm && nested_set) {
                    return Err(error("缺少认证contentType"));
                }
                encoded
            } else {
                if content.len() != 2 {
                    return Err(error("无属性签名缺少内容"));
                }
                payload.to_vec()
            };
            if p + 2 > parts.len() {
                return Err(error("签名者算法或签名缺失"));
            }
            let algorithm = oid(parts[p])?;
            if algorithm != "1.2.156.10197.1.501" && algorithm != "1.2.156.10197.1.301.1" {
                return Err(error("非SM2WithSM3算法"));
            }
            let sig = bytes(parts[p + 1])?;
            if parts.len() > p + 3
                || (parts.len() == p + 3
                    && parts[p + 2].tag() != der::TagNumber(1).context_specific(true))
            {
                return Err(error("签名者尾随字段错误"));
            }
            let mut matched = None;
            for raw in &certs {
                let encoded = raw.to_der().map_err(|e| error(e.to_string()))?;
                if !self.trust_certs.iter().any(|pin| pin == &encoded) {
                    continue;
                }
                let certificate =
                    x509_cert::Certificate::from_der(&encoded).map_err(|e| error(e.to_string()))?;
                let tbs = certificate.tbs_certificate();
                if tbs.issuer().to_der().map_err(|e| error(e.to_string()))?
                    == sid[0].to_der().map_err(|e| error(e.to_string()))?
                    && normalized_integer(tbs.serial_number().as_bytes())
                        == normalized_integer(sid[1].value())
                {
                    matched = Some(
                        tbs.subject_public_key_info()
                            .subject_public_key
                            .raw_bytes()
                            .to_vec(),
                    );
                    break;
                }
            }
            let public = matched.ok_or_else(|| error("签名者证书未匹配显式pin/Issuer/Serial"))?;
            crate::digital_validate_container::verify(&public, &plaintext, sig)?;
        }
        Ok(())
    }
}
