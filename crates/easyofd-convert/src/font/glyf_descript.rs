//! 字形描述符基类。
//!
//! 对应 Java: org.ofdrw.converter.font.GlyfDescript
//!
//! 参考 Apache FontBox，遵循 OpenType `glyf` 表规范。

/// 字形轮廓坐标标志：点在曲线上。
pub const ON_CURVE: u8 = 0x01;
/// 字形轮廓坐标标志：X 坐标为 1 字节短向量。
pub const X_SHORT_VECTOR: u8 = 0x02;
/// 字形轮廓坐标标志：Y 坐标为 1 字节短向量。
pub const Y_SHORT_VECTOR: u8 = 0x04;
/// 字形轮廓坐标标志：下一字节指定重复次数。
pub const REPEAT: u8 = 0x08;
/// 字形轮廓坐标标志：X 双重/符号。
pub const X_DUAL: u8 = 0x10;
/// 字形轮廓坐标标志：Y 双重/符号。
pub const Y_DUAL: u8 = 0x20;

/// 字形描述符。
///
/// 对应 Java `GlyfDescript`（抽象类）。描述一个字形的轮廓信息，
/// 包括轮廓数量和提示指令。
///
/// 根据 `number_of_contours` 值分为：
/// - `>= 0`：简单字形（`GlyfSimpleDescript`）
/// - `< 0`：复合字形（`GlyfCompositeDescript`）
#[derive(Debug, Clone)]
pub struct GlyfDescript {
    /// 轮廓数量。>= 0 为简单字形，< 0 为复合字形。
    contour_count: i16,
    /// 提示指令字节码。
    instructions: Vec<u8>,
}

impl GlyfDescript {
    /// 创建字形描述符。
    ///
    /// # 参数
    /// - `contour_count`：轮廓数量
    pub fn new(contour_count: i16) -> Self {
        Self {
            contour_count,
            instructions: Vec::new(),
        }
    }

    /// 返回轮廓数量。
    pub fn contour_count(&self) -> i16 {
        self.contour_count
    }

    /// 返回提示指令。
    pub fn instructions(&self) -> &[u8] {
        &self.instructions
    }

    /// 设置提示指令。
    pub fn set_instructions(&mut self, instructions: Vec<u8>) {
        self.instructions = instructions;
    }

    /// 是否为简单字形（轮廓数 >= 0）。
    pub fn is_simple(&self) -> bool {
        self.contour_count >= 0
    }

    /// 是否为复合字形（轮廓数 < 0）。
    pub fn is_composite(&self) -> bool {
        self.contour_count < 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_glyph() {
        let gd = GlyfDescript::new(3);
        assert_eq!(gd.contour_count(), 3);
        assert!(gd.is_simple());
        assert!(!gd.is_composite());
    }

    #[test]
    fn test_composite_glyph() {
        let gd = GlyfDescript::new(-1);
        assert_eq!(gd.contour_count(), -1);
        assert!(!gd.is_simple());
        assert!(gd.is_composite());
    }

    #[test]
    fn test_instructions() {
        let mut gd = GlyfDescript::new(1);
        assert!(gd.instructions().is_empty());
        gd.set_instructions(vec![0x01, 0x02, 0x03]);
        assert_eq!(gd.instructions(), &[0x01, 0x02, 0x03]);
    }

    #[test]
    fn test_flags() {
        assert_eq!(ON_CURVE, 0x01);
        assert_eq!(X_SHORT_VECTOR, 0x02);
        assert_eq!(Y_SHORT_VECTOR, 0x04);
        assert_eq!(REPEAT, 0x08);
        assert_eq!(X_DUAL, 0x10);
        assert_eq!(Y_DUAL, 0x20);
    }

    #[test]
    fn test_clone() {
        let gd = GlyfDescript::new(2);
        let gd2 = gd.clone();
        assert_eq!(gd.contour_count(), gd2.contour_count());
    }
}
