//! OFD 文档合并模块。
//!
//! 对应 Java: org.ofdrw.tool.merge
//!
//! 提供将多个 OFD 文档合并为一个的功能。

mod bare_ofd_doc;
mod doc_context;
mod doc_page;
mod ofd_merger;
mod page_entry;
mod resource_dedup;

pub use bare_ofd_doc::BareOFDDoc;
pub use doc_context::DocContext;
pub use doc_page::DocPage;
pub use ofd_merger::OfdMerger;
pub use page_entry::PageEntry;
pub use resource_dedup::ResourceDedup;
