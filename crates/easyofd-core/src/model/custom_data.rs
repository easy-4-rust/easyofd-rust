//! OFD 自定义数据项。
//!
//! 对应 Java: org.ofdrw.core.basicStructure.ofd.docInfo.CT_CustomData

/// 单个自定义数据项（ofd:CustomData）。
///
/// 对应 Java: ofdrw CT_CustomData。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomData {
    /// 自定义数据名称（ofd:CustomData 属性 Name）。
    pub name: String,
    /// 自定义数据值（ofd:CustomData 文本内容）。
    pub value: String,
}

impl CustomData {
    /// 创建新自定义数据项。
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
        }
    }
}
