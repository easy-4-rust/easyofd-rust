//! X.509 证书工具。
//!
//! 对应 Java: org.ofdrw.gm.cert.CertTools
//!
//! 提供 X.509 证书解析、验证和信息提取功能。

use std::time::SystemTime;

use der::Decode;
use x509_cert::Certificate;

/// 证书工具集。
///
/// 对应 Java: org.ofdrw.gm.cert.CertTools
///
/// 提供 X.509 证书解析和验证的工具函数。
pub struct CertTools;

impl CertTools {
    /// 从 DER 编码的证书中提取公钥。
    ///
    /// 对应 Java: org.ofdrw.gm.cert.CertTools#readPublicKey
    ///
    /// 解析 X.509 证书的 SubjectPublicKeyInfo，提取 SM2 公钥。
    /// 返回未压缩格式的公钥（0x04 || X || Y，65 字节）。
    ///
    /// # 参数
    /// - `cert_der`: DER 编码的 X.509 证书
    ///
    /// # 返回
    /// 公钥字节（如果解析成功），格式为未压缩 EC 点（0x04 || X || Y）。
    #[must_use]
    pub fn read_public_key(cert_der: &[u8]) -> Option<Vec<u8>> {
        let cert = Certificate::from_der(cert_der).ok()?;
        let spki = cert.tbs_certificate().subject_public_key_info();
        let key_bytes = spki.subject_public_key.raw_bytes();
        // SM2 未压缩公钥点：0x04 || X || Y（65 字节）
        // 返回完整未压缩点格式
        if key_bytes.is_empty() {
            return None;
        }
        Some(key_bytes.to_vec())
    }

    /// 验证证书是否在有效期内。
    ///
    /// 对应 Java: org.ofdrw.gm.cert.CertTools#checkValidity
    ///
    /// 解析证书的 NotBefore 和 NotAfter 字段，与当前系统时间比较。
    /// 如果证书无法解析，返回 false。
    ///
    /// # 参数
    /// - `cert_der`: DER 编码的 X.509 证书
    #[must_use]
    pub fn check_validity(cert_der: &[u8]) -> bool {
        let cert = match Certificate::from_der(cert_der) {
            Ok(c) => c,
            Err(_) => return false,
        };
        let validity = cert.tbs_certificate().validity();
        let now = SystemTime::now();

        let not_before: SystemTime = validity.not_before.into();
        let not_after: SystemTime = validity.not_after.into();

        now >= not_before && now <= not_after
    }

    /// 从 DER 编码的证书中提取颁发者信息。
    ///
    /// 对应 Java: org.ofdrw.gm.cert.CertTools#getIssuer
    ///
    /// 解析证书的 issuer 字段，返回 RFC 4514 格式的可分辨名称字符串。
    /// 例如："CN=Test Certificate,O=OFD R&W,ST=Zhejiang,L=Hangzhou,C=CN"
    ///
    /// # 参数
    /// - `cert_der`: DER 编码的 X.509 证书
    #[must_use]
    pub fn get_issuer(cert_der: &[u8]) -> String {
        let cert = match Certificate::from_der(cert_der) {
            Ok(c) => c,
            Err(_) => return "Unknown".to_string(),
        };
        format!("{}", cert.tbs_certificate().issuer())
    }

    /// 从 DER 编码的证书中提取使用者信息。
    ///
    /// 对应 Java: org.ofdrw.gm.cert.CertTools#getSubject（扩展方法）
    ///
    /// 解析证书的 subject 字段，返回 RFC 4514 格式的可分辨名称字符串。
    ///
    /// # 参数
    /// - `cert_der`: DER 编码的 X.509 证书
    #[must_use]
    pub fn get_subject(cert_der: &[u8]) -> String {
        let cert = match Certificate::from_der(cert_der) {
            Ok(c) => c,
            Err(_) => return "Unknown".to_string(),
        };
        format!("{}", cert.tbs_certificate().subject())
    }

    /// 从 DER 编码的证书中提取序列号。
    ///
    /// 对应 Java: org.ofdrw.gm.cert.CertTools#getSerialNumber（扩展方法）
    ///
    /// 解析证书的 serialNumber 字段，返回冒号分隔的十六进制字符串。
    /// 例如："01:AB:CD:EF"
    ///
    /// # 参数
    /// - `cert_der`: DER 编码的 X.509 证书
    #[must_use]
    pub fn get_serial_number(cert_der: &[u8]) -> String {
        let cert = match Certificate::from_der(cert_der) {
            Ok(c) => c,
            Err(_) => return "Unknown".to_string(),
        };
        let serial = cert.tbs_certificate().serial_number();
        let bytes = serial.as_bytes();
        if bytes.is_empty() {
            return "00".to_string();
        }
        bytes
            .iter()
            .map(|b| format!("{b:02X}"))
            .collect::<Vec<_>>()
            .join(":")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_public_key_invalid_der() {
        // 非法 DER 数据应返回 None
        assert!(CertTools::read_public_key(&[0x30, 0x03, 0xFF]).is_none());
    }

    #[test]
    fn test_read_public_key_empty() {
        assert!(CertTools::read_public_key(&[]).is_none());
    }

    #[test]
    fn test_check_validity_invalid_der() {
        // 非法 DER 数据应返回 false
        assert!(!CertTools::check_validity(&[0x30, 0x03, 0xFF]));
    }

    #[test]
    fn test_check_validity_empty() {
        assert!(!CertTools::check_validity(&[]));
    }

    #[test]
    fn test_get_issuer_invalid_der() {
        assert_eq!(CertTools::get_issuer(&[0x30, 0x03, 0xFF]), "Unknown");
    }

    #[test]
    fn test_get_issuer_empty() {
        assert_eq!(CertTools::get_issuer(&[]), "Unknown");
    }

    #[test]
    fn test_get_subject_invalid_der() {
        assert_eq!(CertTools::get_subject(&[0x30, 0x03, 0xFF]), "Unknown");
    }

    #[test]
    fn test_get_subject_empty() {
        assert_eq!(CertTools::get_subject(&[]), "Unknown");
    }

    #[test]
    fn test_get_serial_number_invalid_der() {
        assert_eq!(CertTools::get_serial_number(&[0x30, 0x03, 0xFF]), "Unknown");
    }

    #[test]
    fn test_get_serial_number_empty() {
        assert_eq!(CertTools::get_serial_number(&[]), "Unknown");
    }
}
