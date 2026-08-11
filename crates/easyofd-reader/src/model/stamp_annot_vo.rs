//! 签章注释视图对象。
//!
//! 对应 Java: org.ofdrw.reader.model.StampAnnotVo
//!
//! 已废弃：Java 原始类标记为 `@Deprecated`，建议使用 `StampAnnotEntity`。

/// 签章注释视图对象，包含签章注释列表和印章图片数据。
///
/// 对应 Java: `org.ofdrw.reader.model.StampAnnotVo`
///
/// **废弃**：Java 原始类标记为 `@Deprecated`，
/// 建议使用 [`StampAnnotEntity`](easyofd_core::StampAnnotEntity)。
#[derive(Debug, Clone)]
#[deprecated(since = "1.0.0", note = "使用 easyofd_core::StampAnnotEntity 替代")]
#[allow(deprecated)]
pub struct StampAnnotVo {
    /// 签章注释的 XML 描述列表。
    pub stamp_annots: Vec<String>,
    /// 印章图片的原始字节。
    pub img_byte: Vec<u8>,
    /// 印章图片类型（如 "PNG"、"JPEG"）。
    pub image_type: String,
}

#[allow(deprecated)]
impl StampAnnotVo {
    /// 创建新的签章注释视图对象。
    #[must_use]
    pub fn new() -> Self {
        Self {
            stamp_annots: Vec::new(),
            img_byte: Vec::new(),
            image_type: String::new(),
        }
    }

    /// 设置印章图片数据。
    pub fn set_img_byte(&mut self, data: Vec<u8>) {
        self.img_byte = data;
    }

    /// 设置印章图片类型。
    pub fn set_image_type(&mut self, image_type: impl Into<String>) {
        self.image_type = image_type.into();
    }

    /// 添加签章注释描述。
    pub fn add_stamp_annot(&mut self, annot: impl Into<String>) {
        self.stamp_annots.push(annot.into());
    }
}

#[allow(deprecated)]
impl Default for StampAnnotVo {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(deprecated)]
#[cfg(test)]
mod tests {
    #[allow(deprecated)]
    use super::*;

    #[test]
    #[allow(deprecated)]
    fn test_stamp_annot_vo_new() {
        let vo = StampAnnotVo::new();
        assert!(vo.stamp_annots.is_empty());
        assert!(vo.img_byte.is_empty());
        assert!(vo.image_type.is_empty());
    }

    #[test]
    #[allow(deprecated)]
    fn test_stamp_annot_vo_setters() {
        let mut vo = StampAnnotVo::new();
        vo.set_img_byte(vec![0x89, 0x50, 0x4E, 0x47]);
        vo.set_image_type("PNG");
        vo.add_stamp_annot("<StampAnnot/>");
        assert_eq!(vo.img_byte.len(), 4);
        assert_eq!(vo.image_type, "PNG");
        assert_eq!(vo.stamp_annots.len(), 1);
    }
}
