//! 文本度量分析结果。
//!
//! 对应 Java: org.ofdrw.layout.element.canvas.MeasureBody

/// 文本度量分析结果，包含字符间偏移量和总宽度。
///
/// 对应 Java: ofdrw layout canvas MeasureBody。
#[derive(Debug, Clone, PartialEq)]
pub struct MeasureBody {
    /// 文本字符间相对偏移量（mm）。
    pub offset: Vec<f64>,
    /// 文本在阅读方向上的总宽度（mm）。
    pub width: f64,
    /// 第一个字符相对偏移 X 坐标（mm）。
    pub first_char_offset_x: f64,
    /// 第一个字符相对偏移 Y 坐标（mm）。
    pub first_char_offset_y: f64,
}

impl Default for MeasureBody {
    fn default() -> Self {
        Self {
            offset: Vec::new(),
            width: 0.0,
            first_char_offset_x: 0.0,
            first_char_offset_y: 0.0,
        }
    }
}

impl MeasureBody {
    /// 创建空的度量结果。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置偏移量（对应 Java: MeasureBody#offset）。
    #[must_use]
    pub fn offset(mut self, offset: Vec<f64>) -> Self {
        self.offset = offset;
        self
    }

    /// 设置第一个字符偏移（对应 Java: MeasureBody#firstCharOffsetX/Y）。
    #[must_use]
    pub fn first_char_offset(mut self, x: f64, y: f64) -> Self {
        self.first_char_offset_x = x;
        self.first_char_offset_y = y;
        self
    }

    /// 加上偏移量后的总宽度（对应 Java: MeasureBody#with(double)）。
    ///
    /// 计算方式：所有偏移量绝对值之和 + 最后一个字符宽度。
    pub fn with_char_len(&mut self, char_len: f64) {
        self.width = self.offset.iter().map(|o| o.abs()).sum::<f64>() + char_len;
    }

    /// 字符数量。
    #[must_use]
    pub fn char_count(&self) -> usize {
        self.offset.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let body = MeasureBody::new();
        assert!(body.offset.is_empty());
        assert!((body.width - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_builders() {
        let body = MeasureBody::new()
            .offset(vec![1.0, -0.5, 1.2])
            .first_char_offset(0.1, 0.2);
        assert_eq!(body.offset.len(), 3);
        assert!((body.first_char_offset_x - 0.1).abs() < f64::EPSILON);
        assert!((body.first_char_offset_y - 0.2).abs() < f64::EPSILON);
    }

    #[test]
    fn test_with_char_len() {
        let mut body = MeasureBody::new().offset(vec![1.0, -0.5, 1.2]);
        body.with_char_len(0.8);
        // width = |1.0| + |-0.5| + |1.2| + 0.8 = 3.5
        assert!((body.width - 3.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_char_count() {
        let body = MeasureBody::new().offset(vec![1.0, 2.0, 3.0, 4.0]);
        assert_eq!(body.char_count(), 4);
    }
}
