//! IssuerAndSerialNumber 结构。
//!
//! 对应 Java: org.ofdrw.gm.sm2strut.IssuerAndSerialNumber
//!
//! GB/T 35275-2017 6.7 IssuerAndSerialNumber:
//! ```asn1
//! IssuerAndSerialNumber ::= SEQUENCE {
//!     issuer        Name,
//!     serialNumber  CertificateSerialNumber
//! }
//! ```
//!
//! 其中 `issuer` 为 X.500 Name 的完整 DER（RDNSequence），`serialNumber` 为 INTEGER。
//! 与 Java 侧 `IssuerAndSerialNumber(X500Name, ASN1Integer)` 一致。

use der::Decode;

use crate::ses::{DerResult, TAG_INTEGER, decode_sequence, decode_tlv};
use crate::ses::{encode_length, encode_sequence};

/// 证书签发者名称与序列号（PKCS#7 SignerInfo 的 sid）。
///
/// 对应 Java: ofdrw `IssuerAndSerialNumber`。
///
/// DER 布局：`SEQUENCE { issuer Name, certSerialNumber INTEGER }`。
///
/// - `issuer_der`：X.500 Name 的完整 DER 字节（包含 SEQUENCE tag + length + RDN 内容），
///   可从 `x509_cert::name::Name::to_der()` 获取，与 Java `X500Principal.getEncoded()` 一致。
/// - `cert_serial_number`：证书序列号的裸字节（无 INTEGER tag），即 `SerialNumber::as_bytes()`。
///   编码时由 [`Self::to_der`] 自动包装为 DER INTEGER。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IssuerAndSerialNumber {
    /// 签发者名称（DN 字符串），仅供向后兼容。
    ///
    /// **已废弃**：请使用 [`Self::issuer_der`] 获取 X.500 Name 的完整 DER。
    #[deprecated(
        since = "1.0.0",
        note = "请使用 `issuer_der` 字段获取 X.500 Name 的完整 DER 字节。仅当从旧格式 PrintableString 解码时填充。"
    )]
    pub name: String,
    /// 签发者 Name 的完整 DER 字节（包含 SEQUENCE tag + length + RDN 内容）。
    ///
    /// 对应 Java: `X500Name.getEncoded("DER")` 或 `X500Principal.getEncoded()`。
    /// 可通过 `x509_cert::name::Name::to_der()` 获取。
    pub issuer_der: Vec<u8>,
    /// 证书序列号裸字节（不含 INTEGER tag/length），即 `SerialNumber::as_bytes()`。
    ///
    /// 编码时由 [`Self::to_der`] 自动包装为 DER INTEGER。
    pub cert_serial_number: Vec<u8>,
}

impl IssuerAndSerialNumber {
    /// 从 X.500 Name DER 字节和序列号裸字节创建。
    ///
    /// 对应 Java: `IssuerAndSerialNumber(X500Name name, ASN1Integer certSerialNumber)`
    ///
    /// # 参数
    /// - `issuer_der`：Name 的完整 DER 字节（含 SEQUENCE tag），通常由
    ///   `x509_cert::name::Name::to_der()` 生成。
    /// - `cert_serial_number`：序列号裸字节（不含 INTEGER tag），即
    ///   `x509_cert::serial_number::SerialNumber::as_bytes()`。
    #[must_use]
    pub fn new(issuer_der: Vec<u8>, cert_serial_number: Vec<u8>) -> Self {
        Self {
            #[allow(deprecated)]
            name: String::new(),
            issuer_der,
            cert_serial_number,
        }
    }

    /// 从 X.509 证书 DER 中提取签发者和序列号。
    ///
    /// 对应 Java: `IssuerAndSerialNumber(certificate.getIssuer(), certificate.getSerialNumber())`
    ///
    /// # 参数
    /// - `cert_der`：DER 编码的 X.509 证书。
    ///
    /// # Panics
    /// 如果证书 DER 无法解析，则 panic。生产环境应确保证书有效。
    #[must_use]
    pub fn from_certificate_der(cert_der: &[u8]) -> Self {
        use der::Encode as _;

        let cert = x509_cert::Certificate::from_der(cert_der)
            .expect("证书 DER 解析失败，请提供有效的 X.509 证书");

        let issuer_der = cert
            .tbs_certificate()
            .issuer()
            .to_der()
            .expect("issuer Name DER 编码失败");

        let serial_bytes = cert.tbs_certificate().serial_number().as_bytes().to_vec();

        Self::new(issuer_der, serial_bytes)
    }

    /// 以兼容旧格式的 PrintableString 方式创建（测试/向后兼容）。
    ///
    /// **已废弃**：请使用 [`Self::new`] 或 [`Self::from_certificate_der`]。
    #[deprecated(
        since = "1.0.0",
        note = "请使用 `IssuerAndSerialNumber::new(issuer_der, serial)` 或 `from_certificate_der(cert_der)`。"
    )]
    #[must_use]
    pub fn with_name_string(name: impl Into<String>, cert_serial_number: Vec<u8>) -> Self {
        let name_str = name.into();
        // 将 name 字符串编码为 PrintableString DER，存入 issuer_der。
        let mut issuer_der = Vec::new();
        crate::ses::encode_printable_string(&name_str, &mut issuer_der);
        Self {
            #[allow(deprecated)]
            name: name_str,
            issuer_der,
            cert_serial_number,
        }
    }

    /// 从无符号整数创建序列号（测试/向后兼容）。
    ///
    /// **已废弃**：请使用 [`Self::new`] 或 [`Self::from_certificate_der`]。
    ///
    /// # Panics
    /// 如果序列号值大于 `u64::MAX`。
    #[deprecated(
        since = "1.0.0",
        note = "请使用 `IssuerAndSerialNumber::new(issuer_der, serial)` 或 `from_certificate_der(cert_der)`。"
    )]
    #[must_use]
    pub fn from_serial(serial: u64) -> Self {
        let bytes = serial.to_be_bytes();
        let significant = bytes
            .iter()
            .position(|&b| b != 0)
            .unwrap_or(bytes.len() - 1);
        let serial_bytes = bytes[significant..].to_vec();
        Self {
            #[allow(deprecated)]
            name: String::new(),
            issuer_der: Vec::new(),
            cert_serial_number: serial_bytes,
        }
    }

    /// 编码为 DER 字节。
    ///
    /// 布局：`SEQUENCE { issuer Name-DER, serialNumber INTEGER }`。
    ///
    /// # 错误
    ///
    /// 当 `issuer_der` 为空（来自废弃的 `from_serial` 占位）时，
    /// 用 PrintableString("") 兜底以保持编解码兼容。
    pub fn to_der(&self) -> DerResult<Vec<u8>> {
        let mut inner = Vec::new();

        if self.issuer_der.is_empty() {
            // 向后兼容：旧 from_serial() 构造的空 issuer 用 PrintableString("") 兜底。
            crate::ses::encode_printable_string("", &mut inner);
        } else {
            // issuer_der 为完整的 Name DER（含 SEQUENCE tag+length+content），直接拼入。
            inner.extend_from_slice(&self.issuer_der);
        }

        // 序列号：裸字节包 INTEGER。
        encode_integer_raw(&self.cert_serial_number, &mut inner);

        let mut out = Vec::new();
        encode_sequence(&inner, &mut out);
        Ok(out)
    }

    /// 从 DER 字节解码。
    ///
    /// 支持两种 issuer 格式：
    /// - `0x30` SEQUENCE：完整的 X.500 Name DER（真实证书）。
    /// - `0x13` PrintableString：旧格式兼容（`with_name_string` / `from_serial`）。
    ///
    /// # 错误
    ///
    /// 输入不是合法的 IssuerAndSerialNumber DER 序列时返回错误。
    pub fn from_der(der: &[u8]) -> DerResult<Self> {
        let (seq, _) = decode_sequence(der, 0)?;

        // 解码 issuer：可能是 SEQUENCE(0x30) 或 PrintableString(0x13)。
        let (issuer_tag, issuer_val, pos) = decode_tlv(&seq, 0)?;

        // 重建完整的 issuer DER TLV。
        let mut issuer_der_full = Vec::new();
        issuer_der_full.push(issuer_tag);
        crate::ses::encode_length(issuer_val.len(), &mut issuer_der_full);
        issuer_der_full.extend_from_slice(&issuer_val);

        // 向后兼容：如果 issuer 是 PrintableString，填充 name 字段。
        let name = if issuer_tag == crate::ses::TAG_PRINTABLE_STRING {
            String::from_utf8_lossy(&issuer_val).into_owned()
        } else {
            String::new()
        };

        // 解码序列号 INTEGER。
        let (serial_val, _end) = crate::ses::expect_tlv(&seq, pos, crate::ses::TAG_INTEGER)?;
        // serial_val 是 INTEGER 的裸字节值。

        Ok(Self {
            #[allow(deprecated)]
            name,
            issuer_der: issuer_der_full,
            cert_serial_number: serial_val,
        })
    }
}

/// 将裸字节包为 DER INTEGER（无符号）。
///
/// 去除前导零后编码，若最高位为 1 则添加前导零字节以表示正数。
/// 与旧版 `encode_integer_u8` 逻辑一致。
fn encode_integer_raw(bytes: &[u8], out: &mut Vec<u8>) {
    if bytes.is_empty() {
        // 空序列号编码为 INTEGER 0。
        out.push(TAG_INTEGER);
        out.push(0x01);
        out.push(0x00);
        return;
    }
    let significant = bytes
        .iter()
        .position(|&b| b != 0)
        .unwrap_or(bytes.len() - 1);
    let payload = &bytes[significant..];
    let needs_zero = !payload.is_empty() && (payload[0] & 0x80 != 0);
    out.push(TAG_INTEGER);
    encode_length(payload.len() + usize::from(needs_zero), out);
    if needs_zero {
        out.push(0x00);
    }
    out.extend_from_slice(payload);
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── 向后兼容测试（from_serial + PrintableString） ──────────────────────

    #[test]
    fn test_roundtrip_from_serial() {
        #[allow(deprecated)]
        let iasn = IssuerAndSerialNumber::from_serial(12345);
        let der = iasn.to_der().unwrap();
        let decoded = IssuerAndSerialNumber::from_der(&der).unwrap();
        assert_eq!(decoded.cert_serial_number, iasn.cert_serial_number);
    }

    #[test]
    fn test_roundtrip_printable_string() {
        #[allow(deprecated)]
        let iasn = IssuerAndSerialNumber::with_name_string("CN=Test", vec![0x2A]);
        let der = iasn.to_der().unwrap();
        assert_eq!(der[0], crate::ses::TAG_SEQUENCE);
        let decoded = IssuerAndSerialNumber::from_der(&der).unwrap();
        #[allow(deprecated)]
        {
            assert_eq!(decoded.name, "CN=Test");
        }
        assert_eq!(decoded.cert_serial_number, vec![0x2A]);
    }

    #[test]
    fn test_roundtrip_bare_serial() {
        #[allow(deprecated)]
        let iasn = IssuerAndSerialNumber::with_name_string("C=CN", vec![0x2A]);
        let der = iasn.to_der().unwrap();
        let decoded = IssuerAndSerialNumber::from_der(&der).unwrap();
        #[allow(deprecated)]
        {
            assert_eq!(decoded.name, "C=CN");
        }
    }

    // ── 新 API 测试（issuer_der + 真实 Name DER） ──────────────────────────

    #[test]
    fn test_new_with_real_name_der_roundtrip() {
        // 构造一个最小的 SEQUENCE 作为 issuer_der 模拟 X.500 Name。
        // SEQUENCE { SET { SEQUENCE { OID 2.5.4.3, PrintableString "Test" } } }
        let name_content = {
            let mut oid_buf = Vec::new();
            crate::ses::encode_oid(&[2, 5, 4, 3], &mut oid_buf);
            let mut ps_buf = Vec::new();
            crate::ses::encode_printable_string("Test", &mut ps_buf);
            let mut attr_seq = Vec::new();
            attr_seq.extend_from_slice(&oid_buf);
            attr_seq.extend_from_slice(&ps_buf);
            let mut attr_der = Vec::new();
            crate::ses::encode_sequence(&attr_seq, &mut attr_der);
            let mut set_body = Vec::new();
            set_body.extend_from_slice(&attr_der);
            let mut set_tlv = Vec::new();
            set_tlv.push(crate::ses::TAG_SET);
            crate::ses::encode_length(set_body.len(), &mut set_tlv);
            set_tlv.extend_from_slice(&set_body);
            set_tlv
        };
        let mut issuer_der = Vec::new();
        crate::ses::encode_sequence(&name_content, &mut issuer_der);

        let iasn = IssuerAndSerialNumber::new(issuer_der.clone(), vec![0x01, 0x00]);
        let der = iasn.to_der().unwrap();
        assert_eq!(der[0], crate::ses::TAG_SEQUENCE);

        let decoded = IssuerAndSerialNumber::from_der(&der).unwrap();
        assert_eq!(decoded.issuer_der, issuer_der);
        assert_eq!(decoded.cert_serial_number, vec![0x01, 0x00]);
    }

    // ── from_certificate_der 测试 ──────────────────────────────────────────

    #[test]
    fn test_from_certificate_der_roundtrip() {
        let cert_der = crate::pkc_generate::PkcGenerate::generate_self_signed("CN=Test,O=TestOrg");
        let iasn = IssuerAndSerialNumber::from_certificate_der(&cert_der);

        // issuer_der 不为空，且以 SEQUENCE tag 开头。
        assert!(!iasn.issuer_der.is_empty());
        assert_eq!(iasn.issuer_der[0], 0x30);

        // cert_serial_number 不为空。
        assert!(!iasn.cert_serial_number.is_empty());

        // 往返测试。
        let der = iasn.to_der().unwrap();
        let decoded = IssuerAndSerialNumber::from_der(&der).unwrap();
        assert_eq!(decoded.issuer_der, iasn.issuer_der);
        assert_eq!(decoded.cert_serial_number, iasn.cert_serial_number);
    }

    #[test]
    fn test_from_certificate_der_with_known_serial() {
        let cert_der =
            crate::pkc_generate::PkcGenerate::generate_self_signed_with_serial("CN=Test", 42);
        let iasn = IssuerAndSerialNumber::from_certificate_der(&cert_der);
        // 序列号 42 → [0x2A]
        assert_eq!(iasn.cert_serial_number, vec![0x2A]);
    }

    #[test]
    fn test_from_certificate_der_name_der_matches_x509_crate() {
        use der::Encode as _;

        let cert_der =
            crate::pkc_generate::PkcGenerate::generate_self_signed("CN=Issuer Test,C=CN");
        let cert = x509_cert::Certificate::from_der(&cert_der).unwrap();
        let expected_issuer_der = cert.tbs_certificate().issuer().to_der().unwrap();

        let iasn = IssuerAndSerialNumber::from_certificate_der(&cert_der);
        assert_eq!(iasn.issuer_der, expected_issuer_der);
    }

    // ── encode_integer_u8 边界测试 ─────────────────────────────────────────

    #[test]
    fn test_serial_zero() {
        let iasn = IssuerAndSerialNumber::new(vec![0x30, 0x00], vec![0x00]);
        let der = iasn.to_der().unwrap();
        let decoded = IssuerAndSerialNumber::from_der(&der).unwrap();
        assert_eq!(decoded.cert_serial_number, vec![0x00]);
    }

    #[test]
    fn test_serial_256() {
        let iasn = IssuerAndSerialNumber::new(vec![0x30, 0x00], vec![0x01, 0x00]);
        let der = iasn.to_der().unwrap();
        let decoded = IssuerAndSerialNumber::from_der(&der).unwrap();
        assert_eq!(decoded.cert_serial_number, vec![0x01, 0x00]);
    }

    #[test]
    fn test_serial_high_bit_set() {
        // 序列号 0x80 编码为 DER INTEGER 时需要前导零字节（保持正数语义），
        // 解码后裸字节包含该前导零 → [0x00, 0x80]。
        let iasn = IssuerAndSerialNumber::new(vec![0x30, 0x00], vec![0x80]);
        let der = iasn.to_der().unwrap();
        assert_eq!(der[0], crate::ses::TAG_SEQUENCE);
        let decoded = IssuerAndSerialNumber::from_der(&der).unwrap();
        assert_eq!(decoded.cert_serial_number, vec![0x00, 0x80]);
    }
}
