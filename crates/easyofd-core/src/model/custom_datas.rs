//! OFD 自定义数据集合。
//!
//! 对应 Java: org.ofdrw.core.basicStructure.ofd.docInfo.CT_CustomDatas

use crate::model::custom_data::CustomData;

/// 自定义数据集合（ofd:CustomDatas）。
///
/// 对应 Java: ofdrw CT_CustomDatas。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomDatas {
    /// 自定义数据列表。
    pub items: Vec<CustomData>,
}

impl CustomDatas {
    /// 创建空集合。
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    /// 添加自定义数据。
    pub fn push(&mut self, data: CustomData) {
        self.items.push(data);
    }

    /// 数量。
    #[must_use]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// 是否为空。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

impl Default for CustomDatas {
    fn default() -> Self {
        Self::new()
    }
}
