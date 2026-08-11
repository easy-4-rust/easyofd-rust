//! PEM 文件加载工具。
//!
//! 对应 Java: org.ofdrw.gm.cert.PEMLoader
//!
//! 提供 PEM 格式的编解码功能，支持证书和私钥的 PEM 格式转换。

/// PEM 加载/保存工具。
///
/// 对应 Java: org.ofdrw.gm.cert.PEMLoader
///
/// 提供 PEM 格式的解析和编码功能。
pub struct PemLoader;

impl PemLoader {
    /// 从 PEM 编码的数据中加载 DER 字节。
    ///
    /// 对应 Java: org.ofdrw.gm.cert.PEMLoader#loadCert / #loadPrivateKey
    ///
    /// 解析 PEM 格式的数据，返回 (标签, DER 字节)。
    /// 标签如 "CERTIFICATE"、"PRIVATE KEY" 等。
    ///
    /// # 参数
    /// - `pem_data`: PEM 编码的字符串或字节
    ///
    /// # 返回
    /// `(标签, DER 字节)` 元组，如果解析失败返回 `None`。
    #[must_use]
    pub fn load_pem(pem_data: &[u8]) -> Option<(String, Vec<u8>)> {
        let pem = pem::parse(pem_data).ok()?;
        Some((pem.tag().to_string(), pem.contents().to_vec()))
    }

    /// 将 DER 字节编码为 PEM 格式。
    ///
    /// 对应 Java: PEM 编码操作
    ///
    /// 将 DER 编码的二进制数据转换为 PEM 格式字符串。
    ///
    /// # 参数
    /// - `label`: PEM 标签（如 "CERTIFICATE"、"PRIVATE KEY"）
    /// - `der_bytes`: DER 编码的二进制数据
    ///
    /// # 返回
    /// PEM 编码的字符串。
    #[must_use]
    pub fn to_pem(label: &str, der_bytes: &[u8]) -> String {
        let pem = pem::Pem::new(label, der_bytes.to_vec());
        pem::encode(&pem)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_pem_invalid() {
        assert!(PemLoader::load_pem(b"not a pem").is_none());
    }

    #[test]
    fn test_load_pem_empty() {
        assert!(PemLoader::load_pem(b"").is_none());
    }

    #[test]
    fn test_to_pem_and_load_roundtrip() {
        let der_bytes = vec![0x30, 0x03, 0x02, 0x01, 0x01]; // 最小 SEQUENCE
        let pem_str = PemLoader::to_pem("TEST DATA", &der_bytes);
        assert!(pem_str.contains("-----BEGIN TEST DATA-----"));
        assert!(pem_str.contains("-----END TEST DATA-----"));

        let (label, loaded_der) = PemLoader::load_pem(pem_str.as_bytes()).unwrap();
        assert_eq!(label, "TEST DATA");
        assert_eq!(loaded_der, der_bytes);
    }

    #[test]
    fn test_to_pem_certificate() {
        let der_bytes = vec![0x30, 0x82, 0x01, 0x00]; // 模拟证书 DER
        let pem_str = PemLoader::to_pem("CERTIFICATE", &der_bytes);
        assert!(pem_str.contains("BEGIN CERTIFICATE"));

        let (label, loaded_der) = PemLoader::load_pem(pem_str.as_bytes()).unwrap();
        assert_eq!(label, "CERTIFICATE");
        assert_eq!(loaded_der, der_bytes);
    }

    #[test]
    fn test_to_pem_private_key() {
        let der_bytes = vec![0x30, 0x03, 0x02, 0x01, 0x00]; // 模拟私钥 DER
        let pem_str = PemLoader::to_pem("PRIVATE KEY", &der_bytes);
        assert!(pem_str.contains("BEGIN PRIVATE KEY"));

        let (label, loaded_der) = PemLoader::load_pem(pem_str.as_bytes()).unwrap();
        assert_eq!(label, "PRIVATE KEY");
        assert_eq!(loaded_der, der_bytes);
    }
}
