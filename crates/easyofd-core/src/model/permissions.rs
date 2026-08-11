//! OFD 文档权限。
//!
//! 对应 Java: org.ofdrw.core.basicStructure.doc.permission.CT_Permissions

/// 文档权限（ofd:Permissions），位于 Document.xml 根下。
///
/// 对应 Java: ofdrw CT_Permissions。每个权限项为布尔值（true/false），
/// `print` 使用 `Printable` 属性。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Permissions {
    /// 是否允许编辑（ofd:Edit）。
    pub edit: Option<bool>,
    /// 是否允许注释（ofd:Annot）。
    pub annot: Option<bool>,
    /// 是否允许导出（ofd:Export）。
    pub export: Option<bool>,
    /// 是否允许签名（ofd:Signature）。
    pub signature: Option<bool>,
    /// 是否允许水印（ofd:Watermark）。
    pub watermark: Option<bool>,
    /// 是否允许打印屏幕（ofd:PrintScreen）。
    pub print_screen: Option<bool>,
    /// 是否允许打印（ofd:Print @Printable）。
    pub print: Option<bool>,
    /// 是否允许复制文本（ofd:CopyText）。
    pub copy_text: Option<bool>,
    /// 是否允许内容注册（ofd:ContentRegist）。
    pub content_regist: Option<bool>,
}

impl Permissions {
    /// 创建空权限集合。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}
