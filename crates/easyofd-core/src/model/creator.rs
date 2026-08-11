//! OFD 创建者信息。
//!
//! 对应 Java: org.ofdrw.core.basicStructure.ofd.docInfo.CT_DocInfo (Creator/CreatorVersion)

/// 创建应用程序信息（ofd:Creator + ofd:CreatorVersion）。
///
/// 对应 Java: ofdrw Creator + CreatorVersion 字段。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Creator {
    /// 创建应用程序名称（ofd:Creator）。
    pub name: Option<String>,
    /// 创建应用程序版本（ofd:CreatorVersion）。
    pub version: Option<String>,
}

impl Creator {
    /// 创建新的 Creator。
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: Some(name.into()),
            version: Some(version.into()),
        }
    }
}
