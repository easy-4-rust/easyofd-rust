//! 写入选项。

use chrono::Utc;
use easyofd_core::OfdMetadata;

/// OFD 写入选项。
#[derive(Debug, Clone)]
pub struct WriteOptions {
    /// 文档元数据。
    pub metadata: OfdMetadata,
}

impl Default for WriteOptions {
    fn default() -> Self {
        Self {
            metadata: OfdMetadata {
                creation_date: Some(Utc::now().naive_utc()),
                ..OfdMetadata::default()
            },
        }
    }
}
