//! 元素区域（ofd:Area）。
//!
//! 对应 Java: org.ofdrw.core.basicStructure.pageObj.layer.area.Area

use crate::basic_type::ST_Array;

/// 元素区域（ofd:Area），定义绘制参数、变换矩阵与裁剪。
///
/// 对应 Java: ofdrw Area。
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Area {
    /// 绘制参数引用（ST_RefID，可选）。
    pub draw_param: Option<u32>,
    /// 变换矩阵（ST_Array，6 元素 a b c d e f，可选）。
    pub ctm: Option<ST_Array>,
    /// 裁剪区域（原始 XML 内容，可选）。
    pub clip: Option<String>,
}

impl Area {
    /// 创建空区域（对应 Java: Area()）。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置绘制参数引用（对应 Java: Area#setDrawParam）。
    #[must_use]
    pub fn draw_param(mut self, id: u32) -> Self {
        self.draw_param = Some(id);
        self
    }

    /// 设置变换矩阵（对应 Java: Area#setCTM）。
    #[must_use]
    pub fn ctm(mut self, ctm: ST_Array) -> Self {
        self.ctm = Some(ctm);
        self
    }

    /// 设置裁剪区域（对应 Java: Area#setClipObj）。
    #[must_use]
    pub fn clip(mut self, clip: impl Into<String>) -> Self {
        self.clip = Some(clip.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_area_default() {
        let a = Area::new();
        assert!(a.draw_param.is_none());
        assert!(a.ctm.is_none());
    }

    #[test]
    fn test_area_builders() {
        let a = Area::new()
            .draw_param(5)
            .ctm(ST_Array::transform(1.0, 0.0, 0.0, 1.0, 0.0, 0.0))
            .clip("M0 0 L10 0 L10 10 Z");
        assert_eq!(a.draw_param, Some(5));
        assert!(a.ctm.is_some());
        assert_eq!(a.clip.as_deref(), Some("M0 0 L10 0 L10 10 Z"));
    }
}
