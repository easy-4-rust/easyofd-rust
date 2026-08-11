//! 骑缝章位置。
//!
//! 对应 Java: `org.ofdrw.sign.stamppos.RidingStampPos`

use super::Side;

/// 骑缝章位置描述。
///
/// 对应 Java: `org.ofdrw.sign.stamppos.RidingStampPos`
///
/// 描述骑缝章的放置参数。默认图章放在边的正中央。
/// 单位为毫米（mm）。
#[derive(Debug, Clone, PartialEq)]
pub struct RidingStampPos {
    /// 骑缝章所在边（默认右侧）。
    side: Side,
    /// 图章整章宽度（mm）。
    width: f64,
    /// 图章整章高度（mm）。
    height: f64,
    /// 图章在边上距离最近边的偏移坐标（mm）。
    /// `None` 表示居中。
    offset: Option<f64>,
    /// 图章在边上的 margin（mm），默认为 0。
    margin: f64,
    /// 图章指定切割等份数量。
    /// 为 0 时以页数等分。页面数量大于切割数量时印章重复。
    clip_number: u32,
}

impl RidingStampPos {
    /// 创建右侧居中骑缝章。
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
            offset: None,
            margin: 0.0,
            clip_number: 0,
        }
    }

    /// 指定骑缝章所在边。
    #[must_use]
    pub fn with_side(mut self, side: Side) -> Self {
        self.side = side;
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

    /// 指定切割等份数量。
    #[must_use]
    pub fn with_clip_number(mut self, clip_number: u32) -> Self {
        self.clip_number = clip_number;
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

    /// 获取切割等份数量。
    #[must_use]
    pub fn clip_number(&self) -> u32 {
        self.clip_number
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_right_side() {
        let pos = RidingStampPos::new(20.0, 30.0);
        assert_eq!(pos.side(), Side::Right);
        assert!((pos.width() - 20.0).abs() < f64::EPSILON);
        assert!((pos.height() - 30.0).abs() < f64::EPSILON);
        assert!(pos.offset().is_none());
        assert!((pos.margin() - 0.0).abs() < f64::EPSILON);
        assert_eq!(pos.clip_number(), 0);
    }

    #[test]
    fn builder_pattern() {
        let pos = RidingStampPos::new(20.0, 30.0)
            .with_side(Side::Left)
            .with_offset(5.0)
            .with_margin(3.0)
            .with_clip_number(5);
        assert_eq!(pos.side(), Side::Left);
        assert_eq!(pos.offset(), Some(5.0));
        assert!((pos.margin() - 3.0).abs() < f64::EPSILON);
        assert_eq!(pos.clip_number(), 5);
    }

    #[test]
    fn clone_eq() {
        let pos = RidingStampPos::new(20.0, 30.0).with_side(Side::Top);
        let cloned = pos.clone();
        assert_eq!(pos, cloned);
    }
}
