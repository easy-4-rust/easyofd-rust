//! 扩展模块。
//!
//! 对应 GB/T 33190 中的扩展机制，包含 3 个类型：
//! - [`Extensions`] — 扩展根节点
//! - [`CtExtension`] — 单个扩展
//! - [`Property`] — 扩展属性

mod ct_extension;
#[allow(clippy::module_inception)]
mod extensions;
mod property;

pub use ct_extension::CtExtension;
pub use extensions::Extensions;
pub use property::Property;
