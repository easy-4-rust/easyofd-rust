//! 国密保护签名者。
//!
//! 对应 Java: org.ofdrw.crypto.integrity.GMProtectSigner

use crate::integrity::protect_signer::ProtectSigner;

/// 国密（GM）保护签名者，使用 SM2 算法进行签名。
///
/// 对应 Java: `org.ofdrw.crypto.integrity.GMProtectSigner`
///
/// 使用 SM2 非对称签名算法对文档哈希值进行签名。
///
/// 注意：完整实现需要 SM2 签名支持。当前版本提供结构定义，
/// SM2 签名委托给 `easyofd-gm` crate。
#[derive(Debug)]
pub struct GmProtectSigner {
    /// SM2 私钥数据（DER 格式）。
    private_key_der: Vec<u8>,
    /// 证书数据（DER 格式，可选）。
    cert_der: Option<Vec<u8>>,
}

impl GmProtectSigner {
    /// 创建国密保护签名者。
    ///
    /// 对应 Java: `GMProtectSigner(PrivateKey, Certificate)`
    #[must_use]
    pub fn new(private_key_der: Vec<u8>, cert_der: Option<Vec<u8>>) -> Self {
        Self {
            private_key_der,
            cert_der,
        }
    }

    /// 获取私钥数据。
    #[must_use]
    pub fn private_key_der(&self) -> &[u8] {
        &self.private_key_der
    }

    /// 获取证书数据。
    #[must_use]
    pub fn cert_der(&self) -> Option<&[u8]> {
        self.cert_der.as_deref()
    }
}

impl ProtectSigner for GmProtectSigner {
    fn sign(&self, _data: &[u8]) -> Result<Vec<u8>, String> {
        // 简化实现：完整实现需要 SM2 签名。
        Err(format!(
            "SM2 签名需要集成 easyofd-gm（私钥长度: {} 字节）",
            self.private_key_der.len()
        ))
    }

    fn name(&self) -> &'static str {
        "GmProtectSigner"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gm_protect_signer_new() {
        let signer = GmProtectSigner::new(vec![0x30; 32], Some(vec![0x30; 64]));
        assert_eq!(signer.private_key_der().len(), 32);
        assert_eq!(signer.cert_der().unwrap().len(), 64);
    }

    #[test]
    fn test_gm_protect_signer_no_cert() {
        let signer = GmProtectSigner::new(vec![0x30; 32], None);
        assert!(signer.cert_der().is_none());
    }

    #[test]
    fn test_gm_protect_signer_name() {
        let signer = GmProtectSigner::new(vec![], None);
        assert_eq!(signer.name(), "GmProtectSigner");
    }

    #[test]
    fn test_gm_protect_signer_sign_requires_gm() {
        let signer = GmProtectSigner::new(vec![0x30; 32], None);
        let result = signer.sign(b"test");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("SM2"));
    }
}
