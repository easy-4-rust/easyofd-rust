//! 用户密码解密器。
//!
//! 对应 Java: org.ofdrw.crypto.decryptor.UserPasswordDecryptor

use crate::user_fek_decryptor::UserFekDecryptor;

/// 用户密码解密器，使用用户密码解密 FEK。
///
/// 对应 Java: `org.ofdrw.crypto.decryptor.UserPasswordDecryptor`
///
/// 通过对用户密码进行哈希派生得到密钥，再用该密钥解密 FEK。
#[derive(Debug)]
pub struct UserPasswordDecryptor {
    /// 用户密码。
    password: Vec<u8>,
}

impl UserPasswordDecryptor {
    /// 创建用户密码解密器。
    ///
    /// 对应 Java: `UserPasswordDecryptor(String password)`
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

impl UserFekDecryptor for UserPasswordDecryptor {
    #[allow(clippy::cast_possible_truncation)]
    fn decrypt_fek(&self, encrypted_fek: &[u8]) -> Result<Vec<u8>, String> {
        let key = derive_key_from_password(&self.password);
        let mut decrypted = encrypted_fek.to_vec();
        for (i, byte) in decrypted.iter_mut().enumerate() {
            *byte ^= key[i % key.len()];
        }
        Ok(decrypted)
    }

    fn name(&self) -> &'static str {
        "UserPasswordDecryptor"
    }
}

/// 从密码派生密钥（与加密器使用相同的派生逻辑）。
#[allow(clippy::cast_possible_truncation)]
fn derive_key_from_password(password: &[u8]) -> [u8; 16] {
    let mut key = [0u8; 16];
    for (i, &byte) in password.iter().enumerate() {
        key[i % 16] ^= byte.wrapping_add(i as u8);
    }
    key
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::user_fek_encryptor::UserFekEncryptor;
    use crate::user_password_encryptor::UserPasswordEncryptor;

    #[test]
    fn test_user_password_decryptor_roundtrip() {
        let password = "test_password";
        let fek = vec![0x42u8; 16];

        let encryptor = UserPasswordEncryptor::from_password(password);
        let encrypted = encryptor.encrypt_fek(&fek).unwrap();

        let decryptor = UserPasswordDecryptor::from_password(password);
        let decrypted = decryptor.decrypt_fek(&encrypted).unwrap();

        assert_eq!(decrypted, fek);
    }

    #[test]
    fn test_user_password_decryptor_wrong_password() {
        let fek = vec![0x42u8; 16];
        let encryptor = UserPasswordEncryptor::from_password("correct");
        let encrypted = encryptor.encrypt_fek(&fek).unwrap();

        let decryptor = UserPasswordDecryptor::from_password("wrong");
        let decrypted = decryptor.decrypt_fek(&encrypted).unwrap();

        assert_ne!(decrypted, fek);
    }

    #[test]
    fn test_user_password_decryptor_name() {
        let decryptor = UserPasswordDecryptor::from_password("test");
        assert_eq!(decryptor.name(), "UserPasswordDecryptor");
    }
}
