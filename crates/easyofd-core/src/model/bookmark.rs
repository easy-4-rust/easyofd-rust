//! OFD 书签。
//!
//! 对应 Java: org.ofdrw.core.basicStructure.outline.CT_OutlineNode

/// 单个书签项（ofd:Bookmark / ofd:OutlineElem）。
///
/// 对应 Java: ofdrw CT_OutlineNode。
#[derive(Debug, Clone, PartialEq)]
pub struct Bookmark {
    /// 书签名称。
    pub name: String,
    /// 跳转目标页面（ofd:GoTo 属性或子元素），可选。
    pub goto_target: Option<String>,
}

impl Bookmark {
    /// 创建新书签。
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            goto_target: None,
        }
    }

    /// 设置跳转目标。
    #[must_use]
    pub fn with_goto(mut self, target: impl Into<String>) -> Self {
        self.goto_target = Some(target.into());
        self
    }
}
