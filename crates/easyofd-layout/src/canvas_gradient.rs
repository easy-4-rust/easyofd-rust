//! Canvas 线性渐变。
//!
//! 对应 Java: org.ofdrw.layout.element.canvas.CanvasGradient

/// 渐变颜色段。
#[derive(Debug, Clone, PartialEq)]
pub struct ColorStop {
    /// 渐变颜色位置，取值范围 `[0, 1]`。
    pub offset: f64,
    /// 颜色值（16 进制格式，如 `#FF0000`）或颜色名。
    pub color: String,
}

impl ColorStop {
    /// 创建颜色段。
    #[must_use]
    pub fn new(offset: f64, color: impl Into<String>) -> Self {
        Self {
            offset: offset.clamp(0.0, 1.0),
            color: color.into(),
        }
    }
}

/// Canvas 线性渐变对象，用于与 HTML Canvas API 兼容。
///
/// 对应 Java: ofdrw layout canvas CanvasGradient。
#[derive(Debug, Clone, PartialEq)]
pub struct CanvasGradient {
    /// 起始点 X 坐标。
    pub x0: f64,
    /// 起始点 Y 坐标。
    pub y0: f64,
    /// 结束点 X 坐标。
    pub x1: f64,
    /// 结束点 Y 坐标。
    pub y1: f64,
    /// 渐变颜色段列表。
    pub color_stops: Vec<ColorStop>,
}

impl CanvasGradient {
    /// 创建线性渐变（对应 Java: CanvasGradient(x0, y0, x1, y1)）。
    #[must_use]
    pub fn new(x0: f64, y0: f64, x1: f64, y1: f64) -> Self {
        Self {
            x0,
            y0,
            x1,
            y1,
            color_stops: Vec::new(),
        }
    }

    /// 添加渐变颜色段（对应 Java: CanvasGradient#addColorStop）。
    pub fn add_color_stop(&mut self, offset: f64, color: impl Into<String>) {
        let stop = ColorStop::new(offset, color);
        // 按 offset 排序插入
        let pos = self
            .color_stops
            .binary_search_by(|s| {
                s.offset
                    .partial_cmp(&stop.offset)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap_or_else(|i| i);
        self.color_stops.insert(pos, stop);
    }

    /// 颜色段数量。
    #[must_use]
    pub fn stop_count(&self) -> usize {
        self.color_stops.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let g = CanvasGradient::new(0.0, 0.0, 100.0, 100.0);
        assert!((g.x0 - 0.0).abs() < f64::EPSILON);
        assert!((g.x1 - 100.0).abs() < f64::EPSILON);
        assert!(g.color_stops.is_empty());
    }

    #[test]
    fn test_add_color_stop() {
        let mut g = CanvasGradient::new(0.0, 0.0, 100.0, 0.0);
        g.add_color_stop(0.0, "#FF0000");
        g.add_color_stop(1.0, "#0000FF");
        g.add_color_stop(0.5, "#00FF00");
        assert_eq!(g.stop_count(), 3);
        // 验证排序
        assert!((g.color_stops[0].offset - 0.0).abs() < f64::EPSILON);
        assert!((g.color_stops[1].offset - 0.5).abs() < f64::EPSILON);
        assert!((g.color_stops[2].offset - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_color_stop_clamp() {
        let stop = ColorStop::new(-0.5, "#000");
        assert!((stop.offset - 0.0).abs() < f64::EPSILON);
        let stop = ColorStop::new(2.0, "#FFF");
        assert!((stop.offset - 1.0).abs() < f64::EPSILON);
    }
}
