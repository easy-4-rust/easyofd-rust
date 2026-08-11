//! 版本信息模块。
//!
//! 对应 GB/T 33190 第 8 章"版本"，包含 5 个类型：
//! - [`Versions`] — 版本列表入口
//! - [`Version`] — 单个版本描述
//! - [`DocVersion`] — 文档版本
//! - [`FileList`] — 文件列表
//! - [`File`] — 单个文件信息

mod doc_version;
mod file;
mod file_list;
mod version;
#[allow(clippy::module_inception)]
mod versions;

pub use doc_version::DocVersion;
pub use file::File;
pub use file_list::FileList;
pub use version::Version;
pub use versions::Versions;
