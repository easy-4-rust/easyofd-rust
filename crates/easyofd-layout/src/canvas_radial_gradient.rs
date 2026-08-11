//! Canvas 放射状/圆形渐变。
//!
//! 对应 Java: org.ofdrw.layout.element.canvas.CanvasRadialGradient

use crate::canvas_gradient::ColorStop;

/// Canvas 放射状/圆形渐变对象。
///
/// 对应 Java: ofdrw layout canvas CanvasRadialGradient。
#[derive(Debug, Clone, PartialEq)]
pub struct CanvasRadialGradient {
    /// 渐变开始圆的 X 坐标。
    pub x0: f64,
    /// 渐变开始圆的 Y 坐标。
    pub y0: f64,
    /// 开始圆的半径。
    pub r0: f64,
    /// 渐变结束圆的 X 坐标。
    pub x1: f64,
    /// 渐变结束圆的 Y 坐标。
    pub y1: f64,
    /// 结束圆的半径。
    pub r1: f64,
    /// 渐变颜色段列表。
    pub color_stops: Vec<ColorStop>,
}

impl CanvasRadialGradient {
    /// 创建放射状渐变（对应 Java: CanvasRadialGradient(x0, y0, r0, x1, y1, r1)）。
    #[must_use]
    pub fn new(x0: f64, y0: f64, r0: f64, x1: f64, y1: f64, r1: f64) -> Self {
        Self {
            x0,
            y0,
            r0,
            x1,
            y1,
            r1,
            color_stops: Vec::new(),
        }
    }

    /// 添加渐变颜色段（对应 Java: CanvasRadialGradient#addColorStop）。
    pub fn add_color_stop(&mut self, offset: f64, color: impl Into<String>) {
        let stop = ColorStop::new(offset, color);
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
        let g = CanvasRadialGradient::new(50.0, 50.0, 0.0, 50.0, 50.0, 50.0);
        assert!((g.x0 - 50.0).abs() < f64::EPSILON);
        assert!((g.r0 - 0.0).abs() < f64::EPSILON);
        assert!((g.r1 - 50.0).abs() < f64::EPSILON);
        assert!(g.color_stops.is_empty());
    }

    #[test]
    fn test_add_color_stop() {
        let mut g = CanvasRadialGradient::new(0.0, 0.0, 0.0, 0.0, 0.0, 100.0);
        g.add_color_stop(0.0, "#FF0000");
        g.add_color_stop(1.0, "#0000FF");
        assert_eq!(g.stop_count(), 2);
        assert_eq!(g.color_stops[0].color, "#FF0000");
        assert_eq!(g.color_stops[1].color, "#0000FF");
    }
}
