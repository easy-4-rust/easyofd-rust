//! OFD 文档完整性验证。
//!
//! 对应 Java: org.ofdrw.crypto.integrity.OFDIntegrityVerifier

use crate::integrity::protect_verifier::ProtectVerifier;

/// OFD 文档完整性验证器，验证加密 OFD 文档的签名完整性。
///
/// 对应 Java: `org.ofdrw.crypto.integrity.OFDIntegrityVerifier`
#[derive(Debug)]
pub struct OfdIntegrityVerifier {
    /// 验证者。
    verifier: Box<dyn ProtectVerifier>,
}

impl OfdIntegrityVerifier {
    /// 创建 OFD 完整性验证器。
    ///
    /// 对应 Java: `OFDIntegrityVerifier(ProtectVerifier)`
    #[must_use]
    pub fn new(verifier: Box<dyn ProtectVerifier>) -> Self {
        Self { verifier }
    }

    /// 验证数据的签名完整性。
    ///
    /// 对应 Java: `OFDIntegrityVerifier.verify(byte[], byte[])`
    ///
    /// # 参数
    ///
    /// - `data`: 原始数据。
    /// - `signature`: 签名值。
    ///
    /// # 返回
    ///
    /// `true` 表示签名有效。
    pub fn verify(&self, data: &[u8], signature: &[u8]) -> Result<bool, String> {
        self.verifier.verify(data, signature)
    }

    /// 获取验证者名称。
    #[must_use]
    pub fn verifier_name(&self) -> &str {
        self.verifier.name()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::integrity::protect_signer::{ProtectSigner, SimpleSigner};
    use crate::integrity::protect_verifier::SimpleVerifier;

    #[test]
    fn test_ofd_integrity_verifier_valid() {
        let key = vec![0xAB; 16];
        let signer = SimpleSigner::new(key.clone());
        let verifier = SimpleVerifier::new(key);

        let data = b"test data";
        let signature = signer.sign(data).unwrap();

        let ofd_verifier = OfdIntegrityVerifier::new(Box::new(verifier));
        assert!(ofd_verifier.verify(data, &signature).unwrap());
    }

    #[test]
    fn test_ofd_integrity_verifier_invalid() {
        let verifier = SimpleVerifier::new(vec![0xAB; 16]);
        let ofd_verifier = OfdIntegrityVerifier::new(Box::new(verifier));
        assert!(!ofd_verifier.verify(b"data", &[0xFF; 32]).unwrap());
    }

    #[test]
    fn test_ofd_integrity_verifier_name() {
        let verifier = SimpleVerifier::new(vec![]);
        let ofd_verifier = OfdIntegrityVerifier::new(Box::new(verifier));
        assert_eq!(ofd_verifier.verifier_name(), "SimpleVerifier");
    }
}
