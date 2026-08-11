//! OFD-A 转换模块。
//!
//! 对应 Java: org.ofdrw.archive.convert

pub mod archive_handler;
pub mod handler;
pub mod ofd_archive_converter;

pub use archive_handler::ArchiveHandler;
pub use handler::*;
pub use ofd_archive_converter::OfdArchiveConverter;
