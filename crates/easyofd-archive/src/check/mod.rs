//! OFD-A 合规检查模块。
//!
//! 对应 Java: org.ofdrw.archive.check

pub mod archive_rule;
pub mod archive_violation;
pub mod ofd_archive_checker;
pub mod rule;

pub use archive_rule::ArchiveRule;
pub use archive_violation::{ArchiveViolation, Severity};
pub use ofd_archive_checker::OfdArchiveChecker;
