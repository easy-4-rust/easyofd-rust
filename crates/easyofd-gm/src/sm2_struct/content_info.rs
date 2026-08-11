//! ContentInfo 结构。
//!
//! 对应 Java: org.ofdrw.gm.sm2strut.ContentInfo

use crate::ses::{DerResult, TAG_OBJECT_IDENTIFIER, decode_sequence, expect_tlv};
use crate::ses::{encode_oid, encode_sequence};

use super::oids::{SIGNED_DATA, parse_oid};
use super::signed_data::SignedData;

/// 内容信息（PKCS#7 ContentInfo）。
///
/// 对应 Java: ofdrw ContentInfo。
/// DER 布局：`SEQUENCE { contentType OBJECT IDENTIFIER, content ANY }`。
#[derive(Debug, Clone, PartialEq)]
pub struct ContentInfo {
    /// 内容类型 OID 弧段。
    pub content_type: Vec<u32>,
    /// 内容（已编码的 DER 字节，如 SignedData）。
    pub content: Vec<u8>,
}

impl ContentInfo {
    /// 创建新的内容信息。
    #[must_use]
    pub fn new(content_type: Vec<u32>, content: Vec<u8>) -> Self {
        Self {
            content_type,
            content,
        }
    }

    /// 从 SignedData 构建 ContentInfo（contentType = signedData）。
    ///
    /// 对应 Java: ofdrw ContentInfo(SignedData)。
    ///
    /// # 错误
    ///
    /// SignedData DER 编码失败时返回错误。
    pub fn from_signed_data(signed_data: &SignedData) -> DerResult<Self> {
        Ok(Self::new(parse_oid(SIGNED_DATA), signed_data.to_der()?))
    }

    /// 编码为 DER 字节。
    ///
    /// # 错误
    ///
    /// DER 编码不失败，此签名保留以对齐 ofdrw API。
    pub fn to_der(&self) -> DerResult<Vec<u8>> {
        let mut inner = Vec::new();
        encode_oid(&self.content_type, &mut inner);
        inner.extend_from_slice(&self.content);
        let mut out = Vec::new();
        encode_sequence(&inner, &mut out);
        Ok(out)
    }

    /// 从 DER 字节解码。
    ///
    /// # 错误
    ///
    /// 输入不是合法的 ContentInfo DER 序列时返回错误。
    pub fn from_der(der: &[u8]) -> DerResult<Self> {
        let (seq, _) = decode_sequence(der, 0)?;
        let (oid_val, pos) = expect_tlv(&seq, 0, TAG_OBJECT_IDENTIFIER)?;
        let content_type = crate::ses::decode_oid(&oid_val)?;
        let content = seq[pos..].to_vec();
        Ok(Self::new(content_type, content))
    }

    /// 尝试将内容解析为 SignedData。
    ///
    /// # 错误
    ///
    /// 内容不是合法的 SignedData DER 序列时返回错误。
    pub fn get_signed_data(&self) -> DerResult<SignedData> {
        SignedData::from_der(&self.content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip() {
        let ci = ContentInfo::new(parse_oid(SIGNED_DATA), vec![0x30, 0x00]);
        let der = ci.to_der().unwrap();
        let decoded = ContentInfo::from_der(&der).unwrap();
        assert_eq!(decoded.content_type, ci.content_type);
        assert_eq!(decoded.content, vec![0x30, 0x00]);
    }
}
