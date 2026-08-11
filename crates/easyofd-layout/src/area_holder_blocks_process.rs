//! 区域占位块处理流程。
//!
//! 对应 Java: org.ofdrw.layout.areaholder.AreaHolderBlocksProcess
//!
//! 管理区域占位块的注册、查找和替换流程。

use crate::area_holder_block::AreaHolderBlock;

/// 区域占位块处理流程。
///
/// 对应 Java: `org.ofdrw.layout.areaholder.AreaHolderBlocksProcess`
///
/// 负责管理一组 [`AreaHolderBlock`]，支持按名称查找和批量处理。
#[derive(Debug, Default)]
pub struct AreaHolderBlocksProcess {
    /// 已注册的占位块列表。
    blocks: Vec<AreaHolderBlock>,
}

impl AreaHolderBlocksProcess {
    /// 创建空的处理流程。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 注册一个占位块。
    pub fn add_block(&mut self, block: AreaHolderBlock) {
        self.blocks.push(block);
    }

    /// 按名称查找占位块。
    #[must_use]
    pub fn find_by_name(&self, name: &str) -> Option<&AreaHolderBlock> {
        self.blocks.iter().find(|b| b.area_name == name)
    }

    /// 按名称查找占位块（可变引用）。
    pub fn find_by_name_mut(&mut self, name: &str) -> Option<&mut AreaHolderBlock> {
        self.blocks.iter_mut().find(|b| b.area_name == name)
    }

    /// 获取占位块数量。
    #[must_use]
    pub fn len(&self) -> usize {
        self.blocks.len()
    }

    /// 是否为空。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.blocks.is_empty()
    }

    /// 获取所有占位块的只读引用。
    #[must_use]
    pub fn blocks(&self) -> &[AreaHolderBlock] {
        &self.blocks
    }

    /// 清空所有占位块。
    pub fn clear(&mut self) {
        self.blocks.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_is_empty() {
        let process = AreaHolderBlocksProcess::new();
        assert!(process.is_empty());
        assert_eq!(process.len(), 0);
    }

    #[test]
    fn add_and_find() {
        let mut process = AreaHolderBlocksProcess::new();
        process.add_block(AreaHolderBlock::new("header", 100.0, 20.0));
        process.add_block(AreaHolderBlock::new("footer", 100.0, 30.0));

        assert_eq!(process.len(), 2);
        assert!(process.find_by_name("header").is_some());
        assert!(process.find_by_name("footer").is_some());
        assert!(process.find_by_name("missing").is_none());
    }

    #[test]
    fn find_mut_and_modify() {
        let mut process = AreaHolderBlocksProcess::new();
        process.add_block(AreaHolderBlock::new("header", 100.0, 20.0));

        let block = process.find_by_name_mut("header").unwrap();
        block.div.x = 10.0;

        let block = process.find_by_name("header").unwrap();
        assert!((block.div.x - 10.0).abs() < f64::EPSILON);
    }

    #[test]
    fn clear() {
        let mut process = AreaHolderBlocksProcess::new();
        process.add_block(AreaHolderBlock::new("a", 10.0, 10.0));
        process.clear();
        assert!(process.is_empty());
    }
}
