//! 基于类型的 OFD 写入构建器。

use easyofd_core::{OfdMetadata, OfdModel, OfdResult};
use easyofd_writer::{OfdWriter, WriteOptions};

/// 基于 `OfdModel` 的 OFD 写入构建器。
///
/// 由 [`EasyOfd::write::<T>(path)`](crate::EasyOfd::write) 创建。
pub struct OfdWriterBuilder<T: OfdModel> {
    pub(crate) path: String,
    pub(crate) _phantom: std::marker::PhantomData<T>,
    pub(crate) metadata: OfdMetadata,
}

impl<T: OfdModel> OfdWriterBuilder<T> {
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
    /// `data` 中的每个元素对应 OFD 文档中的一个页面。
    ///
    /// # 错误
    ///
    /// 模型转换、ZIP 创建或文件 I/O 失败时返回错误。
    pub fn do_write(&self, data: &[T]) -> OfdResult<()> {
        let pages = T::to_pages(data)?;
        let options = WriteOptions {
            metadata: self.metadata.clone(),
        };
        let mut writer = OfdWriter::with_options(options);
        writer.add_pages(pages);
        writer.build_to_file(&self.path)
    }

    /// 执行写入操作并返回 OFD 字节（无文件 I/O）。
    ///
    /// # 错误
    ///
    /// 模型转换或 ZIP 创建失败时返回错误。
    pub fn do_write_to_bytes(&self, data: &[T]) -> OfdResult<Vec<u8>> {
        let pages = T::to_pages(data)?;
        let options = WriteOptions {
            metadata: self.metadata.clone(),
        };
        let mut writer = OfdWriter::with_options(options);
        writer.add_pages(pages);
        writer.build()
    }
}
