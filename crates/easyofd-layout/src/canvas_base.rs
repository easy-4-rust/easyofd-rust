//! Canvas 扩展基类，用于快速构建基于 Canvas 扩展的自定义元素。
//!
//! 对应 Java: org.ofdrw.layout.element.canvas.CanvasBase
//!
//! 通过 CanvasBase 可以简化 Canvas 的创建过程。实现者需要实现 `Drawer` trait
//! 并在 `draw` 方法中实现自定义绘制逻辑。
//!
//! 一个最简单扩展示例可参考 `Line`。

/// Canvas 扩展基类。
///
/// 对应 Java: ofdrw layout canvas CanvasBase（abstract class extends Canvas implements Drawer）。
///
/// 在 Rust 中以结构体 + trait 实现的组合模式表达。CanvasBase 包含 Canvas 的基础属性
/// 并可选地持有绘制器。
#[derive(Debug, Clone)]
pub struct CanvasBase {
    /// 左上角 X 坐标（mm）。
    pub x: f64,
    /// 左上角 Y 坐标（mm）。
    pub y: f64,
    /// 画布宽度（mm）。
    pub width: f64,
    /// 画布高度（mm）。
    pub height: f64,
}

impl CanvasBase {
    /// 创建 Canvas 基类（对应 Java: CanvasBase(width, height)）。
    #[must_use]
    pub fn new(width: f64, height: f64) -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            width,
            height,
        }
    }

    /// 创建带位置的 Canvas 基类（对应 Java: CanvasBase(x, y, w, h)）。
    #[must_use]
    pub fn with_position(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// 面积（mm^2）。
    #[must_use]
    pub fn area(&self) -> f64 {
        self.width * self.height
    }

    /// 是否为空画布（宽或高为 0）。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.width <= 0.0 || self.height <= 0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let canvas = CanvasBase::new(100.0, 50.0);
        assert!((canvas.x - 0.0).abs() < f64::EPSILON);
        assert!((canvas.y - 0.0).abs() < f64::EPSILON);
        assert!((canvas.width - 100.0).abs() < f64::EPSILON);
        assert!((canvas.height - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_with_position() {
        let canvas = CanvasBase::with_position(10.0, 20.0, 100.0, 50.0);
        assert!((canvas.x - 10.0).abs() < f64::EPSILON);
        assert!((canvas.y - 20.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_area() {
        let canvas = CanvasBase::new(10.0, 5.0);
        assert!((canvas.area() - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_is_empty() {
        assert!(CanvasBase::new(0.0, 10.0).is_empty());
        assert!(CanvasBase::new(10.0, 0.0).is_empty());
        assert!(!CanvasBase::new(10.0, 10.0).is_empty());
    }
}
