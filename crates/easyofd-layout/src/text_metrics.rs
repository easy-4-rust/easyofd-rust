//! 测量的文字信息。
//!
//! 对应 Java: org.ofdrw.layout.element.canvas.TextMetrics

/// 测量的文字信息，包含阅读方向、宽度、字号和字符偏移量。
///
/// 对应 Java: ofdrw layout canvas TextMetrics。
#[derive(Debug, Clone, PartialEq)]
pub struct TextMetrics {
    /// 阅读方向（0、90、180、270）。
    pub read_direction: i32,
    /// 文字宽度（mm）。
    ///
    /// 若 `read_direction` 为 0 或 180 则为水平宽度；
    /// 若为 90 或 270 则为垂直高度。
    pub width: f64,
    /// 文本字体大小（mm）。
    pub font_size: f64,
    /// 后一个字对前一个字的偏移量（mm）。
    pub offset: Vec<f64>,
}

impl TextMetrics {
    /// 创建文本度量信息。
    #[must_use]
    pub fn new(read_direction: i32, width: f64, font_size: f64) -> Self {
        Self {
            read_direction,
            width,
            font_size,
            offset: Vec::new(),
        }
    }

    /// 设置字符偏移量（对应 Java: TextMetrics#offset）。
    #[must_use]
    pub fn offset(mut self, offset: Vec<f64>) -> Self {
        self.offset = offset;
        self
    }

    /// 字符数量（偏移量数组长度）。
    #[must_use]
    pub fn char_count(&self) -> usize {
        self.offset.len()
    }

    /// 是否为水平阅读方向（0 或 180 度）。
    #[must_use]
    pub fn is_horizontal(&self) -> bool {
        self.read_direction == 0 || self.read_direction == 180
    }

    /// 是否为垂直阅读方向（90 或 270 度）。
    #[must_use]
    pub fn is_vertical(&self) -> bool {
        self.read_direction == 90 || self.read_direction == 270
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let tm = TextMetrics::new(0, 50.0, 3.0);
        assert_eq!(tm.read_direction, 0);
        assert!((tm.width - 50.0).abs() < f64::EPSILON);
        assert!((tm.font_size - 3.0).abs() < f64::EPSILON);
        assert!(tm.offset.is_empty());
    }

    #[test]
    fn test_with_offset() {
        let tm = TextMetrics::new(0, 50.0, 3.0).offset(vec![1.0, 1.2, 0.8]);
        assert_eq!(tm.char_count(), 3);
        assert_eq!(tm.offset, vec![1.0, 1.2, 0.8]);
    }

    #[test]
    fn test_is_horizontal() {
        assert!(TextMetrics::new(0, 10.0, 3.0).is_horizontal());
        assert!(TextMetrics::new(180, 10.0, 3.0).is_horizontal());
        assert!(!TextMetrics::new(90, 10.0, 3.0).is_horizontal());
    }

    #[test]
    fn test_is_vertical() {
        assert!(TextMetrics::new(90, 10.0, 3.0).is_vertical());
        assert!(TextMetrics::new(270, 10.0, 3.0).is_vertical());
        assert!(!TextMetrics::new(0, 10.0, 3.0).is_vertical());
    }
}
