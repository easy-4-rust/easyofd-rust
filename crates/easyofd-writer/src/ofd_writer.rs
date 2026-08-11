//! OFD 写入器，将页面构建为 GB/T 33190-2016 合规的 ZIP 归档。

use std::io::{Cursor, Write};

use crate::helpers::{io_err, zip_err};
use crate::write_options::WriteOptions;
use easyofd_core::{ContentObject, ImageFormat, OfdPage, OfdResult};
use easyofd_package::atomic_write;
use zip::write::{SimpleFileOptions, ZipWriter};

/// OFD 写入器，收集页面并写入 ZIP 归档。
pub struct OfdWriter {
    pub(crate) pages: Vec<OfdPage>,
    pub(crate) options: WriteOptions,
}

impl OfdWriter {
    /// 使用默认选项创建新的 OFD 写入器。
    #[must_use]
    pub fn new() -> Self {
        Self {
            pages: Vec::new(),
            options: WriteOptions::default(),
        }
    }

    /// 使用自定义选项创建 OFD 写入器。
    #[must_use]
    pub fn with_options(options: WriteOptions) -> Self {
        Self {
            pages: Vec::new(),
            options,
        }
    }

    /// 设置文档元数据（从 OfdReader 提取时使用）。
    pub fn set_metadata(&mut self, metadata: easyofd_core::OfdMetadata) {
        self.options.metadata = metadata;
    }

    /// 向文档添加一个页面。
    pub fn add_page(&mut self, page: OfdPage) {
        self.pages.push(page);
    }

    /// 向文档添加多个页面。
    pub fn add_pages(&mut self, pages: Vec<OfdPage>) {
        self.pages.extend(pages);
    }

    /// 构建 OFD 文件并返回原始字节。
    ///
    /// # 错误
    ///
    /// ZIP 创建失败时返回错误。
    pub fn build(&self) -> OfdResult<Vec<u8>> {
        let cursor = Cursor::new(Vec::with_capacity(4096));
        let cursor = self.write_to(cursor)?;
        Ok(cursor.into_inner())
    }

    /// 将 OFD 直接写入支持定位的输出，不额外构造完整字节数组。
    ///
    /// # 错误
    ///
    /// ZIP 创建或输出写入失败时返回错误。
    pub fn write_to<W: Write + std::io::Seek>(&self, output: W) -> OfdResult<W> {
        let mut zip = ZipWriter::new(output);
        let options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
        self.write_zip(&mut zip, &options)?;
        zip.finish().map_err(zip_err)
    }

    /// 构建 OFD 文件并写入文件路径。
    ///
    /// # 错误
    ///
    /// ZIP 创建或文件 I/O 失败时返回错误。
    pub fn build_to_file(&self, path: impl AsRef<std::path::Path>) -> OfdResult<()> {
        atomic_write(path, |file| {
            self.write_to(file)?;
            Ok(())
        })
    }

    pub(crate) fn write_zip<W: Write + std::io::Seek>(
        &self,
        zip: &mut ZipWriter<W>,
        options: &SimpleFileOptions,
    ) -> OfdResult<()> {
        // 收集所有页面中的图片资源。
        let mut image_resources: Vec<(String, &[u8], ImageFormat)> = Vec::new();

        for page in &self.pages {
            for obj in &page.content {
                if let ContentObject::Image(img) = obj {
                    let ext = match img.format {
                        ImageFormat::Jpeg => "jpeg",
                        ImageFormat::Png => "png",
                        ImageFormat::Bmp => "bmp",
                        ImageFormat::Tiff => "tiff",
                    };
                    let res_name = format!("Doc_0/Res/Image_{}.{}", image_resources.len(), ext);
                    image_resources.push((res_name, img.data.as_slice(), img.format));
                }
            }
        }

        // 1. 写入 OFD.xml
        let ofd_xml = self.build_ofd_xml();
        zip.start_file("OFD.xml", *options).map_err(zip_err)?;
        zip.write_all(ofd_xml.as_bytes()).map_err(io_err)?;

        // 2. 写入 Document.xml
        let doc_xml = self.build_document_xml(&image_resources);
        zip.start_file("Doc_0/Document.xml", *options)
            .map_err(zip_err)?;
        zip.write_all(doc_xml.as_bytes()).map_err(io_err)?;

        // 3. 写入 DocumentRes.xml
        let doc_res_xml = self.build_document_res_xml(&image_resources);
        zip.start_file("Doc_0/DocumentRes.xml", *options)
            .map_err(zip_err)?;
        zip.write_all(doc_res_xml.as_bytes()).map_err(io_err)?;

        // PublicRes 在 Document.xml 中引用，即使没有自定义字体也始终写入。
        zip.start_file("Doc_0/PublicRes.xml", *options)
            .map_err(zip_err)?;
        zip.write_all(self.build_public_res_xml().as_bytes())
            .map_err(io_err)?;

        // 4. 写入各页面
        let mut page_image_start = 0;
        for (i, page) in self.pages.iter().enumerate() {
            let page_xml = self.build_page_xml(page, i, page_image_start);
            zip.start_file(format!("Doc_0/Pages/Page_{i}/Content.xml"), *options)
                .map_err(zip_err)?;
            zip.write_all(page_xml.as_bytes()).map_err(io_err)?;
            page_image_start += page
                .content
                .iter()
                .filter(|object| matches!(object, ContentObject::Image(_)))
                .count();
        }

        // 5. 写入图片资源
        for (res_name, data, _) in &image_resources {
            zip.start_file(res_name, *options).map_err(zip_err)?;
            zip.write_all(data).map_err(io_err)?;
        }

        Ok(())
    }
}

impl Default for OfdWriter {
    fn default() -> Self {
        Self::new()
    }
}
