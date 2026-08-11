//! 基于页面的 OFD 写入构建器。

use easyofd_core::{OfdMetadata, OfdPage, OfdResult};
use easyofd_writer::{OfdWriter, WriteOptions};

/// 基于页面的 OFD 写入构建器（无需模型类型）。
///
/// 由 [`EasyOfd::write_pages(path)`](crate::EasyOfd::write_pages) 创建。
pub struct PageWriterBuilder {
    pub(crate) path: String,
    pub(crate) metadata: OfdMetadata,
}

impl PageWriterBuilder {
    /// 设置文档标题。
    #[must_use]
    pub fn metadata_title(mut self, title: impl Into<String>) -> Self {
        self.metadata.title = Some(title.into());
        self
    }

    /// 设置文档作者。
    #[must_use]
    pub fn metadata_author(mut self, author: impl Into<String>) -> Self {
        self.metadata.author = Some(author.into());
        self
    }

    /// 设置文档创建者。
    #[must_use]
    pub fn metadata_creator(mut self, creator: impl Into<String>) -> Self {
        self.metadata.creator = Some(creator.into());
        self
    }

    /// 执行写入操作。
    ///
    /// # 错误
    ///
    /// ZIP 创建或文件 I/O 失败时返回错误。
    pub fn do_write(&self, pages: Vec<OfdPage>) -> OfdResult<()> {
        let options = WriteOptions {
            metadata: self.metadata.clone(),
        };
        let mut writer = OfdWriter::with_options(options);
        writer.add_pages(pages);
        writer.build_to_file(&self.path)
    }

    /// 执行写入操作并返回 OFD 字节。
    ///
    /// # 错误
    ///
    /// ZIP 创建失败时返回错误。
    pub fn do_write_to_bytes(&self, pages: Vec<OfdPage>) -> OfdResult<Vec<u8>> {
        let options = WriteOptions {
            metadata: self.metadata.clone(),
        };
        let mut writer = OfdWriter::with_options(options);
        writer.add_pages(pages);
        writer.build()
    }
}
