//! 解密结果。
//!
//! 对应 Java: org.ofdrw.crypto.decryptor.DecryptResult

/// 解密结果，包含解密后的数据和元信息。
///
/// 对应 Java: `org.ofdrw.crypto.decryptor.DecryptResult`
#[derive(Debug, Clone)]
pub struct DecryptResult {
    /// 解密后的文件路径（OFD 容器内相对路径）。
    pub file_path: String,
    /// 解密后的数据。
    pub data: Vec<u8>,
    /// 是否为明文（未加密的文件直接复制）。
    pub is_plaintext: bool,
}

impl DecryptResult {
    /// 创建解密结果。
    #[must_use]
    pub fn new(file_path: impl Into<String>, data: Vec<u8>, is_plaintext: bool) -> Self {
        Self {
            file_path: file_path.into(),
            data,
            is_plaintext,
        }
    }

    /// 创建已解密的结果。
    #[must_use]
    pub fn decrypted(file_path: impl Into<String>, data: Vec<u8>) -> Self {
        Self::new(file_path, data, false)
    }

    /// 创建明文结果（未加密文件）。
    #[must_use]
    pub fn plaintext(file_path: impl Into<String>, data: Vec<u8>) -> Self {
        Self::new(file_path, data, true)
    }

    /// 数据大小（字节）。
    #[must_use]
    pub fn size(&self) -> usize {
        self.data.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decrypt_result_new() {
        let r = DecryptResult::new("Doc.xml", vec![1, 2, 3], false);
        assert_eq!(r.file_path, "Doc.xml");
        assert_eq!(r.data, vec![1, 2, 3]);
        assert!(!r.is_plaintext);
        assert_eq!(r.size(), 3);
    }

    #[test]
    fn test_decrypt_result_decrypted() {
        let r = DecryptResult::decrypted("secret.xml", vec![0xAA]);
        assert!(!r.is_plaintext);
    }

    #[test]
    fn test_decrypt_result_plaintext() {
        let r = DecryptResult::plaintext("OFD.xml", vec![0xBB]);
        assert!(r.is_plaintext);
    }

    #[test]
    fn test_decrypt_result_clone() {
        let r = DecryptResult::new("test", vec![1, 2], true);
        let cloned = r.clone();
        assert_eq!(r.file_path, cloned.file_path);
        assert_eq!(r.data, cloned.data);
    }
}
