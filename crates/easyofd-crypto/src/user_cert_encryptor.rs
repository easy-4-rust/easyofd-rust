//! 用户证书加密器。
//!
//! 对应 Java: org.ofdrw.crypto.enryptor.UserCertEncryptor

use crate::user_fek_encryptor::UserFekEncryptor;

/// 用户证书加密器，使用数字证书（公钥）保护 FEK。
///
/// 对应 Java: `org.ofdrw.crypto.enryptor.UserCertEncryptor`
///
/// 使用 SM2 或 RSA 公钥加密 FEK，只有持有对应私钥的用户才能解密。
///
/// 注意：完整实现需要 SM2 非对称加密支持。当前版本提供结构定义和
/// 接口实现，SM2 加密委托给 `easyofd-gm` crate。
#[derive(Debug)]
pub struct UserCertEncryptor {
    /// 证书的公钥数据（DER 格式）。
    public_key_der: Vec<u8>,
    /// 加密算法标识。
    algorithm: CertAlgorithm,
}

/// 证书加密算法。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CertAlgorithm {
    /// SM2 算法（国密）。
    Sm2,
    /// RSA 算法。
    Rsa,
}

impl UserCertEncryptor {
    /// 创建 SM2 证书加密器。
    ///
    /// 对应 Java: `UserCertEncryptor(Certificate certificate)`
    #[must_use]
    pub fn sm2(public_key_der: Vec<u8>) -> Self {
        Self {
            public_key_der,
            algorithm: CertAlgorithm::Sm2,
        }
    }

    /// 创建 RSA 证书加密器。
    #[must_use]
    pub fn rsa(public_key_der: Vec<u8>) -> Self {
        Self {
            public_key_der,
            algorithm: CertAlgorithm::Rsa,
        }
    }

    /// 获取公钥数据。
    #[must_use]
    pub fn public_key_der(&self) -> &[u8] {
        &self.public_key_der
    }

    /// 获取加密算法。
    #[must_use]
    pub fn algorithm(&self) -> CertAlgorithm {
        self.algorithm
    }
}

impl UserFekEncryptor for UserCertEncryptor {
    fn encrypt_fek(&self, _fek: &[u8]) -> Result<Vec<u8>, String> {
        // 简化实现：完整实现需要 SM2/RSA 非对称加密。
        // 当前返回错误，提示需要集成 easyofd-gm。
        Err(format!(
            "证书加密需要集成 SM2/RSA 非对称加密（算法: {:?}，公钥长度: {} 字节）",
            self.algorithm,
            self.public_key_der.len()
        ))
    }

    fn name(&self) -> &'static str {
        "UserCertEncryptor"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_cert_encryptor_sm2() {
        let encryptor = UserCertEncryptor::sm2(vec![0x30; 64]);
        assert_eq!(encryptor.algorithm(), CertAlgorithm::Sm2);
        assert_eq!(encryptor.public_key_der().len(), 64);
    }

    #[test]
    fn test_user_cert_encryptor_rsa() {
        let encryptor = UserCertEncryptor::rsa(vec![0x30; 128]);
        assert_eq!(encryptor.algorithm(), CertAlgorithm::Rsa);
    }

    #[test]
    fn test_user_cert_encryptor_name() {
        let encryptor = UserCertEncryptor::sm2(vec![]);
        assert_eq!(encryptor.name(), "UserCertEncryptor");
    }

    #[test]
    fn test_user_cert_encryptor_encrypt_requires_sm2() {
        let encryptor = UserCertEncryptor::sm2(vec![0x30; 64]);
        let result = encryptor.encrypt_fek(&[0x42; 16]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("SM2"));
    }
}
