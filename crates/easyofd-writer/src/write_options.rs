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
                version: "1.0".to_string(),
                doc_id: None,
                title: None,
                author: None,
                creator: None,
                creation_date: Some(Utc::now().naive_utc()),
            },
        }
    }
}
