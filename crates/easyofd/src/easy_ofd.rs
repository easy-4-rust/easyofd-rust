//! OFD 操作统一入口。

use easyofd_core::{OfdMetadata, OfdPage, OfdResult};
use easyofd_markdown::MarkdownConversionBuilder;
use easyofd_reader::OfdReader;
use easyofd_template::OfdTemplateFiller;
use easyofd_writer::{OfdStreamWriter, OfdWriter, WriteOptions};

use crate::ofd_read_builder::OfdReadBuilder;
use crate::ofd_writer_builder::OfdWriterBuilder;
use crate::page_writer_builder::PageWriterBuilder;

/// OFD 操作统一入口，对标 EasyExcel 的使用体验。
///
/// 所有方法均为静态方法，返回 Builder 或直接执行操作。
/// 遵循 fluent builder 模式，支持链式调用。
///
/// # 用法示例
///
/// ```rust,ignore
/// use easyofd::EasyOfd;
///
/// // 写入
/// EasyOfd::write("output.ofd").do_write(&data)?;
///
/// // 读取
/// let reader = EasyOfd::read("input.ofd")?;
///
/// // 转 Markdown
/// let md = EasyOfd::to_markdown("input.ofd").do_convert()?;
///
/// // 模板填充
/// EasyOfd::fill_template("template.ofd", &data)?.save("output.ofd")?;
/// ```
pub struct EasyOfd;

impl EasyOfd {
    /// 启动基于类型的写入操作，使用 `OfdModel` 派生宏生成的类型。
    ///
    /// 返回 [`OfdWriterBuilder`]，支持链式配置元数据。
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// use easyofd::{EasyOfd, OfdModel};
    ///
    /// #[derive(OfdModel)]
    /// #[ofd(page_width = 210.0, page_height = 297.0)]
    /// struct Invoice {
    ///     #[ofd(x = 20.0, y = 30.0, size = 18.0, bold)]
    ///     title: String,
    /// }
    ///
    /// EasyOfd::write::<Invoice>("output.ofd")
    ///     .metadata_title("发票")
    ///     .do_write(&data)?;
    /// ```
    pub fn write<T: easyofd_core::OfdModel>(path: impl Into<String>) -> OfdWriterBuilder<T> {
        OfdWriterBuilder {
            path: path.into(),
            _phantom: std::marker::PhantomData,
            metadata: OfdMetadata::default(),
        }
    }

    /// 启动基于页面的写入操作（无需 OfdModel 类型）。
    ///
    /// 返回 [`PageWriterBuilder`]，支持链式配置元数据。
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// use easyofd::{EasyOfd, OfdPage, TextObject};
    ///
    /// let mut page = OfdPage::new(210.0, 297.0);
    /// page.add_text(TextObject::new(20.0, 30.0, "Hello OFD!"));
    ///
    /// EasyOfd::write_pages("output.ofd")
    ///     .metadata_title("我的文档")
    ///     .do_write(vec![page])?;
    /// ```
    pub fn write_pages(path: impl Into<String>) -> PageWriterBuilder {
        PageWriterBuilder {
            path: path.into(),
            metadata: OfdMetadata::default(),
        }
    }

    /// 创建流式 OFD Writer，页面在调用 `write_page` 时直接写入输出。
    ///
    /// 适合大文档场景，内存占用恒定（每页独立写入）。
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// use easyofd::{EasyOfd, OfdPage, TextObject};
    ///
    /// let file = std::fs::File::create("large.ofd")?;
    /// let mut writer = EasyOfd::stream_writer(file);
    /// for i in 1..=100_000 {
    ///     let mut page = OfdPage::new(210.0, 297.0);
    ///     page.add_text(TextObject::new(10.0, 10.0, format!("Page {i}")));
    ///     writer.write_page(page)?;
    /// }
    /// writer.finish()?;
    /// ```
    #[must_use]
    pub fn stream_writer<W: std::io::Write + std::io::Seek>(output: W) -> OfdStreamWriter<W> {
        OfdStreamWriter::new(output, WriteOptions::default())
    }

    /// 使用自定义选项创建流式 OFD Writer。
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// use easyofd::{EasyOfd, WriteOptions, OfdMetadata};
    ///
    /// let options = WriteOptions {
    ///     metadata: OfdMetadata { title: Some("大文档".into()), ..Default::default() },
    /// };
    /// let file = std::fs::File::create("output.ofd")?;
    /// let mut writer = EasyOfd::stream_writer_with_options(file, options);
    /// ```
    #[must_use]
    pub fn stream_writer_with_options<W: std::io::Write + std::io::Seek>(
        output: W,
        options: WriteOptions,
    ) -> OfdStreamWriter<W> {
        OfdStreamWriter::new(output, options)
    }

    /// 将页面直接写入文件（一次性操作）。
    ///
    /// # 错误
    ///
    /// ZIP 创建或文件 I/O 失败时返回错误。
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// use easyofd::{EasyOfd, OfdPage, TextObject};
    ///
    /// let page = OfdPage::new(210.0, 297.0);
    /// EasyOfd::write_pages_to("output.ofd", vec![page])?;
    /// ```
    pub fn write_pages_to(path: impl AsRef<std::path::Path>, pages: Vec<OfdPage>) -> OfdResult<()> {
        let mut writer = OfdWriter::new();
        writer.add_pages(pages);
        writer.build_to_file(path)
    }

    /// 将页面直接写入字节数组（无文件 I/O）。
    ///
    /// # 错误
    ///
    /// ZIP 创建失败时返回错误。
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// use easyofd::{EasyOfd, OfdPage, TextObject};
    ///
    /// let page = OfdPage::new(210.0, 297.0);
    /// let bytes = EasyOfd::write_pages_to_bytes(vec![page])?;
    /// assert_eq!(&bytes[0..2], b"PK");
    /// ```
    pub fn write_pages_to_bytes(pages: Vec<OfdPage>) -> OfdResult<Vec<u8>> {
        let mut writer = OfdWriter::new();
        writer.add_pages(pages);
        writer.build()
    }

    /// 打开并解析 OFD 文件进行读取。
    ///
    /// 返回 [`OfdReader`]，提供页面和文本内容的访问。
    ///
    /// # 错误
    ///
    /// 文件无法读取或不是有效 OFD 文档时返回错误。
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// use easyofd::EasyOfd;
    ///
    /// let reader = EasyOfd::read("input.ofd")?;
    /// println!("页数: {}", reader.page_count());
    /// println!("全文:\n{}", reader.extract_all_text());
    /// ```
    pub fn read(path: impl AsRef<std::path::Path>) -> OfdResult<OfdReader> {
        OfdReader::open(path)
    }

    /// 创建逐页读取构建器，适合大文件和有限内存场景。
    ///
    /// 使用 visitor 模式，页面不会全部驻留在内存中。
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// use easyofd::EasyOfd;
    ///
    /// EasyOfd::read_pages("input.ofd")
    ///     .page_range(1, 10)
    ///     .do_read(|page_number, page| {
    ///         println!("Page {page_number}: {} objects", page.content.len());
    ///         Ok(())
    ///     })?;
    /// ```
    #[must_use]
    pub fn read_pages(path: impl AsRef<std::path::Path>) -> OfdReadBuilder {
        OfdReadBuilder::new(path)
    }

    /// 创建 OFD 到 Markdown 的转换构建器。
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// use easyofd::EasyOfd;
    ///
    /// let result = EasyOfd::to_markdown("input.ofd").do_convert()?;
    /// println!("{}", result.markdown);
    /// ```
    #[must_use]
    pub fn to_markdown(path: impl AsRef<std::path::Path>) -> MarkdownConversionBuilder {
        MarkdownConversionBuilder::new(path)
    }

    /// 从内存字节数组解析 OFD 文档。
    ///
    /// 返回 [`OfdReader`] 用于提取内容。
    ///
    /// # 错误
    ///
    /// 数据不是有效 OFD 文档时返回错误。
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// use easyofd::EasyOfd;
    ///
    /// let bytes = std::fs::read("input.ofd")?;
    /// let reader = EasyOfd::read_from_bytes(&bytes)?;
    /// println!("页数: {}", reader.page_count());
    /// ```
    pub fn read_from_bytes(data: &[u8]) -> OfdResult<OfdReader> {
        OfdReader::from_bytes(data)
    }

    /// 使用占位符值填充 OFD 模板。
    ///
    /// 将 XML 内容中的 `{key}` 模式替换为数据映射中的值。
    ///
    /// # 错误
    ///
    /// 模板文件无法读取时返回错误。
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// use easyofd::EasyOfd;
    /// use std::collections::HashMap;
    ///
    /// let mut data = HashMap::new();
    /// data.insert("name".to_string(), "张三".to_string());
    /// data.insert("amount".to_string(), "¥1,234.00".to_string());
    ///
    /// EasyOfd::fill_template("template.ofd", &data)?.save("filled.ofd")?;
    /// ```
    pub fn fill_template(
        template_path: impl AsRef<std::path::Path>,
        data: &std::collections::HashMap<String, String>,
    ) -> OfdResult<OfdTemplateFiller> {
        OfdTemplateFiller::fill(template_path, data)
    }
}
