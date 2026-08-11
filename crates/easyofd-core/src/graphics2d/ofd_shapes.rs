//! OFD 形状集合。
//!
//! 对应 Java: org.ofdrw.graphics2d.OFDShapes
//!
//! Java 版提供 `java.awt.Shape` 实现集合（矩形、椭圆、圆弧等）。
//! Rust 版提供简化枚举，列出常用形状类型。

/// 形状类型枚举。
///
/// 对应 Java: org.ofdrw.graphics2d.OFDShapes
///
/// 列出 OFD 绘图中常用的 2D 形状类型。
/// Java 版每个变体是独立的 `Shape` 实现类；Rust 版合并为枚举。
#[derive(Debug, Clone, PartialEq)]
#[allow(missing_docs)]
pub enum OfdShape {
    /// 矩形。
    Rect { x: f64, y: f64, w: f64, h: f64 },
    /// 椭圆。
    Ellipse { cx: f64, cy: f64, rx: f64, ry: f64 },
    /// 圆。
    Circle { cx: f64, cy: f64, r: f64 },
    /// 线段。
    Line { x1: f64, y1: f64, x2: f64, y2: f64 },
    /// 圆弧。
    Arc {
        cx: f64,
        cy: f64,
        rx: f64,
        ry: f64,
        start_angle: f64,
        extent: f64,
    },
}

/// OFD 形状集合容器。
///
/// 对应 Java: org.ofdrw.graphics2d.OFDShapes
///
/// 持有一组待绘制的形状。
#[derive(Debug, Clone, Default)]
pub struct OfdShapes {
    /// 形状列表。
    shapes: Vec<OfdShape>,
}

impl OfdShapes {
    /// 创建空形状集合。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 添加形状。
    pub fn push(&mut self, shape: OfdShape) {
        self.shapes.push(shape);
    }

    /// 链式添加形状。
    #[must_use]
    pub fn with(mut self, shape: OfdShape) -> Self {
        self.shapes.push(shape);
        self
    }

    /// 获取形状数量。
    #[must_use]
    pub fn len(&self) -> usize {
        self.shapes.len()
    }

    /// 是否为空。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.shapes.is_empty()
    }

    /// 获取形状列表引用。
    #[must_use]
    pub fn shapes(&self) -> &[OfdShape] {
        &self.shapes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty() {
        let s = OfdShapes::new();
        assert!(s.is_empty());
        assert_eq!(s.len(), 0);
    }

    #[test]
    fn test_push_and_len() {
        let mut s = OfdShapes::new();
        s.push(OfdShape::Rect {
            x: 0.0,
            y: 0.0,
            w: 10.0,
            h: 20.0,
        });
        s.push(OfdShape::Circle {
            cx: 5.0,
            cy: 5.0,
            r: 3.0,
        });
        assert_eq!(s.len(), 2);
        assert!(!s.is_empty());
    }

    #[test]
    fn test_with_chain() {
        let s = OfdShapes::new()
            .with(OfdShape::Line {
                x1: 0.0,
                y1: 0.0,
                x2: 10.0,
                y2: 10.0,
            })
            .with(OfdShape::Ellipse {
                cx: 5.0,
                cy: 5.0,
                rx: 3.0,
                ry: 2.0,
            });
        assert_eq!(s.len(), 2);
    }

    #[test]
    fn test_shape_variants() {
        let rect = OfdShape::Rect {
            x: 1.0,
            y: 2.0,
            w: 3.0,
            h: 4.0,
        };
        let ellipse = OfdShape::Ellipse {
            cx: 0.0,
            cy: 0.0,
            rx: 5.0,
            ry: 3.0,
        };
        let circle = OfdShape::Circle {
            cx: 1.0,
            cy: 1.0,
            r: 2.0,
        };
        let line = OfdShape::Line {
            x1: 0.0,
            y1: 0.0,
            x2: 1.0,
            y2: 1.0,
        };
        let arc = OfdShape::Arc {
            cx: 0.0,
            cy: 0.0,
            rx: 1.0,
            ry: 1.0,
            start_angle: 0.0,
            extent: 90.0,
        };
        // 验证 Debug 不 panic
        let _ = format!("{rect:?} {ellipse:?} {circle:?} {line:?} {arc:?}");
    }

    #[test]
    fn test_clone_eq() {
        let s = OfdShapes::new().with(OfdShape::Rect {
            x: 0.0,
            y: 0.0,
            w: 1.0,
            h: 1.0,
        });
        let s2 = s.clone();
        assert_eq!(s.len(), s2.len());
        assert_eq!(s.shapes(), s2.shapes());
    }
}
