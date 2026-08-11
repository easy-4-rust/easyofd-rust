//! 图片媒体类型模块。
//!
//! 对应 Java: org.ofdrw.converter.image

pub mod image_media;

pub use image_media::ImageMedia;

// 重导出外部 image crate 的常用 API，使 lib.rs 中的
// `image::load_from_memory` 等调用能正确解析到外部 crate。
pub use ::image::{ImageBuffer, ImageEncoder, ImageFormat, Rgb, load_from_memory, open};
