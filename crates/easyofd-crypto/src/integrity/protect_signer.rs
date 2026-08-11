//! 保护签名者接口。
//!
//! 对应 Java: org.ofdrw.crypto.integrity.ProtectSigner

/// 保护签名者接口，用于对 OFD 文档的完整性保护进行签名。
///
/// 对应 Java: `org.ofdrw.crypto.integrity.ProtectSigner`
///
/// 在 OFD 文档加密后，对加密内容进行签名以保证完整性。
/// 签名者使用自己的私钥对文档哈希值进行签名。
pub trait ProtectSigner: std::fmt::Debug {
    /// 对数据进行签名。
    ///
    /// # 参数
    ///
    /// - `data`: 待签名的数据（通常是文档内容的哈希值）。
    ///
    /// # 返回
    ///
    /// 签名值。
    ///
    /// # 错误
    ///
    /// 签名失败时返回错误描述字符串。
    #[allow(clippy::cast_possible_truncation)]
    fn sign(&self, data: &[u8]) -> Result<Vec<u8>, String>;

    /// 获取签名者名称。
    fn name(&self) -> &'static str;
}

/// 基于 HMAC 的简单签名者。
///
/// 简化实现，用于测试或不需要国密签名的场景。
#[derive(Debug)]
pub struct SimpleSigner {
    /// 签名密钥。
    key: Vec<u8>,
}

impl SimpleSigner {
    /// 创建简单签名者。
    #[must_use]
    pub fn new(key: Vec<u8>) -> Self {
        Self { key }
    }
}

impl ProtectSigner for SimpleSigner {
    #[allow(clippy::cast_possible_truncation)]
    fn sign(&self, data: &[u8]) -> Result<Vec<u8>, String> {
        // 简化实现：HMAC-like 签名（生产环境应使用 SM2 签名）
        let mut signature = Vec::with_capacity(data.len() + self.key.len());
        signature.extend_from_slice(&self.key);
        signature.extend_from_slice(data);
        // 简单哈希折叠
        let mut hash = [0u8; 32];
        for (i, &byte) in signature.iter().enumerate() {
            hash[i % 32] ^= byte.wrapping_add(i as u8);
        }
        Ok(hash.to_vec())
    }

    fn name(&self) -> &'static str {
        "SimpleSigner"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_signer() {
        let signer = SimpleSigner::new(vec![0xAB; 16]);
        let data = b"test data to sign";
        let signature = signer.sign(data).unwrap();
        assert_eq!(signature.len(), 32);
    }

    #[test]
    fn test_simple_signer_deterministic() {
        let signer = SimpleSigner::new(vec![0xAB; 16]);
        let data = b"same data";
        let sig1 = signer.sign(data).unwrap();
        let sig2 = signer.sign(data).unwrap();
        assert_eq!(sig1, sig2);
    }

    #[test]
    fn test_simple_signer_different_data() {
        let signer = SimpleSigner::new(vec![0xAB; 16]);
        let sig1 = signer.sign(b"data1").unwrap();
        let sig2 = signer.sign(b"data2").unwrap();
        assert_ne!(sig1, sig2);
    }

    #[test]
    fn test_simple_signer_name() {
        let signer = SimpleSigner::new(vec![]);
        assert_eq!(signer.name(), "SimpleSigner");
    }
}
