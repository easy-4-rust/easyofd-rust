//! 骑缝章切割比例。
//!
//! 对应 Java: `org.ofdrw.sign.stamppos.CuttingRatio`

/// 两侧对开骑缝章时左右两边的比例。
///
/// 对应 Java: `org.ofdrw.sign.stamppos.CuttingRatio`
#[derive(Debug, Clone, PartialEq)]
pub struct CuttingRatio {
    /// 左侧比例。
    left: f64,
    /// 右侧比例。
    right: f64,
}

impl CuttingRatio {
    /// 创建切割比例。
    #[must_use]
    pub fn new(left: f64, right: f64) -> Self {
        Self { left, right }
    }

    /// 创建等分比例（1:1）。
    #[must_use]
    pub fn equal() -> Self {
        Self::new(1.0, 1.0)
    }

    /// 获取左侧比例。
    #[must_use]
    pub fn left(&self) -> f64 {
        self.left
    }

    /// 获取右侧比例。
    #[must_use]
    pub fn right(&self) -> f64 {
        self.right
    }

    /// 设置左侧比例。
    pub fn set_left(&mut self, left: f64) {
        self.left = left;
    }

    /// 设置右侧比例。
    pub fn set_right(&mut self, right: f64) {
        self.right = right;
    }
}

impl Default for CuttingRatio {
    fn default() -> Self {
        Self::equal()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_and_access() {
        let ratio = CuttingRatio::new(3.0, 7.0);
        assert!((ratio.left() - 3.0).abs() < f64::EPSILON);
        assert!((ratio.right() - 7.0).abs() < f64::EPSILON);
    }

    #[test]
    fn equal_ratio() {
        let ratio = CuttingRatio::equal();
        assert!((ratio.left() - 1.0).abs() < f64::EPSILON);
        assert!((ratio.right() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn default_is_equal() {
        let ratio = CuttingRatio::default();
        assert_eq!(ratio, CuttingRatio::equal());
    }

    #[test]
    fn setters() {
        let mut ratio = CuttingRatio::equal();
        ratio.set_left(2.0);
        ratio.set_right(8.0);
        assert!((ratio.left() - 2.0).abs() < f64::EPSILON);
        assert!((ratio.right() - 8.0).abs() < f64::EPSILON);
    }
}
