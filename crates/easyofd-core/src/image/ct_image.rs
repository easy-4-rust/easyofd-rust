//! 图像对象。
//!
//! 对应 Java: org.ofdrw.core.image.CT_Image

/// 图像对象。
///
/// 对应 Java: org.ofdrw.core.image.CT_Image
#[allow(non_camel_case_types)]
#[derive(Debug, Clone)]
pub struct CT_Image {
    /// 对象 ID。
    pub id: u32,
    /// 边界框 "x y width height"。
    pub boundary: String,
    /// 资源 ID（引用 MultiMedia 中的图像资源）。
    pub resource_id: u32,
    /// 是否插值绘制。
    pub interpolate: bool,
}

impl CT_Image {
    /// 创建图像对象。
    #[must_use]
    pub fn new(id: u32, boundary: impl Into<String>, resource_id: u32) -> Self {
        Self {
            id,
            boundary: boundary.into(),
            resource_id,
            interpolate: false,
        }
    }

    /// 设置插值绘制。
    #[must_use]
    pub fn interpolate(mut self, interpolate: bool) -> Self {
        self.interpolate = interpolate;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ct_image_new() {
        let img = CT_Image::new(1, "0 0 100 100", 5);
        assert_eq!(img.id, 1);
        assert_eq!(img.resource_id, 5);
        assert!(!img.interpolate);
    }

    #[test]
    fn ct_image_builder() {
        let img = CT_Image::new(2, "10 20 50 50", 3).interpolate(true);
        assert!(img.interpolate);
    }
}
