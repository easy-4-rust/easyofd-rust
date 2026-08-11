use std::io::{Seek, Write};

use easyofd_core::{ContentObject, ImageFormat, OfdPage, OfdResult};
use zip::write::{SimpleFileOptions, ZipWriter};

use crate::helpers::{io_err, zip_err};
use crate::{OfdWriter, WriteOptions};

/// 逐页写入 OFD 的常量内容内存 Writer。
///
/// 页面 XML 和图片字节在 `write_page` 时立即写入 ZIP；实例只保留页面尺寸、页面引用和
/// 资源目录。调用方必须显式调用 `finish` 以完成中央目录和文档索引。
pub struct OfdStreamWriter<W: Write + Seek> {
    zip: Option<ZipWriter<W>>,
    options: WriteOptions,
    page_descriptors: Vec<OfdPage>,
    image_resources: Vec<(String, ImageFormat)>,
    file_options: SimpleFileOptions,
}

impl<W: Write + Seek> OfdStreamWriter<W> {
    /// 从可写、可定位的输出创建流式 Writer。
    #[must_use]
    pub fn new(output: W, options: WriteOptions) -> Self {
        Self {
            zip: Some(ZipWriter::new(output)),
            options,
            page_descriptors: Vec::new(),
            image_resources: Vec::new(),
            file_options: SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated),
        }
    }

    /// 写入一个页面及其图片资源。
    ///
    /// # Errors
    ///
    /// 页面 XML 或资源写入 ZIP 失败时返回错误。
    pub fn write_page(&mut self, page: OfdPage) -> OfdResult<()> {
        let page_index = self.page_descriptors.len();
        let page_image_start = self.image_resources.len();
        let doc_dir = self.options.metadata.doc_dir.clone();
        let mut page_images = Vec::new();
        for object in &page.content {
            if let ContentObject::Image(image) = object {
                let resource_index = self.image_resources.len();
                let name = format!(
                    "{doc_dir}/Res/Image_{resource_index}.{}",
                    image_extension(image.format)
                );
                self.image_resources.push((name.clone(), image.format));
                page_images.push((name, image.data.as_slice()));
            }
        }

        let helper = OfdWriter::with_options(self.options.clone());
        let page_xml = helper.build_page_xml(&page, page_index, page_image_start);
        let file_options = self.file_options;
        let zip = self.zip_mut()?;
        zip.start_file(
            format!("{doc_dir}/Pages/Page_{page_index}/Content.xml"),
            file_options,
        )
        .map_err(zip_err)?;
        zip.write_all(page_xml.as_bytes()).map_err(io_err)?;
        for (name, bytes) in page_images {
            zip.start_file(name, file_options).map_err(zip_err)?;
            zip.write_all(bytes).map_err(io_err)?;
        }
        self.page_descriptors
            .push(OfdPage::new(page.width, page.height));
        Ok(())
    }

    /// 依次写入一批页面，不要求调用方预先收集为 `Vec`。
    ///
    /// # Errors
    ///
    /// 任一页面写入失败时立即返回错误。
    pub fn write_pages(&mut self, pages: impl IntoIterator<Item = OfdPage>) -> OfdResult<()> {
        for page in pages {
            self.write_page(page)?;
        }
        Ok(())
    }

    /// 完成 OFD 索引和 ZIP 中央目录，并返回底层输出。
    ///
    /// # Errors
    ///
    /// 文档索引或 ZIP 完成失败时返回错误。
    pub fn finish(mut self) -> OfdResult<W> {
        let doc_dir = self.options.metadata.doc_dir.clone();
        let document_file = self.options.metadata.document_file.clone();
        let public_res_present = self.options.metadata.public_res_present;
        let mut helper = OfdWriter::with_options(self.options.clone());
        let resources = self.resource_view();
        helper.pages = std::mem::take(&mut self.page_descriptors);
        let ofd_xml = helper.build_ofd_xml();
        let document_xml = helper.build_document_xml(&resources);
        let document_res_xml = helper.build_document_res_xml(&resources);
        let public_res_xml = helper.build_public_res_xml(&resources);
        let file_options = self.file_options;
        let zip = self.zip_mut()?;
        write_xml(zip, file_options, "OFD.xml", &ofd_xml)?;
        write_xml(
            zip,
            file_options,
            &format!("{doc_dir}/{document_file}"),
            &document_xml,
        )?;
        if !resources.is_empty() {
            write_xml(
                zip,
                file_options,
                &format!("{doc_dir}/DocumentRes.xml"),
                &document_res_xml,
            )?;
        }
        if public_res_present {
            write_xml(
                zip,
                file_options,
                &format!("{doc_dir}/PublicRes.xml"),
                &public_res_xml,
            )?;
        }
        self.zip
            .take()
            .expect("stream writer ZIP exists until finish")
            .finish()
            .map_err(zip_err)
    }

    fn resource_view(&self) -> Vec<(String, &'static [u8], ImageFormat)> {
        self.image_resources
            .iter()
            .map(|(name, format)| (name.clone(), &[][..], *format))
            .collect()
    }

    fn zip_mut(&mut self) -> OfdResult<&mut ZipWriter<W>> {
        self.zip.as_mut().ok_or_else(|| {
            easyofd_core::OfdError::InvalidDocument(
                "stream writer has already finished".to_string(),
            )
        })
    }
}

fn write_xml<W: Write + Seek>(
    zip: &mut ZipWriter<W>,
    options: SimpleFileOptions,
    name: &str,
    xml: &str,
) -> OfdResult<()> {
    zip.start_file(name, options).map_err(zip_err)?;
    zip.write_all(xml.as_bytes()).map_err(io_err)
}

fn image_extension(format: ImageFormat) -> &'static str {
    match format {
        ImageFormat::Jpeg => "jpeg",
        ImageFormat::Png => "png",
        ImageFormat::Bmp => "bmp",
        ImageFormat::Tiff => "tiff",
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use easyofd_core::{ContentObject, ImageObject, TextObject};

    use super::*;

    #[test]
    fn streams_pages_and_resources_without_retaining_content() {
        let mut writer = OfdStreamWriter::new(Cursor::new(Vec::new()), WriteOptions::default());
        for number in 1..=3 {
            let mut page = OfdPage::new(210.0, 297.0);
            page.add_text(TextObject::new(10.0, 10.0, format!("page {number}")));
            page.add_image(ImageObject::png(
                10.0,
                30.0,
                10.0,
                10.0,
                vec![u8::try_from(number).unwrap()],
            ));
            writer.write_page(page).unwrap();
        }
        let bytes = writer.finish().unwrap().into_inner();
        let reader = easyofd_reader::OfdReader::from_bytes(&bytes).unwrap();
        assert_eq!(reader.page_count(), 3);
        let ContentObject::Image(image) = &reader.pages()[2].content[1] else {
            panic!("expected image");
        };
        assert_eq!(image.data, vec![3]);
    }
}
