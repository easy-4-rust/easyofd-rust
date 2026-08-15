//! 资源文件内容寻址去重。
//!
//! 对应 Java: org.ofdrw.tool.merge.OFDMerger#copyResFile
//!
//! 按文件内容的 SM3 哈希值进行去重：相同内容的资源文件在合并产物中
//! 只保留一份拷贝，后续遇到相同内容时直接复用已有路径。

use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// 资源文件内容寻址去重器。
///
/// 对应 Java: `OFDMerger.resFileHashTable` + `OFDMerger.resFileCounter`
///
/// 通过计算文件内容的 SM3 哈希值来判断两个资源文件是否相同。
/// 相同内容的文件只拷贝一次，后续请求返回已有路径。
///
/// # 哈希选型
///
/// Java 版使用 BouncyCastle SM3（`org.bouncycastle.jcajce.provider.digest.SM3`），
/// Rust 版使用 `sm3` crate（`sm3::Sm3`），两者输出一致，均为 256 位摘要。
/// 哈希 key 为 SM3 摘要的十六进制小写字符串，与 Java `Hex.toHexString` 行为对齐。
#[derive(Debug, Default)]
pub struct ResourceDedup {
    /// SM3 哈希（hex）→ 合并产物中的资源相对路径。
    hash_to_path: HashMap<String, String>,
    /// 资源文件计数器，用于生成唯一文件名（从 1 开始递增）。
    counter: usize,
}

impl ResourceDedup {
    /// 创建空的资源去重器。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 计算字节内容的 SM3 哈希（十六进制小写字符串）。
    ///
    /// 对应 Java: `SM3.Digest` + `Hex.toHexString(digest.digest())`
    ///
    /// 使用 `sm3::Sm3`（RustCrypto）实现，输出与 BouncyCastle SM3 一致。
    pub fn compute_hash(data: &[u8]) -> String {
        use sm3::Digest;
        let mut hasher = sm3::Sm3::new();
        hasher.update(data);
        let result = hasher.finalize();
        // SM3 输出 32 字节，转为 64 字符十六进制小写
        result.iter().map(|b| format!("{b:02x}")).collect()
    }

    /// 计算文件内容的 SM3 哈希。
    ///
    /// 对应 Java: `copyResFile` 中读取文件并计算 SM3 的逻辑。
    ///
    /// # 错误
    ///
    /// 当文件读取失败时返回 IO 错误。
    pub fn compute_file_hash(path: &Path) -> Result<String, std::io::Error> {
        let data = fs::read(path)?;
        Ok(Self::compute_hash(&data))
    }

    /// 注册或复用资源文件。
    ///
    /// 对应 Java: `OFDMerger#copyResFile`
    ///
    /// 1. 计算文件内容 SM3 哈希。
    /// 2. 若哈希已存在于去重表中，返回已有的资源路径（复用）。
    /// 3. 若不存在，生成新的资源文件名（`{counter}{ext}`），将文件内容
    ///    写入 `output_dir`，注册到去重表并返回新路径。
    ///
    /// # 参数
    ///
    /// - `source_path`：源文件路径。
    /// - `output_dir`：资源输出目录（合并产物的 `Res` 目录）。
    ///
    /// # 返回
    ///
    /// `(资源相对路径, 是否为新拷贝)`。路径格式为 `{counter}{ext}`。
    ///
    /// # 错误
    ///
    /// 当源文件读取或目标文件写入失败时返回 IO 错误。
    pub fn copy_or_reuse(
        &mut self,
        source_path: &Path,
        output_dir: &Path,
    ) -> Result<(String, bool), std::io::Error> {
        let hash = Self::compute_file_hash(source_path)?;

        // 命中去重表：直接返回已有路径
        if let Some(existing) = self.hash_to_path.get(&hash) {
            return Ok((existing.clone(), false));
        }

        // 未命中：生成新文件名，保留原后缀
        self.counter += 1;
        let ext = source_path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| format!(".{e}"))
            .unwrap_or_default();
        let new_name = format!("{}{ext}", self.counter);

        // 确保输出目录存在
        fs::create_dir_all(output_dir)?;
        let dest = output_dir.join(&new_name);
        fs::copy(source_path, &dest)?;

        self.hash_to_path.insert(hash, new_name.clone());
        Ok((new_name, true))
    }

    /// 按哈希查询已注册的资源路径。
    #[must_use]
    pub fn get_by_hash(&self, hash: &str) -> Option<&str> {
        self.hash_to_path.get(hash).map(|s| s.as_str())
    }

    /// 已注册的去重资源数量。
    #[must_use]
    pub fn len(&self) -> usize {
        self.hash_to_path.len()
    }

    /// 是否为空。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.hash_to_path.is_empty()
    }

    /// 获取当前文件计数器值。
    #[must_use]
    pub fn counter(&self) -> usize {
        self.counter
    }

    /// 注册一个哈希-路径映射（用于内存级去重，不需要文件 I/O）。
    ///
    /// 对应 Java: `resFileHashTable.put(hash, resLoc)`
    ///
    /// 在合并器中，当图片数据的 SM3 哈希未命中去重表时，调用此方法注册新的映射。
    /// 调用方应先通过 [`compute_hash`] 计算哈希，再通过 [`get_by_hash`] 检查是否已存在。
    ///
    /// [`compute_hash`]: ResourceDedup::compute_hash
    /// [`get_by_hash`]: ResourceDedup::get_by_hash
    pub fn register(&mut self, hash: String, path: String) {
        self.counter += 1;
        self.hash_to_path.insert(hash, path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compute_hash_deterministic() {
        let data = b"hello world";
        let h1 = ResourceDedup::compute_hash(data);
        let h2 = ResourceDedup::compute_hash(data);
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 64); // SM3 = 256 bits = 64 hex chars
    }

    #[test]
    fn compute_hash_different_content() {
        let h1 = ResourceDedup::compute_hash(b"aaa");
        let h2 = ResourceDedup::compute_hash(b"bbb");
        assert_ne!(h1, h2);
    }

    #[test]
    fn copy_or_reuse_same_content() {
        let tmp = tempfile::tempdir().unwrap();
        let src_dir = tmp.path().join("src");
        let out_dir = tmp.path().join("out");
        fs::create_dir_all(&src_dir).unwrap();

        // 创建两个内容相同的文件
        let file_a = src_dir.join("a.png");
        let file_b = src_dir.join("b.png");
        fs::write(&file_a, b"same image data").unwrap();
        fs::write(&file_b, b"same image data").unwrap();

        let mut dedup = ResourceDedup::new();

        // 第一次拷贝：新文件
        let (path1, is_new1) = dedup.copy_or_reuse(&file_a, &out_dir).unwrap();
        assert!(is_new1);
        assert_eq!(path1, "1.png");
        assert!(out_dir.join(&path1).exists());

        // 第二次拷贝相同内容：复用
        let (path2, is_new2) = dedup.copy_or_reuse(&file_b, &out_dir).unwrap();
        assert!(!is_new2);
        assert_eq!(path2, "1.png"); // 同一路径
        assert_eq!(dedup.len(), 1); // 去重表只有一条
    }

    #[test]
    fn copy_or_reuse_different_content() {
        let tmp = tempfile::tempdir().unwrap();
        let src_dir = tmp.path().join("src");
        let out_dir = tmp.path().join("out");
        fs::create_dir_all(&src_dir).unwrap();

        let file_a = src_dir.join("a.ttf");
        let file_b = src_dir.join("b.ttf");
        fs::write(&file_a, b"font A data").unwrap();
        fs::write(&file_b, b"font B data").unwrap();

        let mut dedup = ResourceDedup::new();

        let (path1, _) = dedup.copy_or_reuse(&file_a, &out_dir).unwrap();
        let (path2, _) = dedup.copy_or_reuse(&file_b, &out_dir).unwrap();

        assert_ne!(path1, path2);
        assert_eq!(dedup.len(), 2);
        assert_eq!(dedup.counter(), 2);
    }

    #[test]
    fn no_extension() {
        let tmp = tempfile::tempdir().unwrap();
        let src_dir = tmp.path().join("src");
        let out_dir = tmp.path().join("out");
        fs::create_dir_all(&src_dir).unwrap();

        let file = src_dir.join("noext");
        fs::write(&file, b"data").unwrap();

        let mut dedup = ResourceDedup::new();
        let (path, _) = dedup.copy_or_reuse(&file, &out_dir).unwrap();
        assert_eq!(path, "1"); // 无后缀
    }

    #[test]
    fn get_by_hash_works() {
        let mut dedup = ResourceDedup::new();
        let hash = ResourceDedup::compute_hash(b"test");
        dedup.hash_to_path.insert(hash.clone(), "1.bin".to_string());
        assert_eq!(dedup.get_by_hash(&hash), Some("1.bin"));
        assert!(dedup.get_by_hash("nonexistent").is_none());
    }

    #[test]
    fn empty_and_len() {
        let dedup = ResourceDedup::new();
        assert!(dedup.is_empty());
        assert_eq!(dedup.len(), 0);
        assert_eq!(dedup.counter(), 0);
    }
}
