//! 文档缩放设置。
//!
//! 对应 Java: org.ofdrw.core.basicStructure.doc.vpreferences.zoom.Zoom

/// 文档查看缩放比例（ofd:Zoom）。
///
/// 对应 Java: ofdrw Zoom，`value` 为百分比缩放值。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Zoom {
    /// 缩放值（百分比，如 100 表示 100%）。
    pub value: f64,
}

impl Zoom {
    /// 创建缩放设置（对应 Java: Zoom(value)）。
    #[must_use]
    pub fn new(value: f64) -> Self {
        Self { value }
    }

    /// 设置缩放值（对应 Java: Zoom#setValue）。
    #[must_use]
    pub fn with_value(mut self, value: f64) -> Self {
        self.value = value;
        self
    }
}

impl From<f64> for Zoom {
    fn from(value: f64) -> Self {
        Self::new(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zoom_new() {
        let z = Zoom::new(100.0);
        assert!((z.value - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_with_value_and_from() {
        let z = Zoom::new(50.0).with_value(75.0);
        assert!((z.value - 75.0).abs() < f64::EPSILON);
        assert!((Zoom::from(120.0).value - 120.0).abs() < f64::EPSILON);
    }
}
