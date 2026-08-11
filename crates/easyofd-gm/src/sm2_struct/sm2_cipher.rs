//! SM2Cipher 结构。
//!
//! 对应 Java: org.ofdrw.gm.sm2strut.SM2Cipher

use crate::ses::{DerResult, TAG_INTEGER, TAG_OCTET_STRING, decode_sequence, expect_tlv};
use crate::ses::{encode_integer, encode_octet_string, encode_sequence};

/// SM2 加密结果（GB/T 35275 SM2Cipher）。
///
/// 对应 Java: ofdrw SM2Cipher。
/// DER 布局：
/// ```asn1
/// SM2Cipher ::= SEQUENCE {
///     xCoordinate  INTEGER,
///     yCoordinate  INTEGER,
///     hash         OCTET STRING,
///     cipherText   OCTET STRING
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sm2Cipher {
    /// 椭圆曲线点 X 坐标（DER INTEGER 原始字节）。
    pub x_coordinate: Vec<u8>,
    /// 椭圆曲线点 Y 坐标（DER INTEGER 原始字节）。
    pub y_coordinate: Vec<u8>,
    /// 杂凑值。
    pub hash: Vec<u8>,
    /// 密文。
    pub cipher_text: Vec<u8>,
}

impl Sm2Cipher {
    /// 创建 SM2 加密结果。
    #[must_use]
    pub fn new(
        x_coordinate: Vec<u8>,
        y_coordinate: Vec<u8>,
        hash: Vec<u8>,
        cipher_text: Vec<u8>,
    ) -> Self {
        Self {
            x_coordinate,
            y_coordinate,
            hash,
            cipher_text,
        }
    }

    /// 编码为 DER 字节。
    ///
    /// # 错误
    ///
    /// DER 编码不失败，此签名保留以对齐 ofdrw API。
    pub fn to_der(&self) -> DerResult<Vec<u8>> {
        let mut inner = Vec::new();
        encode_int_raw(&self.x_coordinate, &mut inner);
        encode_int_raw(&self.y_coordinate, &mut inner);
        encode_octet_string(&self.hash, &mut inner);
        encode_octet_string(&self.cipher_text, &mut inner);
        let mut out = Vec::new();
        encode_sequence(&inner, &mut out);
        Ok(out)
    }

    /// 从 DER 字节解码。
    ///
    /// # 错误
    ///
    /// 输入不是合法的 SM2Cipher DER 序列时返回错误。
    pub fn from_der(der: &[u8]) -> DerResult<Self> {
        let (seq, _) = decode_sequence(der, 0)?;
        let (x_val, pos) = expect_tlv(&seq, 0, TAG_INTEGER)?;
        let (y_val, pos) = expect_tlv(&seq, pos, TAG_INTEGER)?;
        let (hash, pos) = expect_tlv(&seq, pos, TAG_OCTET_STRING)?;
        let (cipher, _) = expect_tlv(&seq, pos, TAG_OCTET_STRING)?;
        Ok(Self {
            x_coordinate: x_val,
            y_coordinate: y_val,
            hash,
            cipher_text: cipher,
        })
    }
}

/// 将裸坐标字节编码为 DER INTEGER（含 0x00 前导保护位）。
fn encode_int_raw(bytes: &[u8], out: &mut Vec<u8>) {
    let significant = bytes
        .iter()
        .position(|&b| b != 0)
        .unwrap_or(bytes.len() - 1);
    let payload = &bytes[significant..];
    let needs_zero = !payload.is_empty() && (payload[0] & 0x80 != 0);
    out.push(TAG_INTEGER);
    crate::ses::encode_length(payload.len() + usize::from(needs_zero), out);
    if needs_zero {
        out.push(0x00);
    }
    out.extend_from_slice(payload);
}

/// 便捷：从 u64 坐标构建（用于测试与简化调用）。
impl Sm2Cipher {
    /// 从无符号整数坐标创建。
    #[must_use]
    pub fn from_u64(x: u64, y: u64, hash: Vec<u8>, cipher_text: Vec<u8>) -> Self {
        let mut xb = Vec::new();
        encode_integer(x, &mut xb);
        let mut yb = Vec::new();
        encode_integer(y, &mut yb);
        Self::new(xb, yb, hash, cipher_text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip() {
        let c = Sm2Cipher::from_u64(0x1122, 0x3344, vec![0xAB], vec![0xCD, 0xEF]);
        let der = c.to_der().unwrap();
        let decoded = Sm2Cipher::from_der(&der).unwrap();
        assert_eq!(decoded.hash, vec![0xAB]);
        assert_eq!(decoded.cipher_text, vec![0xCD, 0xEF]);
        assert_eq!(decoded.x_coordinate, c.x_coordinate);
    }
}
