//! 测量的文字区域信息。
//!
//! 对应 Java: org.ofdrw.layout.element.canvas.TextMetricsArea

/// 单个字符的区域信息。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CharArea {
    /// 字符左上角 X（mm）。
    pub x: f64,
    /// 字符左上角 Y（mm）。
    pub y: f64,
    /// 字符宽度（mm）。
    pub width: f64,
    /// 字符高度（mm）。
    pub height: f64,
}

impl CharArea {
    /// 创建字符区域。
    #[must_use]
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

/// 测量的文字区域信息，包含每个字符的尺寸和整体宽高。
///
/// 对应 Java: ofdrw layout canvas TextMetricsArea。
#[derive(Debug, Clone, PartialEq)]
pub struct TextMetricsArea {
    /// 每个字符的区域信息。
    pub char_areas: Vec<CharArea>,
    /// 文字区域宽度（mm）。
    ///
    /// 宽度 = 每个字宽度 + 字间距 * (n - 1)，n 为文字数量。
    pub width: f64,
    /// 文字区域高度（mm）。
    pub height: f64,
    /// 字间距（mm）。
    pub letter_spacing: f64,
}

impl Default for TextMetricsArea {
    fn default() -> Self {
        Self {
            char_areas: Vec::new(),
            width: 0.0,
            height: 0.0,
            letter_spacing: 0.0,
        }
    }
}

impl TextMetricsArea {
    /// 创建空的文字区域信息。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置字符区域列表。
    #[must_use]
    pub fn char_areas(mut self, areas: Vec<CharArea>) -> Self {
        self.char_areas = areas;
        self
    }

    /// 设置区域宽度。
    #[must_use]
    pub fn width(mut self, width: f64) -> Self {
        self.width = width;
        self
    }

    /// 设置区域高度。
    #[must_use]
    pub fn height(mut self, height: f64) -> Self {
        self.height = height;
        self
    }

    /// 设置字间距。
    #[must_use]
    pub fn letter_spacing(mut self, spacing: f64) -> Self {
        self.letter_spacing = spacing;
        self
    }

    /// 字符数量。
    #[must_use]
    pub fn char_count(&self) -> usize {
        self.char_areas.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let area = TextMetricsArea::new();
        assert!(area.char_areas.is_empty());
        assert!((area.width - 0.0).abs() < f64::EPSILON);
        assert!((area.height - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_builders() {
        let area = TextMetricsArea::new()
            .char_areas(vec![
                CharArea::new(0.0, 0.0, 1.0, 3.0),
                CharArea::new(1.5, 0.0, 1.2, 3.0),
            ])
            .width(2.7)
            .height(3.0)
            .letter_spacing(0.5);
        assert_eq!(area.char_count(), 2);
        assert!((area.width - 2.7).abs() < f64::EPSILON);
        assert!((area.height - 3.0).abs() < f64::EPSILON);
        assert!((area.letter_spacing - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_char_area() {
        let ca = CharArea::new(1.0, 2.0, 3.0, 4.0);
        assert!((ca.x - 1.0).abs() < f64::EPSILON);
        assert!((ca.y - 2.0).abs() < f64::EPSILON);
        assert!((ca.width - 3.0).abs() < f64::EPSILON);
        assert!((ca.height - 4.0).abs() < f64::EPSILON);
    }
}
