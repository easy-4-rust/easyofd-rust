//! 水平度量表。
//!
//! 对应 Java: org.ofdrw.converter.font.HorizontalMetricsTable
//!
//! 参考 OpenType `hmtx` 表规范。

/// 水平度量条目。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LongHorMetric {
    /// 前进宽度。
    pub advance_width: u16,
    /// 左留白。
    pub left_side_bearing: i16,
}

/// 水平度量表（`hmtx`）。
///
/// 对应 Java `HorizontalMetricsTable`。包含每个字形的前进宽度和左留白。
#[derive(Debug, Clone)]
pub struct HorizontalMetricsTable {
    /// 长水平度量数组（前进宽度 + 左留白）。
    metrics: Vec<LongHorMetric>,
    /// 额外的左留白数组（当字形数量 > `number_of_h_metrics` 时）。
    left_side_bearings: Vec<i16>,
}

impl HorizontalMetricsTable {
    /// 创建空的水平度量表。
    pub fn new() -> Self {
        Self {
            metrics: Vec::new(),
            left_side_bearings: Vec::new(),
        }
    }

    /// 从度量数组创建。
    pub fn from_metrics(metrics: Vec<LongHorMetric>, left_side_bearings: Vec<i16>) -> Self {
        Self {
            metrics,
            left_side_bearings,
        }
    }

    /// 返回指定字形的前进宽度。
    pub fn get_advance_width(&self, glyph_index: u16) -> u16 {
        let idx = glyph_index as usize;
        if idx < self.metrics.len() {
            self.metrics[idx].advance_width
        } else if !self.metrics.is_empty() {
            // 超出范围时使用最后一个度量的前进宽度
            self.metrics.last().map_or(0, |m| m.advance_width)
        } else {
            0
        }
    }

    /// 返回指定字形的左留白。
    pub fn get_left_side_bearing(&self, glyph_index: u16) -> i16 {
        let idx = glyph_index as usize;
        if idx < self.metrics.len() {
            self.metrics[idx].left_side_bearing
        } else {
            let lsb_idx = idx - self.metrics.len();
            if lsb_idx < self.left_side_bearings.len() {
                self.left_side_bearings[lsb_idx]
            } else {
                0
            }
        }
    }

    /// 返回度量数量。
    pub fn len(&self) -> usize {
        self.metrics.len() + self.left_side_bearings.len()
    }

    /// 是否为空。
    pub fn is_empty(&self) -> bool {
        self.metrics.is_empty() && self.left_side_bearings.is_empty()
    }

    /// 返回度量数组引用。
    pub fn metrics(&self) -> &[LongHorMetric] {
        &self.metrics
    }
}

impl Default for HorizontalMetricsTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let hmtx = HorizontalMetricsTable::new();
        assert!(hmtx.is_empty());
        assert_eq!(hmtx.len(), 0);
    }

    #[test]
    fn test_from_metrics() {
        let metrics = vec![
            LongHorMetric {
                advance_width: 600,
                left_side_bearing: 0,
            },
            LongHorMetric {
                advance_width: 700,
                left_side_bearing: 10,
            },
        ];
        let hmtx = HorizontalMetricsTable::from_metrics(metrics, vec![]);
        assert_eq!(hmtx.len(), 2);
        assert_eq!(hmtx.get_advance_width(0), 600);
        assert_eq!(hmtx.get_advance_width(1), 700);
        assert_eq!(hmtx.get_left_side_bearing(0), 0);
        assert_eq!(hmtx.get_left_side_bearing(1), 10);
    }

    #[test]
    fn test_fallback_advance_width() {
        let metrics = vec![LongHorMetric {
            advance_width: 500,
            left_side_bearing: 0,
        }];
        let hmtx = HorizontalMetricsTable::from_metrics(metrics, vec![]);
        // 超出范围时使用最后一个度量
        assert_eq!(hmtx.get_advance_width(10), 500);
    }

    #[test]
    fn test_left_side_bearings_overflow() {
        let metrics = vec![LongHorMetric {
            advance_width: 500,
            left_side_bearing: 0,
        }];
        let lsb = vec![5, 10, 15];
        let hmtx = HorizontalMetricsTable::from_metrics(metrics, lsb);
        assert_eq!(hmtx.get_left_side_bearing(1), 5);
        assert_eq!(hmtx.get_left_side_bearing(2), 10);
        assert_eq!(hmtx.get_left_side_bearing(3), 15);
    }

    #[test]
    fn test_default() {
        let hmtx = HorizontalMetricsTable::default();
        assert!(hmtx.is_empty());
    }
}
