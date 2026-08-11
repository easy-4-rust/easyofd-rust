//! OFD 模板页。
//!
//! 对应 Java: org.ofdrw.core.basicStructure.doc.CT_PageArea 之 TemplatePage

/// 模板页引用（ofd:TemplatePage），位于 Document.xml 的 CommonData 中。
///
/// 对应 Java: ofdrw CT_TemplatePage。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplatePage {
    /// 模板页标识（ofd:TemplatePage @ID）。
    pub id: String,
    /// 模板页内容位置（ofd:TemplatePage @BaseLoc），相对文档目录。
    pub base_loc: String,
}

impl TemplatePage {
    /// 创建模板页引用。
    pub fn new(id: impl Into<String>, base_loc: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            base_loc: base_loc.into(),
        }
    }
}
