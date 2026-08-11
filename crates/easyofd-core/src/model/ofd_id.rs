//! OFD 文档标识符。
//!
//! 对应 Java: org.ofdrw.core.basicStructure.ofd.docInfo.CT_DocInfo (DocID)

/// OFD 文档唯一标识符（ofd:DocID）。
///
/// 对应 Java: ofdrw DocID 字段。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OfdId {
    /// 文档唯一标识符 UUID 字符串。
    pub uuid: String,
}

impl OfdId {
    /// 创建新的 OfdId。
    pub fn new(uuid: impl Into<String>) -> Self {
        Self { uuid: uuid.into() }
    }
}
