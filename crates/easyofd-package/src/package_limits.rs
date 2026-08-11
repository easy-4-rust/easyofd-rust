//! OFD 包的资源限制。

/// OFD 包的资源限制。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PackageLimits {
    /// ZIP 条目数量上限。
    pub max_entries: usize,
    /// 所有解压条目的总字节数上限。
    pub max_total_uncompressed_size: u64,
    /// 单个条目的解压字节数上限。
    pub max_entry_uncompressed_size: u64,
    /// 最大压缩比，阻止高压缩率 ZIP 炸弹。
    pub max_compression_ratio: u64,
}

impl Default for PackageLimits {
    fn default() -> Self {
        Self {
            max_entries: 20_000,
            max_total_uncompressed_size: 1_073_741_824,
            max_entry_uncompressed_size: 268_435_456,
            max_compression_ratio: 1_000,
        }
    }
}
