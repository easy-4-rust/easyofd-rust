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
    /// 原样保留的 ZIP 条目（roundtrip 时从 `OfdReader::raw_entries()` 复制）。
    pub(crate) preserved_entries: Vec<(String, Vec<u8>)>,
}

impl OfdWriter {
    /// 使用默认选项创建新的 OFD 写入器。
    #[must_use]
    pub fn new() -> Self {
        Self {
            pages: Vec::new(),
            options: WriteOptions::default(),
            preserved_entries: Vec::new(),
        }
    }

    /// 使用自定义选项创建 OFD 写入器。
    #[must_use]
    pub fn with_options(options: WriteOptions) -> Self {
        Self {
            pages: Vec::new(),
            options,
            preserved_entries: Vec::new(),
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

    /// 原样保留 ZIP 条目（如模板页、注释、附件、签名、自定义标签内容）。
    ///
    /// 供无损 roundtrip 使用：把 `OfdReader::raw_entries()` 的结果传入，
    /// 写入器会在写出时将这些条目按原字节复制，跳过已由写入器生成的条目。
    pub fn preserve_entries(&mut self, entries: Vec<(String, Vec<u8>)>) {
        self.preserved_entries.extend(entries);
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

    /// 计算页面的归档路径：优先使用 roundtrip 读取时保留的原始路径
    /// （`OfdPage::base_path`），否则按 `Pages/Page_{index}/Content.xml` 命名。
    fn page_archive_path(&self, page: &OfdPage, index: usize) -> String {
        let doc_dir = &self.options.metadata.doc_dir;
        let doc_dir_prefix = format!("{doc_dir}/");
        match page.base_path.as_deref() {
            Some(path) if path.starts_with(&doc_dir_prefix) || path.starts_with('/') => {
                path.trim_start_matches('/').to_string()
            }
            Some(path) => format!("{doc_dir}/{path}"),
            None => format!("{doc_dir}/Pages/Page_{index}/Content.xml"),
        }
    }

    pub(crate) fn write_zip<W: Write + std::io::Seek>(
        &self,
        zip: &mut ZipWriter<W>,
        options: &SimpleFileOptions,
    ) -> OfdResult<()> {
        // 收集所有页面中的图片资源。
        let doc_dir = &self.options.metadata.doc_dir;
        let mut image_resources: Vec<(String, &[u8], ImageFormat)> = Vec::new();

        for page in &self.pages {
            for obj in &page.content {
                if let ContentObject::Image(img) = obj {
                    // Prefer the original resource path from a roundtrip read;
                    // fall back to writer-assigned names otherwise.
                    let res_name = img.res_name.clone().unwrap_or_else(|| {
                        let ext = match img.format {
                            ImageFormat::Jpeg => "jpeg",
                            ImageFormat::Png => "png",
                            ImageFormat::Bmp => "bmp",
                            ImageFormat::Tiff => "tiff",
                        };
                        format!("{doc_dir}/Res/Image_{}.{}", image_resources.len(), ext)
                    });
                    // Normalize to an archive path: the original resource path
                    // from a read is relative to the document directory
                    // (e.g. "Res/qrcode.png") or already archive-absolute with
                    // the doc directory (case may differ, e.g. "DOC_0/Res/...").
                    let doc_dir_prefix = format!("{doc_dir}/");
                    let res_name = if res_name.starts_with('/')
                        || res_name
                            .get(..doc_dir_prefix.len())
                            .is_some_and(|head| head.eq_ignore_ascii_case(&doc_dir_prefix))
                    {
                        res_name
                    } else {
                        format!("{doc_dir}/{res_name}")
                    };
                    image_resources.push((res_name, img.data.as_slice(), img.format));
                }
            }
        }

        // 1. 写入 OFD.xml
        let ofd_xml = self.build_ofd_xml();
        zip.start_file("OFD.xml", *options).map_err(zip_err)?;
        zip.write_all(ofd_xml.as_bytes()).map_err(io_err)?;

        // 2. 写入 Document（文件名为原始 DocRoot 的文件名）
        let doc_path = format!("{doc_dir}/{}", self.options.metadata.document_file);
        let doc_xml = self.build_document_xml(&image_resources);
        zip.start_file(&doc_path, *options).map_err(zip_err)?;
        zip.write_all(doc_xml.as_bytes()).map_err(io_err)?;

        // 3. 写入 DocumentRes.xml（与 Document.xml 中的引用保持一致：仅在
        // 存在图片资源时输出）
        if !image_resources.is_empty() {
            let doc_res_xml = self.build_document_res_xml(&image_resources);
            zip.start_file(format!("{doc_dir}/DocumentRes.xml"), *options)
                .map_err(zip_err)?;
            zip.write_all(doc_res_xml.as_bytes()).map_err(io_err)?;
        }

        // PublicRes.xml: ofdrw only writes the file when font resources were
        // explicitly added; a roundtrip source without it is preserved as-is.
        if self.options.metadata.public_res_present {
            zip.start_file(format!("{doc_dir}/PublicRes.xml"), *options)
                .map_err(zip_err)?;
            zip.write_all(self.build_public_res_xml(&image_resources).as_bytes())
                .map_err(io_err)?;
        }

        // 4. 写入各页面
        let mut page_image_start = 0;
        for (i, page) in self.pages.iter().enumerate() {
            let page_xml = self.build_page_xml(page, i, page_image_start);
            let page_path = self.page_archive_path(page, i);
            zip.start_file(&page_path, *options).map_err(zip_err)?;
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

        // 6. 原样保留的条目（roundtrip 容器内容），跳过已生成的同名条目
        let mut written: std::collections::HashSet<String> = std::collections::HashSet::new();
        written.insert("OFD.xml".to_string());
        written.insert(doc_path);
        // DocumentRes.xml is only regenerated when there are image resources;
        // a leftover DocumentRes.xml in the source archive is preserved then.
        if !image_resources.is_empty() {
            written.insert(format!("{doc_dir}/DocumentRes.xml"));
        }
        written.insert(format!("{doc_dir}/PublicRes.xml"));
        written.extend(image_resources.iter().map(|(name, _, _)| name.clone()));
        for (i, page) in self.pages.iter().enumerate() {
            written.insert(self.page_archive_path(page, i));
        }
        for (name, data) in &self.preserved_entries {
            if !written.contains(name) {
                zip.start_file(name, *options).map_err(zip_err)?;
                zip.write_all(data).map_err(io_err)?;
                written.insert(name.clone());
            }
        }

        Ok(())
    }
}

impl Default for OfdWriter {
    fn default() -> Self {
        Self::new()
    }
}
