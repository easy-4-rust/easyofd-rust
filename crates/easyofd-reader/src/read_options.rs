//! 读取选项。

use easyofd_package::PackageLimits;

/// OFD 读取选项。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ReadOptions {
    /// 第一个读取页码，使用从 1 开始的页码。
    pub first_page: Option<usize>,
    /// 最后一个读取页码，使用从 1 开始的页码。
    pub last_page: Option<usize>,
    /// ZIP 包安全限制。
    pub package_limits: PackageLimits,
}
