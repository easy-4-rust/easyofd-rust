use std::path::{Path, PathBuf};

use easyofd_core::{OfdPage, OfdResult};
use easyofd_package::PackageLimits;
use easyofd_reader::{OfdReader, ReadOptions};

/// 逐页读取 OFD 的流畅配置入口。
#[derive(Debug, Clone)]
pub struct OfdReadBuilder {
    source: PathBuf,
    options: ReadOptions,
}

impl OfdReadBuilder {
    /// 从 OFD 文件创建逐页读取构建器。
    #[must_use]
    pub fn new(source: impl AsRef<Path>) -> Self {
        Self {
            source: source.as_ref().to_path_buf(),
            options: ReadOptions::default(),
        }
    }

    /// 限制读取页码范围，页码从 1 开始且包含边界。
    #[must_use]
    pub fn page_range(mut self, first: usize, last: usize) -> Self {
        self.options.first_page = Some(first);
        self.options.last_page = Some(last);
        self
    }

    /// 配置 ZIP 包安全限制。
    #[must_use]
    pub fn package_limits(mut self, limits: PackageLimits) -> Self {
        self.options.package_limits = limits;
        self
    }

    /// 逐页读取并调用处理函数，不保留已处理页面。
    ///
    /// # Errors
    ///
    /// OFD 解析或处理函数失败时返回错误。
    pub fn do_read(self, visitor: impl FnMut(usize, OfdPage) -> OfdResult<()>) -> OfdResult<usize> {
        OfdReader::visit_path(self.source, self.options, visitor)
    }

    /// 显式将选定页面完整加载到内存。
    ///
    /// # Errors
    ///
    /// OFD 解析失败时返回错误。
    pub fn read_document(self) -> OfdResult<OfdReader> {
        OfdReader::open_with_options(self.source, self.options)
    }
}
