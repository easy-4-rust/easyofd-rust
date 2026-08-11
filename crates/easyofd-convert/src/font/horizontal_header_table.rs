//! 水平头表。
//!
//! 对应 Java: org.ofdrw.converter.font.HorizontalHeaderTable
//!
//! 参考 OpenType `hhea` 表规范。

/// 水平头表（`hhea`）。
///
/// 对应 Java `HorizontalHeaderTable`。包含字体水平布局的全局度量信息。
#[derive(Debug, Clone)]
pub struct HorizontalHeaderTable {
    /// 版本号（通常为 0x00010000）。
    version: u32,
    /// 上升高度（基线到最高字形顶部的距离）。
    ascender: i16,
    /// 下降深度（基线到最低字形底部的距离，通常为负值）。
    descender: i16,
    /// 行间距。
    line_gap: i16,
    /// 最大前进宽度。
    advance_width_max: u16,
    /// 最小左留白。
    min_left_side_bearing: i16,
    /// 最小右留白。
    min_right_side_bearing: i16,
    /// 最大 X 范围。
    x_max_extent: i16,
    /// 胸线斜率。
    caret_slope_rise: i16,
    /// 胸线斜率运行。
    caret_slope_run: i16,
    /// 胸线偏移。
    caret_offset: i16,
    /// 水平度量数量。
    number_of_h_metrics: u16,
}

impl HorizontalHeaderTable {
    /// 创建空的水平头表。
    pub fn new() -> Self {
        Self {
            version: 0x0001_0000,
            ascender: 0,
            descender: 0,
            line_gap: 0,
            advance_width_max: 0,
            min_left_side_bearing: 0,
            min_right_side_bearing: 0,
            x_max_extent: 0,
            caret_slope_rise: 0,
            caret_slope_run: 0,
            caret_offset: 0,
            number_of_h_metrics: 0,
        }
    }

    // ─── getter/setter ───────────────────────────────────────────────────────

    pub fn ascender(&self) -> i16 {
        self.ascender
    }
    pub fn set_ascender(&mut self, v: i16) {
        self.ascender = v;
    }

    pub fn descender(&self) -> i16 {
        self.descender
    }
    pub fn set_descender(&mut self, v: i16) {
        self.descender = v;
    }

    pub fn line_gap(&self) -> i16 {
        self.line_gap
    }
    pub fn set_line_gap(&mut self, v: i16) {
        self.line_gap = v;
    }

    pub fn advance_width_max(&self) -> u16 {
        self.advance_width_max
    }
    pub fn set_advance_width_max(&mut self, v: u16) {
        self.advance_width_max = v;
    }

    pub fn min_left_side_bearing(&self) -> i16 {
        self.min_left_side_bearing
    }
    pub fn set_min_left_side_bearing(&mut self, v: i16) {
        self.min_left_side_bearing = v;
    }

    pub fn min_right_side_bearing(&self) -> i16 {
        self.min_right_side_bearing
    }
    pub fn set_min_right_side_bearing(&mut self, v: i16) {
        self.min_right_side_bearing = v;
    }

    pub fn x_max_extent(&self) -> i16 {
        self.x_max_extent
    }
    pub fn set_x_max_extent(&mut self, v: i16) {
        self.x_max_extent = v;
    }

    pub fn caret_slope_rise(&self) -> i16 {
        self.caret_slope_rise
    }
    pub fn set_caret_slope_rise(&mut self, v: i16) {
        self.caret_slope_rise = v;
    }

    pub fn caret_slope_run(&self) -> i16 {
        self.caret_slope_run
    }
    pub fn set_caret_slope_run(&mut self, v: i16) {
        self.caret_slope_run = v;
    }

    pub fn caret_offset(&self) -> i16 {
        self.caret_offset
    }
    pub fn set_caret_offset(&mut self, v: i16) {
        self.caret_offset = v;
    }

    pub fn number_of_h_metrics(&self) -> u16 {
        self.number_of_h_metrics
    }
    pub fn set_number_of_h_metrics(&mut self, v: u16) {
        self.number_of_h_metrics = v;
    }

    pub fn version(&self) -> u32 {
        self.version
    }
}

impl Default for HorizontalHeaderTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let hhea = HorizontalHeaderTable::new();
        assert_eq!(hhea.version(), 0x0001_0000);
        assert_eq!(hhea.ascender(), 0);
    }

    #[test]
    fn test_setters() {
        let mut hhea = HorizontalHeaderTable::new();
        hhea.set_ascender(800);
        hhea.set_descender(-200);
        hhea.set_line_gap(100);
        hhea.set_advance_width_max(1500);
        hhea.set_number_of_h_metrics(256);

        assert_eq!(hhea.ascender(), 800);
        assert_eq!(hhea.descender(), -200);
        assert_eq!(hhea.line_gap(), 100);
        assert_eq!(hhea.advance_width_max(), 1500);
        assert_eq!(hhea.number_of_h_metrics(), 256);
    }

    #[test]
    fn test_caret() {
        let mut hhea = HorizontalHeaderTable::new();
        hhea.set_caret_slope_rise(1);
        hhea.set_caret_slope_run(0);
        hhea.set_caret_offset(0);
        assert_eq!(hhea.caret_slope_rise(), 1);
    }

    #[test]
    fn test_default() {
        let hhea = HorizontalHeaderTable::default();
        assert_eq!(hhea.version(), 0x0001_0000);
    }
}
