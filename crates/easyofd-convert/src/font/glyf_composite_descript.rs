//! 复合字形描述符。
//!
//! 对应 Java: org.ofdrw.converter.font.GlyfCompositeDescript

use crate::font::glyf_composite_comp::GlyfCompositeComp;

/// 复合字形描述符。
///
/// 对应 Java `GlyfCompositeDescript`。复合字形由多个分量（component）组成，
/// 每个分量引用另一个字形并可应用变换（缩放、平移等）。
///
/// 参考 OpenType `glyf` 表规范中复合字形的数据格式。
#[derive(Debug, Clone)]
pub struct GlyfCompositeDescript {
    /// 轮廓数量（固定为 -1，表示复合字形）。
    contour_count: i16,
    /// 分量列表。
    components: Vec<GlyfCompositeComp>,
    /// 提示指令。
    instructions: Vec<u8>,
}

impl GlyfCompositeDescript {
    /// 创建复合字形描述符。
    ///
    /// # 参数
    /// - `components`：分量列表
    /// - `instructions`：提示指令
    pub fn new(components: Vec<GlyfCompositeComp>, instructions: Vec<u8>) -> Self {
        Self {
            contour_count: -1,
            components,
            instructions,
        }
    }

    /// 创建空的复合字形描述符。
    pub fn empty() -> Self {
        Self {
            contour_count: -1,
            components: Vec::new(),
            instructions: Vec::new(),
        }
    }

    /// 返回轮廓数量（固定为 -1）。
    pub fn contour_count(&self) -> i16 {
        self.contour_count
    }

    /// 返回分量列表。
    pub fn components(&self) -> &[GlyfCompositeComp] {
        &self.components
    }

    /// 返回可变分量列表。
    pub fn components_mut(&mut self) -> &mut Vec<GlyfCompositeComp> {
        &mut self.components
    }

    /// 返回提示指令。
    pub fn instructions(&self) -> &[u8] {
        &self.instructions
    }

    /// 是否为复合字形。
    pub fn is_composite(&self) -> bool {
        true
    }

    /// 分量数量。
    pub fn component_count(&self) -> usize {
        self.components.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::glyf_composite_comp::{ARGS_ARE_XY_VALUES, GlyfCompositeComp};

    #[test]
    fn test_empty() {
        let desc = GlyfCompositeDescript::empty();
        assert_eq!(desc.contour_count(), -1);
        assert!(desc.is_composite());
        assert_eq!(desc.component_count(), 0);
    }

    #[test]
    fn test_with_components() {
        let comp1 = GlyfCompositeComp::new(ARGS_ARE_XY_VALUES, 1, 10, 20);
        let comp2 = GlyfCompositeComp::new(ARGS_ARE_XY_VALUES, 2, 30, 40);
        let desc = GlyfCompositeDescript::new(vec![comp1, comp2], vec![0x01]);
        assert_eq!(desc.component_count(), 2);
        assert_eq!(desc.instructions(), &[0x01]);
    }

    #[test]
    fn test_components_access() {
        let comp = GlyfCompositeComp::new(ARGS_ARE_XY_VALUES, 5, 100, 200);
        let desc = GlyfCompositeDescript::new(vec![comp], vec![]);
        assert_eq!(desc.components()[0].glyph_index(), 5);
    }

    #[test]
    fn test_clone() {
        let comp = GlyfCompositeComp::new(ARGS_ARE_XY_VALUES, 1, 0, 0);
        let desc = GlyfCompositeDescript::new(vec![comp], vec![]);
        let desc2 = desc.clone();
        assert_eq!(desc.component_count(), desc2.component_count());
    }
}
