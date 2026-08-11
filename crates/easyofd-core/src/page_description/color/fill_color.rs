//! 填充颜色。
//!
//! 对应 Java: org.ofdrw.core.graph.pathObj.FillColor

/// 填充颜色。
///
/// 对应 Java: org.ofdrw.core.graph.pathObj.FillColor
#[derive(Debug, Clone, Default)]
pub struct FillColor {
    /// 颜色值（RGB hex 或颜色空间引用）。
    pub value: Option<u32>,
    /// 颜色空间引用 ID。
    pub color_space_ref: Option<u32>,
    /// Alpha 值（0-255）。
    pub alpha: Option<u8>,
}

impl FillColor {
    /// 创建填充颜色。
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
    fn fill_color_new() {
        let fc = FillColor::new(0x00FF_0000);
        assert_eq!(fc.value, Some(0x00FF_0000));
    }

    #[test]
    fn fill_color_builder() {
        let fc = FillColor::new(0x0000_FF00).alpha(128).color_space_ref(1);
        assert_eq!(fc.alpha, Some(128));
        assert_eq!(fc.color_space_ref, Some(1));
    }
}
