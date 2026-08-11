//! IssuerAndSerialNumber 结构。
//!
//! 对应 Java: org.ofdrw.gm.sm2strut.IssuerAndSerialNumber

use crate::ses::{DerResult, TAG_INTEGER, decode_sequence, expect_tlv};
use crate::ses::{encode_integer, encode_printable_string, encode_sequence};

/// 证书签发者名称与序列号（PKCS#7 SignerInfo 的 sid）。
///
/// 对应 Java: ofdrw IssuerAndSerialNumber。
/// DER 布局：`SEQUENCE { name Name, certSerialNumber INTEGER }`。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IssuerAndSerialNumber {
    /// 签发者名称（DN 字符串，编码为 PrintableString）。
    pub name: String,
    /// 证书序列号（DER INTEGER 原始字节）。
    pub cert_serial_number: Vec<u8>,
}

impl IssuerAndSerialNumber {
    /// 创建新的签发者信息。
    #[must_use]
    pub fn new(name: impl Into<String>, cert_serial_number: Vec<u8>) -> Self {
        Self {
            name: name.into(),
            cert_serial_number,
        }
    }

    /// 编码为 DER 字节。
    ///
    /// # 错误
    ///
    /// DER 编码不失败，此签名保留以对齐 ofdrw API。
    pub fn to_der(&self) -> DerResult<Vec<u8>> {
        let mut inner = Vec::new();
        encode_printable_string(&self.name, &mut inner);
        // 序列号：已有 DER INTEGER 完整编码则直接拼接，否则视为裸字节包 INTEGER。
        if self.cert_serial_number.first() == Some(&TAG_INTEGER) {
            inner.extend_from_slice(&self.cert_serial_number);
        } else {
            encode_integer_u8(&self.cert_serial_number, &mut inner);
        }
        let mut out = Vec::new();
        encode_sequence(&inner, &mut out);
        Ok(out)
    }

    /// 从 DER 字节解码。
    ///
    /// # 错误
    ///
    /// 输入不是合法的 IssuerAndSerialNumber DER 序列时返回错误。
    pub fn from_der(der: &[u8]) -> DerResult<Self> {
        let (seq, _) = decode_sequence(der, 0)?;
        let (name_val, pos) = expect_tlv(&seq, 0, 0x13)?; // PrintableString
        let name = String::from_utf8_lossy(&name_val).into_owned();
        let (serial, _) = expect_tlv(&seq, pos, TAG_INTEGER)?;
        let mut full = vec![TAG_INTEGER];
        let mut len_buf = Vec::new();
        crate::ses::encode_length(serial.len(), &mut len_buf);
        full.extend_from_slice(&len_buf);
        full.extend_from_slice(&serial);
        Ok(Self::new(name, full))
    }
}

/// 将裸字节包为 DER INTEGER（无符号）。
fn encode_integer_u8(bytes: &[u8], out: &mut Vec<u8>) {
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

/// 序列号便捷构造（无符号大整数，自动处理前导零）。
impl IssuerAndSerialNumber {
    /// 从无符号整数创建序列号。
    #[must_use]
    pub fn from_serial(serial: u64) -> Self {
        let mut bytes = Vec::new();
        encode_integer(serial, &mut bytes);
        Self::new("", bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip() {
        let iasn = IssuerAndSerialNumber::from_serial(12345);
        let der = iasn.to_der().unwrap();
        let decoded = IssuerAndSerialNumber::from_der(&der).unwrap();
        assert_eq!(decoded.cert_serial_number, iasn.cert_serial_number);
        assert_eq!(decoded.name, iasn.name);
    }

    #[test]
    fn test_der_starts_with_sequence() {
        let iasn = IssuerAndSerialNumber::new("CN=Test", vec![TAG_INTEGER, 0x01, 0x2A]);
        let der = iasn.to_der().unwrap();
        assert_eq!(der[0], crate::ses::TAG_SEQUENCE);
        let decoded = IssuerAndSerialNumber::from_der(&der).unwrap();
        assert_eq!(decoded.name, "CN=Test");
    }

    #[test]
    fn test_roundtrip_bare_serial() {
        let iasn = IssuerAndSerialNumber::new("C=CN", vec![0x2A]);
        let der = iasn.to_der().unwrap();
        let decoded = IssuerAndSerialNumber::from_der(&der).unwrap();
        assert_eq!(decoded.name, "C=CN");
    }
}
