//! 保护验证者接口。
//!
//! 对应 Java: org.ofdrw.crypto.integrity.ProtectVerifier

/// 保护验证者接口，用于验证 OFD 文档完整性保护的签名。
///
/// 对应 Java: `org.ofdrw.crypto.integrity.ProtectVerifier`
///
/// 验证者使用签名者的公钥验证文档签名的正确性。
pub trait ProtectVerifier: std::fmt::Debug {
    /// 验证签名。
    ///
    /// # 参数
    ///
    /// - `data`: 原始数据（通常是文档内容的哈希值）。
    /// - `signature`: 待验证的签名值。
    ///
    /// # 返回
    ///
    /// `true` 表示签名有效，`false` 表示签名无效。
    ///
    /// # 错误
    ///
    /// 验证过程出错时返回错误描述字符串。
    #[allow(clippy::cast_possible_truncation)]
    fn verify(&self, data: &[u8], signature: &[u8]) -> Result<bool, String>;

    /// 获取验证者名称。
    fn name(&self) -> &'static str;
}

/// 基于 HMAC 的简单验证者。
///
/// 简化实现，与 [`super::protect_signer::SimpleSigner`] 配对使用。
#[derive(Debug)]
pub struct SimpleVerifier {
    /// 验证密钥（与签名密钥相同）。
    key: Vec<u8>,
}

impl SimpleVerifier {
    /// 创建简单验证者。
    #[must_use]
    pub fn new(key: Vec<u8>) -> Self {
        Self { key }
    }
}

impl ProtectVerifier for SimpleVerifier {
    #[allow(clippy::cast_possible_truncation)]
    fn verify(&self, data: &[u8], signature: &[u8]) -> Result<bool, String> {
        // 使用与 SimpleSigner 相同的逻辑重新计算签名
        let mut sig_data = Vec::with_capacity(data.len() + self.key.len());
        sig_data.extend_from_slice(&self.key);
        sig_data.extend_from_slice(data);
        let mut hash = [0u8; 32];
        for (i, &byte) in sig_data.iter().enumerate() {
            hash[i % 32] ^= byte.wrapping_add(i as u8);
        }
        Ok(hash.as_slice() == signature)
    }

    fn name(&self) -> &'static str {
        "SimpleVerifier"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::integrity::protect_signer::{ProtectSigner, SimpleSigner};

    #[test]
    fn test_simple_verifier_valid() {
        let key = vec![0xAB; 16];
        let signer = SimpleSigner::new(key.clone());
        let verifier = SimpleVerifier::new(key);

        let data = b"test data";
        let signature = signer.sign(data).unwrap();
        assert!(verifier.verify(data, &signature).unwrap());
    }

    #[test]
    fn test_simple_verifier_invalid() {
        let key = vec![0xAB; 16];
        let verifier = SimpleVerifier::new(key);
        let data = b"test data";
        let wrong_signature = vec![0xFF; 32];
        assert!(!verifier.verify(data, &wrong_signature).unwrap());
    }

    #[test]
    fn test_simple_verifier_name() {
        let verifier = SimpleVerifier::new(vec![]);
        assert_eq!(verifier.name(), "SimpleVerifier");
    }
}
