//! 复合对象子包 (compositeObj)。
//!
//! 对应 GB/T 33190-2016 第 13.6 节，包含复合对象容器、矢量图形复合对象
//! 以及矢量内容描述。

mod content;
mod ct_composite;
mod ct_vector_g;

pub use content::Content;
pub use ct_composite::CT_Composite;
pub use ct_vector_g::CT_VectorG;
