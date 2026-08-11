//! 复合图形单元资源列表。
//!
//! 对应 Java: org.ofdrw.core.basicStructure.res.resources.CompositeGraphicUnits

/// 复合图形单元资源列表。
///
/// 对应 Java: org.ofdrw.core.basicStructure.res.resources.CompositeGraphicUnits
#[derive(Debug, Clone, Default)]
pub struct CompositeGraphicUnits {
    /// 复合图形单元列表。
    pub items: Vec<String>,
}

impl CompositeGraphicUnits {
    /// 创建空列表。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 添加单元。
    pub fn add(&mut self, item: impl Into<String>) {
        self.items.push(item.into());
    }

    /// 获取数量。
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn composite_graphic_units_new() {
        let cgu = CompositeGraphicUnits::new();
        assert!(cgu.is_empty());
    }
}
