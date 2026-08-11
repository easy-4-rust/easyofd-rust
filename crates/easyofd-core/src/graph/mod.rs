//! 图形子包 (graph)。
//!
//! 对应 GB/T 33190-2016 第 9 节，包含路径对象和路径缩写数据等类型。

mod abbreviated_data;
mod ct_path;

pub use abbreviated_data::{AbbreviatedData, PathCommand};
pub use ct_path::{CT_Path, FillRule};
