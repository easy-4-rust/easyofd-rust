//! 颜色空间资源列表。
//!
//! 对应 Java: org.ofdrw.core.basicStructure.res.resources.ColorSpaces

/// 颜色空间资源列表。
///
/// 对应 Java: org.ofdrw.core.basicStructure.res.resources.ColorSpaces
#[derive(Debug, Clone, Default)]
pub struct ColorSpaces {
    /// 颜色空间列表。
    pub items: Vec<String>,
}

impl ColorSpaces {
    /// 创建空颜色空间列表。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 添加颜色空间。
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
    fn color_spaces_new() {
        let cs = ColorSpaces::new();
        assert!(cs.is_empty());
    }

    #[test]
    fn color_spaces_add() {
        let mut cs = ColorSpaces::new();
        cs.add("<ofd:ColorSpace Type=\"RGB\"/>");
        assert_eq!(cs.len(), 1);
    }
}
