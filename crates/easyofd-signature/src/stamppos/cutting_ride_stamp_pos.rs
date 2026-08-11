//! 骑缝章切割位置。
//!
//! 对应 Java: `org.ofdrw.sign.stamppos.CuttingRideStampPos`
//!
//! 结合 [`CuttingRatio`] 和 [`RidingStampPos`] 的参数，描述骑缝章的切割方式与位置。

use super::Side;

/// 骑缝章切割位置描述。
///
/// 对应 Java: `org.ofdrw.sign.stamppos.CuttingRideStampPos`
///
/// 将骑缝章按指定比例切割后分别放置在相邻两页的指定边上。
/// 与 [`RidingStampPos`] 不同，此类型通过 [`CuttingRatio`] 控制切割比例，
/// 而非等分。
#[derive(Debug, Clone, PartialEq)]
pub struct CuttingRideStampPos {
    /// 骑缝章所在边（默认右侧）。
    side: Side,
    /// 图章整章宽度（mm）。
    width: f64,
    /// 图章整章高度（mm）。
    height: f64,
    /// 切割比例（左页占比，0.0..=1.0）。
    /// 默认 0.5 表示等分。
    cutting_ratio: f64,
    /// 图章在边上距离最近边的偏移坐标（mm）。
    /// `None` 表示居中。
    offset: Option<f64>,
    /// 图章在边上的 margin（mm），默认为 0。
    margin: f64,
}

impl CuttingRideStampPos {
    /// 创建右侧居中骑缝章，切割比例 0.5。
    ///
    /// # 参数
    ///
    /// - `width`：章宽度（mm）
    /// - `height`：章高度（mm）
    #[must_use]
    pub fn new(width: f64, height: f64) -> Self {
        Self {
            side: Side::Right,
            width,
            height,
            cutting_ratio: 0.5,
            offset: None,
            margin: 0.0,
        }
    }

    /// 指定骑缝章所在边。
    #[must_use]
    pub fn with_side(mut self, side: Side) -> Self {
        self.side = side;
        self
    }

    /// 指定切割比例（左页占比，0.0..=1.0）。
    ///
    /// 超出范围的值会被自动钳制到 0.0 或 1.0。
    #[must_use]
    pub fn with_cutting_ratio(mut self, ratio: f64) -> Self {
        self.cutting_ratio = ratio.clamp(0.0, 1.0);
        self
    }

    /// 指定偏移量（距最近边的距离）。
    #[must_use]
    pub fn with_offset(mut self, offset: f64) -> Self {
        self.offset = Some(offset);
        self
    }

    /// 指定 margin。
    #[must_use]
    pub fn with_margin(mut self, margin: f64) -> Self {
        self.margin = margin;
        self
    }

    /// 获取所在边。
    #[must_use]
    pub fn side(&self) -> Side {
        self.side
    }

    /// 获取宽度。
    #[must_use]
    pub fn width(&self) -> f64 {
        self.width
    }

    /// 获取高度。
    #[must_use]
    pub fn height(&self) -> f64 {
        self.height
    }

    /// 获取切割比例。
    #[must_use]
    pub fn cutting_ratio(&self) -> f64 {
        self.cutting_ratio
    }

    /// 获取偏移量。
    #[must_use]
    pub fn offset(&self) -> Option<f64> {
        self.offset
    }

    /// 获取 margin。
    #[must_use]
    pub fn margin(&self) -> f64 {
        self.margin
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_right_side() {
        let pos = CuttingRideStampPos::new(20.0, 30.0);
        assert_eq!(pos.side(), Side::Right);
        assert!((pos.width() - 20.0).abs() < f64::EPSILON);
        assert!((pos.height() - 30.0).abs() < f64::EPSILON);
        assert!((pos.cutting_ratio() - 0.5).abs() < f64::EPSILON);
        assert!(pos.offset().is_none());
        assert!((pos.margin() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn builder_pattern() {
        let pos = CuttingRideStampPos::new(20.0, 30.0)
            .with_side(Side::Left)
            .with_cutting_ratio(0.3)
            .with_offset(5.0)
            .with_margin(3.0);
        assert_eq!(pos.side(), Side::Left);
        assert!((pos.cutting_ratio() - 0.3).abs() < f64::EPSILON);
        assert_eq!(pos.offset(), Some(5.0));
        assert!((pos.margin() - 3.0).abs() < f64::EPSILON);
    }

    #[test]
    fn cutting_ratio_clamp() {
        let pos = CuttingRideStampPos::new(20.0, 30.0).with_cutting_ratio(1.5);
        assert!((pos.cutting_ratio() - 1.0).abs() < f64::EPSILON);

        let pos = CuttingRideStampPos::new(20.0, 30.0).with_cutting_ratio(-0.1);
        assert!((pos.cutting_ratio() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn clone_eq() {
        let pos = CuttingRideStampPos::new(20.0, 30.0).with_side(Side::Top);
        let cloned = pos.clone();
        assert_eq!(pos, cloned);
    }
}
