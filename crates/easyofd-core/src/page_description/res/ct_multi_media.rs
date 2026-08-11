//! 多媒体资源描述。
//!
//! 对应 Java: org.ofdrw.core.basicStructure.res.CT_MultiMedia

use super::MediaType;

/// 多媒体资源描述。
///
/// 对应 Java: org.ofdrw.core.basicStructure.res.CT_MultiMedia
#[derive(Debug, Clone)]
pub struct CT_MultiMedia {
    /// 资源 ID。
    pub id: u32,
    /// 多媒体类型。
    pub media_type: MediaType,
    /// 文件格式（如 PNG、JPEG）。
    pub format: Option<String>,
    /// 文件路径。
    pub file: Option<String>,
}

impl CT_MultiMedia {
    /// 创建多媒体资源。
    #[must_use]
    pub fn new(id: u32, media_type: MediaType) -> Self {
        Self {
            id,
            media_type,
            format: None,
            file: None,
        }
    }

    /// 设置格式。
    #[must_use]
    pub fn format(mut self, format: impl Into<String>) -> Self {
        self.format = Some(format.into());
        self
    }

    /// 设置文件路径。
    #[must_use]
    pub fn file(mut self, file: impl Into<String>) -> Self {
        self.file = Some(file.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ct_multi_media_new() {
        let mm = CT_MultiMedia::new(1, MediaType::Image);
        assert_eq!(mm.id, 1);
        assert_eq!(mm.media_type, MediaType::Image);
        assert!(mm.format.is_none());
    }

    #[test]
    fn ct_multi_media_builder() {
        let mm = CT_MultiMedia::new(2, MediaType::Image)
            .format("PNG")
            .file("image.png");
        assert_eq!(mm.format.unwrap(), "PNG");
        assert_eq!(mm.file.unwrap(), "image.png");
    }
}
