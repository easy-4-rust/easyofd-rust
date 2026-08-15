//! 复合字形分量。
//!
//! 对应 Java: org.ofdrw.converter.font.GlyfCompositeComp
//!
//! 参考 Apache FontBox，遵循 OpenType `glyf` 表复合字形规范。

// ─── 复合字形标志位 ─────────────────────────────────────────────────────────

/// 参数为 16 位（否则为 8 位）。
pub const ARG_1_AND_2_ARE_WORDS: u16 = 0x0001;
/// 参数为 xy 值（否则为点号）。
pub const ARGS_ARE_XY_VALUES: u16 = 0x0002;
/// xy 值舍入到最近网格线。
pub const ROUND_XY_TO_GRID: u16 = 0x0004;
/// 存在简单缩放。
pub const WE_HAVE_A_SCALE: u16 = 0x0008;
/// 后面还有更多分量。
pub const MORE_COMPONENTS: u16 = 0x0020;
/// X 和 Y 方向使用不同缩放。
pub const WE_HAVE_AN_X_AND_Y_SCALE: u16 = 0x0040;
/// 存在 2x2 变换矩阵。
pub const WE_HAVE_A_TWO_BY_TWO: u16 = 0x0080;
/// 最后一个分量之后有指令。
pub const WE_HAVE_INSTRUCTIONS: u16 = 0x0100;
/// 强制复合字形使用此分量的 aw/lsb。
pub const USE_MY_METRICS: u16 = 0x0200;

/// 复合字形中的一个分量。
///
/// 对应 Java `GlyfCompositeComp`。复合字形由多个分量组成，
/// 每个分量引用另一个字形并应用变换。
#[derive(Debug, Clone)]
pub struct GlyfCompositeComp {
    /// 分量的标志位。
    flags: u16,
    /// 引用的字形索引。
    glyph_index: u16,
    /// 参数 1（原始值）。
    argument1: i16,
    /// 参数 2（原始值）。
    argument2: i16,
    /// X 缩放。
    xscale: f64,
    /// Y 缩放。
    yscale: f64,
    /// 交叉缩放 01。
    scale01: f64,
    /// 交叉缩放 10。
    scale10: f64,
    /// X 平移。
    xtranslate: i32,
    /// Y 平移。
    ytranslate: i32,
    /// 匹配点 1。
    point1: u16,
    /// 匹配点 2。
    point2: u16,
}

impl GlyfCompositeComp {
    /// 从原始字段创建复合字形分量。
    pub fn new(flags: u16, glyph_index: u16, argument1: i16, argument2: i16) -> Self {
        let mut comp = Self {
            flags,
            glyph_index,
            argument1,
            argument2,
            xscale: 1.0,
            yscale: 1.0,
            scale01: 0.0,
            scale10: 0.0,
            xtranslate: 0,
            ytranslate: 0,
            point1: 0,
            point2: 0,
        };

        // 根据标志位解析参数
        if (flags & ARGS_ARE_XY_VALUES) != 0 {
            comp.xtranslate = argument1 as i32;
            comp.ytranslate = argument2 as i32;
        } else {
            comp.point1 = argument1 as u16;
            comp.point2 = argument2 as u16;
        }

        comp
    }

    /// 设置缩放值。
    pub fn set_scale(&mut self, xscale: f64, yscale: f64) {
        self.xscale = xscale;
        self.yscale = yscale;
    }

    /// 设置 2x2 变换矩阵的交叉分量。
    pub fn set_affine(&mut self, scale01: f64, scale10: f64) {
        self.scale01 = scale01;
        self.scale10 = scale10;
    }

    // ─── getter ──────────────────────────────────────────────────────────────

    /// 返回复合字形组件标志。
    pub fn flags(&self) -> u16 {
        self.flags
    }
    /// 返回字形索引。
    pub fn glyph_index(&self) -> u16 {
        self.glyph_index
    }
    /// 返回参数 1（偏移或匹配点）。
    pub fn argument1(&self) -> i16 {
        self.argument1
    }
    /// 返回参数 2（偏移或匹配点）。
    pub fn argument2(&self) -> i16 {
        self.argument2
    }
    /// 返回 X 缩放因子。
    pub fn xscale(&self) -> f64 {
        self.xscale
    }
    /// 返回 Y 缩放因子。
    pub fn yscale(&self) -> f64 {
        self.yscale
    }
    /// 返回交叉缩放因子 scale01。
    pub fn scale01(&self) -> f64 {
        self.scale01
    }
    /// 返回交叉缩放因子 scale10。
    pub fn scale10(&self) -> f64 {
        self.scale10
    }
    /// 返回 X 平移量。
    pub fn xtranslate(&self) -> i32 {
        self.xtranslate
    }
    /// 返回 Y 平移量。
    pub fn ytranslate(&self) -> i32 {
        self.ytranslate
    }

    /// 变换 X 坐标。
    pub fn scale_x(&self, x: i32, y: i32) -> i32 {
        (x as f64 * self.xscale + y as f64 * self.scale10).round() as i32
    }

    /// 变换 Y 坐标。
    pub fn scale_y(&self, x: i32, y: i32) -> i32 {
        (x as f64 * self.scale01 + y as f64 * self.yscale).round() as i32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_with_xy_values() {
        let comp = GlyfCompositeComp::new(ARGS_ARE_XY_VALUES | ARG_1_AND_2_ARE_WORDS, 5, 100, 200);
        assert_eq!(comp.glyph_index(), 5);
        assert_eq!(comp.xtranslate(), 100);
        assert_eq!(comp.ytranslate(), 200);
    }

    #[test]
    fn test_new_with_point_numbers() {
        let comp = GlyfCompositeComp::new(ARG_1_AND_2_ARE_WORDS, 3, 10, 20);
        assert_eq!(comp.glyph_index(), 3);
        assert_eq!(comp.point1, 10);
        assert_eq!(comp.point2, 20);
    }

    #[test]
    fn test_scale() {
        let mut comp = GlyfCompositeComp::new(ARGS_ARE_XY_VALUES, 1, 0, 0);
        comp.set_scale(2.0, 3.0);
        assert!((comp.xscale() - 2.0).abs() < f64::EPSILON);
        assert!((comp.yscale() - 3.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_scale_x_y() {
        let mut comp = GlyfCompositeComp::new(ARGS_ARE_XY_VALUES, 1, 0, 0);
        comp.set_scale(2.0, 3.0);
        assert_eq!(comp.scale_x(10, 0), 20);
        assert_eq!(comp.scale_y(0, 10), 30);
    }

    #[test]
    fn test_affine() {
        let mut comp = GlyfCompositeComp::new(ARGS_ARE_XY_VALUES, 1, 0, 0);
        comp.set_affine(0.5, 0.5);
        assert!((comp.scale01() - 0.5).abs() < f64::EPSILON);
        assert!((comp.scale10() - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_flags() {
        assert_eq!(ARG_1_AND_2_ARE_WORDS, 0x0001);
        assert_eq!(ARGS_ARE_XY_VALUES, 0x0002);
        assert_eq!(WE_HAVE_A_SCALE, 0x0008);
        assert_eq!(MORE_COMPONENTS, 0x0020);
        assert_eq!(WE_HAVE_AN_X_AND_Y_SCALE, 0x0040);
        assert_eq!(WE_HAVE_A_TWO_BY_TWO, 0x0080);
        assert_eq!(WE_HAVE_INSTRUCTIONS, 0x0100);
        assert_eq!(USE_MY_METRICS, 0x0200);
    }

    #[test]
    fn test_clone() {
        let comp = GlyfCompositeComp::new(ARGS_ARE_XY_VALUES, 1, 10, 20);
        let comp2 = comp.clone();
        assert_eq!(comp.glyph_index(), comp2.glyph_index());
        assert_eq!(comp.xtranslate(), comp2.xtranslate());
    }
}
