//! 用户 FEK 解密器接口。
//!
//! 对应 Java: org.ofdrw.crypto.decryptor.UserFEKDecryptor

/// 用户 FEK（File Encryption Key）解密器接口。
///
/// 对应 Java: `org.ofdrw.crypto.decryptor.UserFEKDecryptor`
///
/// FEK 是文件加密密钥，用于加密 OFD 文档中的实际文件内容。
/// 用户通过自己的密钥（如密码或证书）解密 FEK，再用 FEK 解密文件。
pub trait UserFekDecryptor: std::fmt::Debug {
    /// 解密 FEK。
    ///
    /// # 参数
    ///
    /// - `encrypted_fek`: 加密后的 FEK 数据。
    ///
    /// # 返回
    ///
    /// 解密后的 FEK（通常为 16 字节 SM4 密钥）。
    ///
    /// # 错误
    ///
    /// 解密失败时返回错误描述字符串。
    fn decrypt_fek(&self, encrypted_fek: &[u8]) -> Result<Vec<u8>, String>;

    /// 获取解密器的描述名称。
    fn name(&self) -> &'static str;
}

/// 基于密钥的 FEK 解密器。
///
/// 使用预设的密钥直接作为 FEK（简化实现，适用于测试或已知密钥的场景）。
#[derive(Debug)]
pub struct DirectFekDecryptor {
    /// 预设的 FEK。
    key: Vec<u8>,
}

impl DirectFekDecryptor {
    /// 创建直接 FEK 解密器。
    #[must_use]
    pub fn new(key: Vec<u8>) -> Self {
        Self { key }
    }
}

impl UserFekDecryptor for DirectFekDecryptor {
    fn decrypt_fek(&self, _encrypted_fek: &[u8]) -> Result<Vec<u8>, String> {
        Ok(self.key.clone())
    }

    fn name(&self) -> &'static str {
        "DirectFekDecryptor"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_direct_fek_decryptor() {
        let key = vec![0x01u8; 16];
        let decryptor = DirectFekDecryptor::new(key.clone());
        let result = decryptor.decrypt_fek(&[0xFF; 32]).unwrap();
        assert_eq!(result, key);
        assert_eq!(decryptor.name(), "DirectFekDecryptor");
    }
}
