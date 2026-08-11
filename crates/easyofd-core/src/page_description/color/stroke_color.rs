//! 描边颜色。
//!
//! 对应 Java: org.ofdrw.core.graph.pathObj.StrokeColor

/// 描边颜色。
///
/// 对应 Java: org.ofdrw.core.graph.pathObj.StrokeColor
#[derive(Debug, Clone, Default)]
pub struct StrokeColor {
    /// 颜色值（RGB hex 或颜色空间引用）。
    pub value: Option<u32>,
    /// 颜色空间引用 ID。
    pub color_space_ref: Option<u32>,
    /// Alpha 值（0-255）。
    pub alpha: Option<u8>,
}

impl StrokeColor {
    /// 创建描边颜色。
    #[must_use]
    pub fn new(value: u32) -> Self {
        Self {
            value: Some(value),
            color_space_ref: None,
            alpha: None,
        }
    }

    /// 设置颜色空间引用。
    #[must_use]
    pub fn color_space_ref(mut self, ref_id: u32) -> Self {
        self.color_space_ref = Some(ref_id);
        self
    }

    /// 设置 Alpha 值。
    #[must_use]
    pub fn alpha(mut self, alpha: u8) -> Self {
        self.alpha = Some(alpha);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stroke_color_new() {
        let sc = StrokeColor::new(0x0000_00FF);
        assert_eq!(sc.value, Some(0x0000_00FF));
    }

    #[test]
    fn stroke_color_builder() {
        let sc = StrokeColor::new(0x0000_0000).alpha(255).color_space_ref(2);
        assert_eq!(sc.alpha, Some(255));
    }
}
