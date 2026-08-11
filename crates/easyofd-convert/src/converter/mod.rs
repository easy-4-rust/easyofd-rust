//! 转换器类型模块。
//!
//! 对应 Java: org.ofdrw.converter (部分类型)

pub mod cg_transform_map;
pub mod doc_converter;

pub use cg_transform_map::{CgTransformEntry, CgTransformMap};
pub use doc_converter::{
    DocConverter, ImageConverterConfig, PdfConverterConfig, TextConverterConfig,
};
