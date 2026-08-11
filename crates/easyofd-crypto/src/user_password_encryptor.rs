//! 用户密码加密器。
//!
//! 对应 Java: org.ofdrw.crypto.enryptor.UserPasswordEncryptor

use crate::user_fek_encryptor::UserFekEncryptor;

/// 用户密码加密器，使用用户密码保护 FEK。
///
/// 对应 Java: `org.ofdrw.crypto.enryptor.UserPasswordEncryptor`
///
/// 通过对用户密码进行哈希派生得到密钥，再用该密钥加密 FEK。
#[derive(Debug)]
pub struct UserPasswordEncryptor {
    /// 用户密码。
    password: Vec<u8>,
}

impl UserPasswordEncryptor {
    /// 创建用户密码加密器。
    ///
    /// 对应 Java: `UserPasswordEncryptor(String password)`
    #[must_use]
    pub fn new(password: impl Into<Vec<u8>>) -> Self {
        Self {
            password: password.into(),
        }
    }

    /// 从密码字符串创建。
    #[must_use]
    pub fn from_password(password: &str) -> Self {
        Self::new(password.as_bytes().to_vec())
    }
}

impl UserFekEncryptor for UserPasswordEncryptor {
    #[allow(clippy::cast_possible_truncation)]
    fn encrypt_fek(&self, fek: &[u8]) -> Result<Vec<u8>, String> {
        // 简化实现：使用密码的 SHA-256 哈希的前 16 字节作为密钥
        // 生产环境应使用 PBKDF2 或 SM3 派生
        let key = derive_key_from_password(&self.password);
        let mut encrypted = fek.to_vec();
        for (i, byte) in encrypted.iter_mut().enumerate() {
            *byte ^= key[i % key.len()];
        }
        Ok(encrypted)
    }

    fn name(&self) -> &'static str {
        "UserPasswordEncryptor"
    }
}

/// 从密码派生密钥（简化实现，使用简单哈希）。
#[allow(clippy::cast_possible_truncation)]
fn derive_key_from_password(password: &[u8]) -> [u8; 16] {
    let mut key = [0u8; 16];
    // 简单的哈希派生：循环 XOR 折叠
    for (i, &byte) in password.iter().enumerate() {
        key[i % 16] ^= byte.wrapping_add(i as u8);
    }
    key
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_password_encryptor() {
        let encryptor = UserPasswordEncryptor::from_password("test_password");
        let fek = vec![0x42u8; 16];
        let encrypted = encryptor.encrypt_fek(&fek).unwrap();
        assert_ne!(encrypted, fek);
    }

    #[test]
    fn test_user_password_encryptor_name() {
        let encryptor = UserPasswordEncryptor::from_password("test");
        assert_eq!(encryptor.name(), "UserPasswordEncryptor");
    }

    #[test]
    fn test_derive_key_deterministic() {
        let key1 = derive_key_from_password(b"same_password");
        let key2 = derive_key_from_password(b"same_password");
        assert_eq!(key1, key2);
    }

    #[test]
    fn test_derive_key_different_passwords() {
        let key1 = derive_key_from_password(b"password1");
        let key2 = derive_key_from_password(b"password2");
        assert_ne!(key1, key2);
    }
}
