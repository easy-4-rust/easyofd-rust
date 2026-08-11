//! 普通印章位置。
//!
//! 对应 Java: `org.ofdrw.sign.stamppos.NormalStampPos`

/// 普通印章位置描述。
///
/// 对应 Java: `org.ofdrw.sign.stamppos.NormalStampPos`
///
/// 描述印章在某一页上的放置位置和尺寸，单位为毫米（mm）。
#[derive(Debug, Clone, PartialEq)]
pub struct NormalStampPos {
    /// 图章所在页面页码（从 1 起）。
    page: u32,
    /// 图章左上角 X 坐标（mm）。
    tlx: f64,
    /// 图章左上角 Y 坐标（mm）。
    tly: f64,
    /// 图章宽度（mm）。
    width: f64,
    /// 图章高度（mm）。
    height: f64,
}

impl NormalStampPos {
    /// 创建普通印章位置。
    ///
    /// # 参数
    ///
    /// - `page`：页码（从 1 起）
    /// - `tlx`：左上角 X 坐标（mm）
    /// - `tly`：左上角 Y 坐标（mm）
    /// - `width`：宽度（mm）
    /// - `height`：高度（mm）
    #[must_use]
    pub fn new(page: u32, tlx: f64, tly: f64, width: f64, height: f64) -> Self {
        Self {
            page,
            tlx,
            tly,
            width,
            height,
        }
    }

    /// 获取页码。
    #[must_use]
    pub fn page(&self) -> u32 {
        self.page
    }

    /// 获取左上角 X 坐标。
    #[must_use]
    pub fn tlx(&self) -> f64 {
        self.tlx
    }

    /// 获取左上角 Y 坐标。
    #[must_use]
    pub fn tly(&self) -> f64 {
        self.tly
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

    /// 设置页码。
    pub fn set_page(&mut self, page: u32) {
        self.page = page;
    }

    /// 设置左上角 X 坐标。
    pub fn set_tlx(&mut self, tlx: f64) {
        self.tlx = tlx;
    }

    /// 设置左上角 Y 坐标。
    pub fn set_tly(&mut self, tly: f64) {
        self.tly = tly;
    }

    /// 设置宽度。
    pub fn set_width(&mut self, width: f64) {
        self.width = width;
    }

    /// 设置高度。
    pub fn set_height(&mut self, height: f64) {
        self.height = height;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_and_access() {
        let pos = NormalStampPos::new(1, 10.0, 20.0, 50.0, 30.0);
        assert_eq!(pos.page(), 1);
        assert!((pos.tlx() - 10.0).abs() < f64::EPSILON);
        assert!((pos.tly() - 20.0).abs() < f64::EPSILON);
        assert!((pos.width() - 50.0).abs() < f64::EPSILON);
        assert!((pos.height() - 30.0).abs() < f64::EPSILON);
    }

    #[test]
    fn setters() {
        let mut pos = NormalStampPos::new(1, 0.0, 0.0, 100.0, 100.0);
        pos.set_page(5);
        pos.set_tlx(15.5);
        pos.set_tly(25.5);
        pos.set_width(80.0);
        pos.set_height(60.0);
        assert_eq!(pos.page(), 5);
        assert!((pos.tlx() - 15.5).abs() < f64::EPSILON);
        assert!((pos.tly() - 25.5).abs() < f64::EPSILON);
        assert!((pos.width() - 80.0).abs() < f64::EPSILON);
        assert!((pos.height() - 60.0).abs() < f64::EPSILON);
    }

    #[test]
    fn clone_eq() {
        let pos = NormalStampPos::new(1, 10.0, 20.0, 50.0, 30.0);
        let cloned = pos.clone();
        assert_eq!(pos, cloned);
    }
}
