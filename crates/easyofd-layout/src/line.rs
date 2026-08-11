//! OFDRW 线条元素，用于快速构建一条线条。
//!
//! 对应 Java: org.ofdrw.layout.element.canvas.Line
//!
//! 若需要绘制复杂图形，请使用 Canvas 对象并提供 Drawer 实现。
//! 若绘制简单矩形，可以使用 Div 对象设置边框实现。

/// 线条元素，包含起点、终点、颜色、线宽和透明度。
///
/// 对应 Java: ofdrw layout canvas Line。尺寸单位均为毫米（mm）。
#[derive(Debug, Clone, PartialEq)]
pub struct Line {
    /// 画线区域左上角 X 坐标（mm）。
    pub x: f64,
    /// 画线区域左上角 Y 坐标（mm）。
    pub y: f64,
    /// 画线区域宽度（mm）。
    pub width: f64,
    /// 画线区域高度（mm）。
    pub height: f64,
    /// 线条起点坐标 `[x, y]`。
    pub begin_point: [f64; 2],
    /// 线条终点坐标 `[x, y]`。
    pub end_point: [f64; 2],
    /// 线条颜色，支持格式：`#000000`、`rgb(0,0,0)`、`rgba(0,0,0,1)`、颜色名。
    pub line_color: String,
    /// 线条宽度（mm），默认 0.353。
    pub line_width: f64,
    /// 线条透明度，范围 `[0, 1]`，默认 1.0（不透明）。
    pub line_opacity: f64,
}

impl Default for Line {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
            begin_point: [0.0, 0.0],
            end_point: [0.0, 0.0],
            line_color: "#000000".to_owned(),
            line_width: 0.353,
            line_opacity: 1.0,
        }
    }
}

impl Line {
    /// 创建线条元素（对应 Java: Line(width, height)）。
    #[must_use]
    pub fn new(width: f64, height: f64) -> Self {
        Self {
            width,
            height,
            ..Self::default()
        }
    }

    /// 创建带位置的线条元素（对应 Java: Line(x, y, w, h)）。
    #[must_use]
    pub fn with_position(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
            ..Self::default()
        }
    }

    /// 设置线条起点（对应 Java: Line#setBeginPoint(double, double)）。
    #[must_use]
    pub fn begin_point(mut self, x: f64, y: f64) -> Self {
        self.begin_point = [x, y];
        self
    }

    /// 设置线条终点（对应 Java: Line#setEndPoint(double, double)）。
    #[must_use]
    pub fn end_point(mut self, x: f64, y: f64) -> Self {
        self.end_point = [x, y];
        self
    }

    /// 设置线条颜色（对应 Java: Line#setLineColor）。
    #[must_use]
    pub fn line_color(mut self, color: impl Into<String>) -> Self {
        self.line_color = color.into();
        self
    }

    /// 设置线条宽度（对应 Java: Line#setLineWidth）。
    #[must_use]
    pub fn line_width(mut self, width: f64) -> Self {
        self.line_width = width;
        self
    }

    /// 设置线条透明度（对应 Java: Line#setLineOpacity）。
    #[must_use]
    pub fn line_opacity(mut self, opacity: f64) -> Self {
        self.line_opacity = opacity.clamp(0.0, 1.0);
        self
    }

    /// 是否为零长度线条（起点和终点重合）。
    #[must_use]
    pub fn is_zero_length(&self) -> bool {
        (self.begin_point[0] - self.end_point[0]).abs() < f64::EPSILON
            && (self.begin_point[1] - self.end_point[1]).abs() < f64::EPSILON
    }

    /// 是否为有效可绘制线条（线宽大于 0 且非零长度）。
    #[must_use]
    pub fn is_drawable(&self) -> bool {
        self.line_width > 0.0 && !self.is_zero_length()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let line = Line::default();
        assert!((line.line_width - 0.353).abs() < f64::EPSILON);
        assert!((line.line_opacity - 1.0).abs() < f64::EPSILON);
        assert_eq!(line.line_color, "#000000");
    }

    #[test]
    fn test_new() {
        let line = Line::new(100.0, 50.0);
        assert!((line.width - 100.0).abs() < f64::EPSILON);
        assert!((line.height - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_with_position() {
        let line = Line::with_position(10.0, 20.0, 100.0, 50.0);
        assert!((line.x - 10.0).abs() < f64::EPSILON);
        assert!((line.y - 20.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_builders() {
        let line = Line::new(100.0, 50.0)
            .begin_point(0.0, 0.0)
            .end_point(10.0, 10.0)
            .line_color("#FF0000")
            .line_width(1.0)
            .line_opacity(0.5);
        assert!((line.begin_point[0] - 0.0).abs() < f64::EPSILON);
        assert!((line.end_point[0] - 10.0).abs() < f64::EPSILON);
        assert_eq!(line.line_color, "#FF0000");
        assert!((line.line_width - 1.0).abs() < f64::EPSILON);
        assert!((line.line_opacity - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_line_opacity_clamp() {
        let line = Line::new(10.0, 10.0).line_opacity(2.0);
        assert!((line.line_opacity - 1.0).abs() < f64::EPSILON);

        let line = Line::new(10.0, 10.0).line_opacity(-0.5);
        assert!((line.line_opacity - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_is_zero_length() {
        let line = Line::new(10.0, 10.0)
            .begin_point(5.0, 5.0)
            .end_point(5.0, 5.0);
        assert!(line.is_zero_length());
        assert!(!line.is_drawable());

        let line = Line::new(10.0, 10.0)
            .begin_point(0.0, 0.0)
            .end_point(10.0, 10.0);
        assert!(!line.is_zero_length());
        assert!(line.is_drawable());
    }

    #[test]
    fn test_is_drawable_zero_width() {
        let line = Line::new(10.0, 10.0)
            .begin_point(0.0, 0.0)
            .end_point(10.0, 10.0)
            .line_width(0.0);
        assert!(!line.is_drawable());
    }
}
