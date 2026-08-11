//! OFD reader 数据模型。
//!
//! 对应 Java: org.ofdrw.reader.model

mod annotation_entity;
mod ofd_document_vo;
mod ofd_page_vo;
mod seal_data_vo;
mod stamp_annot_vo;
mod template_page_entity;

pub use annotation_entity::AnnotionEntity;
#[allow(deprecated)]
pub use ofd_document_vo::OfdDocumentVo;
#[allow(deprecated)]
pub use ofd_page_vo::OfdPageVo;
pub use seal_data_vo::SealDataVo;
#[allow(deprecated)]
pub use stamp_annot_vo::StampAnnotVo;
pub use template_page_entity::{TemplatePageEntity, TemplateZOrder};
