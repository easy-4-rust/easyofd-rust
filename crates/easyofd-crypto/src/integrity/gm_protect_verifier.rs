//! 国密保护验证者。
//!
//! 对应 Java: org.ofdrw.crypto.integrity.GMProtectVerifier

use crate::integrity::protect_verifier::ProtectVerifier;

/// 国密（GM）保护验证者，使用 SM2 算法验证签名。
///
/// 对应 Java: `org.ofdrw.crypto.integrity.GMProtectVerifier`
///
/// 使用 SM2 非对称签名算法验证文档签名的正确性。
///
/// 注意：完整实现需要 SM2 签名验证支持。当前版本提供结构定义，
/// SM2 验证委托给 `easyofd-gm` crate。
#[derive(Debug)]
pub struct GmProtectVerifier {
    /// SM2 公钥数据（DER 格式）。
    public_key_der: Vec<u8>,
}

impl GmProtectVerifier {
    /// 创建国密保护验证者。
    ///
    /// 对应 Java: `GMProtectVerifier(Certificate)`
    #[must_use]
    pub fn new(public_key_der: Vec<u8>) -> Self {
        Self { public_key_der }
    }

    /// 获取公钥数据。
    #[must_use]
    pub fn public_key_der(&self) -> &[u8] {
        &self.public_key_der
    }
}

impl ProtectVerifier for GmProtectVerifier {
    fn verify(&self, _data: &[u8], _signature: &[u8]) -> Result<bool, String> {
        // 简化实现：完整实现需要 SM2 签名验证。
        Err(format!(
            "SM2 验证需要集成 easyofd-gm（公钥长度: {} 字节）",
            self.public_key_der.len()
        ))
    }

    fn name(&self) -> &'static str {
        "GmProtectVerifier"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gm_protect_verifier_new() {
        let verifier = GmProtectVerifier::new(vec![0x30; 64]);
        assert_eq!(verifier.public_key_der().len(), 64);
    }

    #[test]
    fn test_gm_protect_verifier_name() {
        let verifier = GmProtectVerifier::new(vec![]);
        assert_eq!(verifier.name(), "GmProtectVerifier");
    }

    #[test]
    fn test_gm_protect_verifier_verify_requires_gm() {
        let verifier = GmProtectVerifier::new(vec![0x30; 64]);
        let result = verifier.verify(b"data", b"sig");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("SM2"));
    }
}
