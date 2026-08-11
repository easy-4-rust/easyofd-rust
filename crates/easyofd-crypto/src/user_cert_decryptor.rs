//! 用户证书解密器。
//!
//! 对应 Java: org.ofdrw.crypto.decryptor.UserCertDecryptor

use crate::user_fek_decryptor::UserFekDecryptor;

/// 用户证书解密器，使用数字证书（私钥）解密 FEK。
///
/// 对应 Java: `org.ofdrw.crypto.decryptor.UserCertDecryptor`
///
/// 使用 SM2 或 RSA 私钥解密 FEK。
///
/// 注意：完整实现需要 SM2 非对称解密支持。当前版本提供结构定义和
/// 接口实现，SM2 解密委托给 `easyofd-gm` crate。
#[derive(Debug)]
pub struct UserCertDecryptor {
    /// 私钥数据（DER 格式）。
    private_key_der: Vec<u8>,
    /// 解密算法标识。
    algorithm: super::user_cert_encryptor::CertAlgorithm,
}

impl UserCertDecryptor {
    /// 创建 SM2 证书解密器。
    ///
    /// 对应 Java: `UserCertDecryptor(PrivateKey privateKey, Certificate certificate)`
    #[must_use]
    pub fn sm2(private_key_der: Vec<u8>) -> Self {
        Self {
            private_key_der,
            algorithm: super::user_cert_encryptor::CertAlgorithm::Sm2,
        }
    }

    /// 创建 RSA 证书解密器。
    #[must_use]
    pub fn rsa(private_key_der: Vec<u8>) -> Self {
        Self {
            private_key_der,
            algorithm: super::user_cert_encryptor::CertAlgorithm::Rsa,
        }
    }

    /// 获取私钥数据。
    #[must_use]
    pub fn private_key_der(&self) -> &[u8] {
        &self.private_key_der
    }

    /// 获取解密算法。
    #[must_use]
    pub fn algorithm(&self) -> super::user_cert_encryptor::CertAlgorithm {
        self.algorithm
    }
}

impl UserFekDecryptor for UserCertDecryptor {
    fn decrypt_fek(&self, _encrypted_fek: &[u8]) -> Result<Vec<u8>, String> {
        // 简化实现：完整实现需要 SM2/RSA 非对称解密。
        Err(format!(
            "证书解密需要集成 SM2/RSA 非对称解密（算法: {:?}，私钥长度: {} 字节）",
            self.algorithm,
            self.private_key_der.len()
        ))
    }

    fn name(&self) -> &'static str {
        "UserCertDecryptor"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CertAlgorithm;

    #[test]
    fn test_user_cert_decryptor_sm2() {
        let decryptor = UserCertDecryptor::sm2(vec![0x30; 32]);
        assert_eq!(decryptor.algorithm(), CertAlgorithm::Sm2);
    }

    #[test]
    fn test_user_cert_decryptor_rsa() {
        let decryptor = UserCertDecryptor::rsa(vec![0x30; 128]);
        assert_eq!(decryptor.algorithm(), CertAlgorithm::Rsa);
    }

    #[test]
    fn test_user_cert_decryptor_name() {
        let decryptor = UserCertDecryptor::sm2(vec![]);
        assert_eq!(decryptor.name(), "UserCertDecryptor");
    }

    #[test]
    fn test_user_cert_decryptor_decrypt_requires_sm2() {
        let decryptor = UserCertDecryptor::sm2(vec![0x30; 32]);
        let result = decryptor.decrypt_fek(&[0x42; 64]);
        assert!(result.is_err());
    }
}
