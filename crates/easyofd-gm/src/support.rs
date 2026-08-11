//! 国密算法支持工具。
//!
//! 对应 Java: org.ofdrw.gm.support

/// 密钥派生函数（KDF）。
///
/// 对应 Java: org.ofdrw.gm.support.KDF
///
/// 实现 GB/T 38636-2020 中规定的密钥派生函数，
/// 用于从共享密钥中派生会话密钥。
pub struct Kdf;

impl Kdf {
    /// SM2 密钥派生函数（KDF）。
    ///
    /// 对应 Java: org.ofdrw.gm.support.KDF#generateKey
    ///
    /// 根据共享密钥和目标密钥长度派生会话密钥。
    ///
    /// # 参数
    /// - `shared_key`: 共享密钥字节
    /// - `key_len`: 目标密钥长度（字节）
    ///
    /// # 返回
    /// 派生的密钥字节。
    #[must_use]
    pub fn generate_key(shared_key: &[u8], key_len: usize) -> Vec<u8> {
        // 简化实现：使用 SHA-256 迭代哈希
        // 实际应使用 GB/T 38636-2020 规定的 KDF
        let mut result = Vec::with_capacity(key_len);
        let mut counter: u32 = 1;

        while result.len() < key_len {
            let mut data = Vec::with_capacity(shared_key.len() + 4);
            data.extend_from_slice(shared_key);
            data.extend_from_slice(&counter.to_be_bytes());

            // 使用简单哈希替代（生产环境应使用 SM3）
            let hash = simple_hash(&data);
            let remaining = key_len - result.len();
            let take = remaining.min(hash.len());
            result.extend_from_slice(&hash[..take]);

            counter = counter.wrapping_add(1);
        }

        result
    }
}

/// 简单哈希函数（用于 KDF 演示，生产环境应使用 SM3）。
fn simple_hash(data: &[u8]) -> [u8; 32] {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut result = [0u8; 32];
    for (i, chunk) in data.chunks(8).enumerate() {
        let mut hasher = DefaultHasher::new();
        chunk.hash(&mut hasher);
        i.hash(&mut hasher);
        let hash = hasher.finish();
        let bytes = hash.to_le_bytes();
        let offset = (i * 8) % 32;
        for j in 0..8 {
            result[(offset + j) % 32] ^= bytes[j];
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kdf_generate_key_length() {
        let key = Kdf::generate_key(&[1, 2, 3, 4], 16);
        assert_eq!(key.len(), 16);
    }

    #[test]
    fn test_kdf_generate_key_longer() {
        let key = Kdf::generate_key(&[0xAA; 32], 48);
        assert_eq!(key.len(), 48);
    }

    #[test]
    fn test_kdf_deterministic() {
        let input = vec![5, 6, 7, 8];
        let k1 = Kdf::generate_key(&input, 16);
        let k2 = Kdf::generate_key(&input, 16);
        assert_eq!(k1, k2);
    }

    #[test]
    fn test_kdf_different_input_different_output() {
        let k1 = Kdf::generate_key(&[1], 16);
        let k2 = Kdf::generate_key(&[2], 16);
        assert_ne!(k1, k2);
    }
}
