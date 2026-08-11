//! 工具类。
//!
//! 对应 Java: org.ofdrw.reader.tools

pub mod image_utils;
pub mod namespace_cleaner;
pub mod namespace_modifier;

pub use image_utils::{ImageFormat, detect_format, gray};
pub use namespace_cleaner::NamespaceCleaner;
#[allow(deprecated)]
pub use namespace_modifier::{NamespaceModifier, OFD_NAMESPACE, OFD_PREFIX};
