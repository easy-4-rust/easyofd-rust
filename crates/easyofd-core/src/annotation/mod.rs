//! 注释子包（GB/T 33190 第 16 章）。
//!
//! 提供 OFD 文档注释相关的数据类型。

mod ann_page;
mod annot;
mod annot_type;
mod annotations;
mod appearance;
mod page_annot;

pub use ann_page::AnnPage;
pub use annot::Annot;
pub use annot_type::AnnotType;
pub use annotations::Annotations;
pub use appearance::Appearance;
pub use page_annot::PageAnnot;
