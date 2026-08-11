//! 签章注释实体（StampAnnotEntity）。
//!
//! 对应 Java: org.ofdrw.reader.model.StampAnnotEntity
//!
//! OFD 中的签章信息，包含签名信息、印章图片数据和图片类型。

use super::signed_info::SignedInfo;
use super::stamp_annot::StampAnnot;

/// 印章图片类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SealImageType {
    /// OFD 格式。
    OFD,
    /// PNG 格式。
    PNG,
    /// GIF 格式。
    GIF,
    /// SVG 格式。
    SVG,
    /// JPEG 格式。
    JPEG,
    /// 未知格式。
    Unknown,
}

impl SealImageType {
    /// 从 MIME 类型字符串解析。
    #[must_use]
    pub fn from_mime(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "ofd" => Self::OFD,
            "png" => Self::PNG,
            "gif" => Self::GIF,
            "svg" => Self::SVG,
            "jpeg" | "jpg" => Self::JPEG,
            _ => Self::Unknown,
        }
    }

    /// 获取类型名称。
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::OFD => "OFD",
            Self::PNG => "PNG",
            Self::GIF => "GIF",
            Self::SVG => "SVG",
            Self::JPEG => "JPEG",
            Self::Unknown => "Unknown",
        }
    }
}

impl std::fmt::Display for SealImageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// OFD 中的签章信息实体。
///
/// 封装签名信息、印章图片数据和图片类型，
/// 便于读取和处理 OFD 文档中的签章。
#[derive(Debug, Clone)]
pub struct StampAnnotEntity {
    /// 签名要保护的原文及本次签名相关的信息。
    pub signed_info: SignedInfo,
    /// 印章图片数据（可能是 OFD、PNG、GIF、SVG 等）。
    pub image_byte: Vec<u8>,
    /// 印章图片类型。
    pub img_type: SealImageType,
}

impl StampAnnotEntity {
    /// 创建新的签章注释实体。
    #[must_use]
    pub fn new(signed_info: SignedInfo, image_byte: Vec<u8>, img_type: SealImageType) -> Self {
        Self {
            signed_info,
            image_byte,
            img_type,
        }
    }

    /// 获取签名的外观序列。
    pub fn stamp_annots(&self) -> &[StampAnnot] {
        &self.signed_info.stamp_annots
    }

    /// 获取图片数据。
    pub fn image_byte(&self) -> &[u8] {
        &self.image_byte
    }

    /// 获取印章图片类型。
    pub fn img_type(&self) -> SealImageType {
        self.img_type
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signatures::reference::Reference;
    use crate::signatures::references::References;
    use crate::signatures::signed_info::Provider;

    fn make_signed_info() -> SignedInfo {
        SignedInfo::new(
            Provider::new("TestProvider"),
            References::new().add_reference(Reference::new("/Doc_0/Document.xml", "hash")),
        )
    }

    #[test]
    fn test_stamp_annot_entity_new() {
        let entity = StampAnnotEntity::new(
            make_signed_info(),
            vec![0x89, 0x50, 0x4E, 0x47],
            SealImageType::PNG,
        );
        assert_eq!(entity.image_byte(), &[0x89, 0x50, 0x4E, 0x47]);
        assert_eq!(entity.img_type(), SealImageType::PNG);
        assert!(entity.stamp_annots().is_empty());
    }

    #[test]
    fn test_stamp_annot_entity_with_annots() {
        use crate::basic_type::{ST_Box, ST_RefID};

        let si = make_signed_info().add_stamp_annot(StampAnnot::new(
            "s1",
            ST_RefID::new(1),
            ST_Box::new(0.0, 0.0, 100.0, 100.0),
        ));
        let entity = StampAnnotEntity::new(si, vec![1, 2, 3], SealImageType::OFD);
        assert_eq!(entity.stamp_annots().len(), 1);
        assert_eq!(entity.stamp_annots()[0].id, "s1");
    }

    #[test]
    fn test_seal_image_type_from_mime() {
        assert_eq!(SealImageType::from_mime("PNG"), SealImageType::PNG);
        assert_eq!(SealImageType::from_mime("png"), SealImageType::PNG);
        assert_eq!(SealImageType::from_mime("jpg"), SealImageType::JPEG);
        assert_eq!(SealImageType::from_mime("svg"), SealImageType::SVG);
        assert_eq!(SealImageType::from_mime("bmp"), SealImageType::Unknown);
    }

    #[test]
    fn test_seal_image_type_display() {
        assert_eq!(SealImageType::PNG.to_string(), "PNG");
        assert_eq!(SealImageType::OFD.to_string(), "OFD");
    }
}
