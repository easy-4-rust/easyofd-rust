//! 调色板。
//!
//! 对应 Java: org.ofdrw.core.pageDescription.color.colorSpace.Palette

use super::CV;

/// 调色板。
///
/// 对应 Java: org.ofdrw.core.pageDescription.color.colorSpace.Palette
#[derive(Debug, Clone, Default)]
pub struct Palette {
    /// 颜色分量值列表。
    pub entries: Vec<CV>,
}

impl Palette {
    /// 创建空调色板。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 添加颜色条目。
    pub fn add(&mut self, entry: CV) {
        self.entries.push(entry);
    }

    /// 获取条目数量。
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// 是否为空。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn palette_new() {
        let p = Palette::new();
        assert!(p.is_empty());
    }

    #[test]
    fn palette_add() {
        let mut p = Palette::new();
        p.add(CV::new(vec![0.0, 0.0, 0.0]));
        p.add(CV::new(vec![1.0, 1.0, 1.0]));
        assert_eq!(p.len(), 2);
    }
}
