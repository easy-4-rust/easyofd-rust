//! # `easyofd-tool`
//!
//! OFD 文档工具库，对应 Java 版 [`ofdrw-tool`](https://github.com/ofdrw/ofdrw) 子项目。
//!
//! ## 功能模块
//!
//! - [`merge`] — OFD 文档合并（OfdMerger、DocContext、DocPage、BareOFDDoc）
//! - [`page_deleter`] — OFD 页面删除（OfdPageDeleter）
//!
//! ## 对应 Java 类型
//!
//! | Java 类型 | Rust 类型 | 说明 |
//! |-----------|-----------|------|
//! | `org.ofdrw.tool.merge.DocContext` | [`merge::DocContext`] | 合并上下文 |
//! | `org.ofdrw.tool.merge.DocPage` | [`merge::DocPage`] | 文档页面描述 |
//! | `org.ofdrw.tool.merge.BareOFDDoc` | [`merge::BareOFDDoc`] | 裸 OFD 文档 |
//! | `org.ofdrw.tool.merge.OFDMerger` | [`merge::OfdMerger`] | OFD 合并器 |
//! | `org.ofdrw.tool.merge.PageEntry` | [`merge::PageEntry`] | 页面项目 |
//! | `org.ofdrw.tool.merge.OFDPageDeleter` | [`page_deleter::OfdPageDeleter`] | 页面删除器 |

pub mod merge;
pub mod page_deleter;

pub use merge::{BareOFDDoc, DocContext, DocPage, OfdMerger, PageEntry, ResourceDedup};
pub use page_deleter::OfdPageDeleter;

/// 对应 Java: OFDMerger（Rust 命名别名）。
pub type OFDMerger = OfdMerger;

/// 对应 Java: OFDPageDeleter（Rust 命名别名）。
pub type OFDPageDeleter = OfdPageDeleter;
