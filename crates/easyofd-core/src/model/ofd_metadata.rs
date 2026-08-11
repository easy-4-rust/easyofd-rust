//! OFD 文档元数据。

use chrono::NaiveDateTime;

/// OFD 文档元数据（OFD.xml 层级）。
#[derive(Debug, Clone)]
pub struct OfdMetadata {
    /// 文档版本（默认: "1.0"）。
    pub version: String,
    /// 文档标识符（ofdrw: DocID）。
    pub doc_id: Option<String>,
    /// 文档标题。
    pub title: Option<String>,
    /// 文档作者。
    pub author: Option<String>,
    /// 创建应用程序名称。
    pub creator: Option<String>,
    /// 创建日期。
    pub creation_date: Option<NaiveDateTime>,
}

impl Default for OfdMetadata {
    fn default() -> Self {
        Self {
            version: "1.0".to_string(),
            doc_id: None,
            title: None,
            author: None,
            creator: None,
            creation_date: None,
        }
    }
}
