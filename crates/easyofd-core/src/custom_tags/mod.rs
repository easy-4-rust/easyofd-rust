//! 自定义标引模块。
//!
//! 包含 2 个类型：
//! - [`CustomTags`] — 标引入口
//! - [`CustomTag`] — 单个标引

mod custom_tag;
#[allow(clippy::module_inception)]
mod custom_tags;

pub use custom_tag::CustomTag;
pub use custom_tags::CustomTags;
