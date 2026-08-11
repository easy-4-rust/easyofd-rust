//! 用户 FEK 加密器接口。
//!
//! 对应 Java: org.ofdrw.crypto.enryptor.UserFEKEncryptor

/// 用户 FEK（File Encryption Key）加密器接口。
///
/// 对应 Java: `org.ofdrw.crypto.enryptor.UserFEKEncryptor`
///
/// 将 FEK 用用户的密钥加密，以便只有授权用户才能解密 FEK，
/// 进而解密文档内容。
pub trait UserFekEncryptor: std::fmt::Debug {
    /// 加密 FEK。
    ///
    /// # 参数
    ///
    /// - `fek`: 明文 FEK 数据。
    ///
    /// # 返回
    ///
    /// 加密后的 FEK 数据。
    ///
    /// # 错误
    ///
    /// 加密失败时返回错误描述字符串。
    fn encrypt_fek(&self, fek: &[u8]) -> Result<Vec<u8>, String>;

    /// 获取加密器的描述名称。
    fn name(&self) -> &'static str;
}

/// 基于密钥的 FEK 加密器。
///
/// 使用预设密钥加密 FEK（简化实现，使用 XOR 混淆）。
#[derive(Debug)]
pub struct SimpleFekEncryptor {
    /// 用户密钥。
    key: Vec<u8>,
}

impl SimpleFekEncryptor {
    /// 创建简单 FEK 加密器。
    #[must_use]
    pub fn new(key: Vec<u8>) -> Self {
        Self { key }
    }
}

impl UserFekEncryptor for SimpleFekEncryptor {
    fn encrypt_fek(&self, fek: &[u8]) -> Result<Vec<u8>, String> {
        // 简化实现：XOR 混淆（生产环境应使用 SM2/RSA 等非对称加密）
        let mut encrypted = fek.to_vec();
        for (i, byte) in encrypted.iter_mut().enumerate() {
            *byte ^= self.key[i % self.key.len()];
        }
        Ok(encrypted)
    }

    fn name(&self) -> &'static str {
        "SimpleFekEncryptor"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_fek_encryptor_roundtrip() {
        let key = vec![0xABu8; 16];
        let fek = vec![0x42u8; 16];

        let encryptor = SimpleFekEncryptor::new(key.clone());
        let encrypted = encryptor.encrypt_fek(&fek).unwrap();

        // XOR roundtrip
        let mut decrypted = encrypted.clone();
        for (i, byte) in decrypted.iter_mut().enumerate() {
            *byte ^= key[i % key.len()];
        }
        assert_eq!(decrypted, fek);
    }

    #[test]
    fn test_simple_fek_encryptor_name() {
        let encryptor = SimpleFekEncryptor::new(vec![0x00]);
        assert_eq!(encryptor.name(), "SimpleFekEncryptor");
    }
}
