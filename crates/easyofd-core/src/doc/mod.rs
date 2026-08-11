//! 文档结构模块。
//!
//! 包含文档级别的子结构：
//! - [`bookmark`] — 书签集
//! - [`permission`] — 权限控制
//! - [`ct_doc_info`] — 文档元数据信息
//! - [`ct_v_preferences`] — 视图首选项
//! - [`ofd_dir`] — OFD 包根目录
//! - [`doc_dir`] — 文档目录

pub mod bookmark;
pub mod ct_doc_info;
pub mod ct_v_preferences;
pub mod doc_dir;
pub mod ofd_dir;
pub mod permission;
