//! OFD 文档完整性保护。
//!
//! 对应 Java: org.ofdrw.crypto.integrity.OFDIntegrity

use crate::integrity::protect_signer::ProtectSigner;

/// OFD 文档完整性保护，对加密后的 OFD 文档进行签名保护。
///
/// 对应 Java: `org.ofdrw.crypto.integrity.OFDIntegrity`
///
/// 完整性保护流程：
/// 1. 加密 OFD 文档
/// 2. 对加密后的文档内容计算哈希
/// 3. 使用签名者对哈希进行签名
/// 4. 将签名值存入文档
#[derive(Debug)]
pub struct OfdIntegrity {
    /// 签名者。
    signer: Box<dyn ProtectSigner>,
}

impl OfdIntegrity {
    /// 创建 OFD 完整性保护实例。
    ///
    /// 对应 Java: `OFDIntegrity(ProtectSigner)`
    #[must_use]
    pub fn new(signer: Box<dyn ProtectSigner>) -> Self {
        Self { signer }
    }

    /// 对数据进行签名保护。
    ///
    /// 对应 Java: `OFDIntegrity.protect(byte[])`
    ///
    /// # 参数
    ///
    /// - `data`: 待保护的数据。
    ///
    /// # 返回
    ///
    /// 签名值。
    ///
    /// # 错误
    ///
    /// 签名失败时返回错误描述。
    pub fn protect(&self, data: &[u8]) -> Result<Vec<u8>, String> {
        self.signer.sign(data)
    }

    /// 获取签名者名称。
    #[must_use]
    pub fn signer_name(&self) -> &str {
        self.signer.name()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::integrity::protect_signer::SimpleSigner;

    #[test]
    fn test_ofd_integrity_protect() {
        let signer = SimpleSigner::new(vec![0xAB; 16]);
        let integrity = OfdIntegrity::new(Box::new(signer));
        let data = b"encrypted ofd content";
        let signature = integrity.protect(data).unwrap();
        assert_eq!(signature.len(), 32);
    }

    #[test]
    fn test_ofd_integrity_signer_name() {
        let signer = SimpleSigner::new(vec![0xAB; 16]);
        let integrity = OfdIntegrity::new(Box::new(signer));
        assert_eq!(integrity.signer_name(), "SimpleSigner");
    }
}
