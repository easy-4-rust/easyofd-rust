//! 印章数据视图对象。
//!
//! 对应 Java: org.ofdrw.reader.model.SealDataVo

/// 印章数据视图对象，包含印章的基本信息。
///
/// 对应 Java: `org.ofdrw.reader.model.SealDataVo`
#[derive(Debug, Clone)]
pub struct SealDataVo {
    /// 印章 ID。
    pub seal_id: String,
    /// 印章名称。
    pub seal_name: Option<String>,
    /// 印章图片数据。
    pub img_byte: Vec<u8>,
    /// 印章图片类型（如 "PNG"、"JPEG"）。
    pub image_type: String,
}

impl SealDataVo {
    /// 创建新的印章数据视图对象。
    #[must_use]
    pub fn new(seal_id: impl Into<String>) -> Self {
        Self {
            seal_id: seal_id.into(),
            seal_name: None,
            img_byte: Vec::new(),
            image_type: String::new(),
        }
    }

    /// 设置印章名称。
    #[must_use]
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.seal_name = Some(name.into());
        self
    }

    /// 设置印章图片数据。
    #[must_use]
    pub fn with_img(mut self, data: Vec<u8>, image_type: impl Into<String>) -> Self {
        self.img_byte = data;
        self.image_type = image_type.into();
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seal_data_vo_new() {
        let vo = SealDataVo::new("seal_001");
        assert_eq!(vo.seal_id, "seal_001");
        assert!(vo.seal_name.is_none());
        assert!(vo.img_byte.is_empty());
    }

    #[test]
    fn test_seal_data_vo_with_name() {
        let vo = SealDataVo::new("seal_001").with_name("Company Seal");
        assert_eq!(vo.seal_name.as_deref(), Some("Company Seal"));
    }

    #[test]
    fn test_seal_data_vo_with_img() {
        let data = vec![0x89, 0x50, 0x4E, 0x47];
        let vo = SealDataVo::new("seal_001").with_img(data.clone(), "PNG");
        assert_eq!(vo.img_byte, data);
        assert_eq!(vo.image_type, "PNG");
    }
}
