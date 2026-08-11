//! 附件子包（GB/T 33190 第 17 章）。
//!
//! 提供 OFD 文档附件相关的数据类型。

mod attachments;
mod ct_attachment;

pub use attachments::Attachments;
pub use ct_attachment::CTAttachment;
