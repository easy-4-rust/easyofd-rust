#![allow(clippy::format_push_string)]

//! # easyofd-writer
//!
//! OFD file writer that produces GB/T 33190-2016 compliant ZIP archives.
//!
//! ## Architecture
//!
//! The writer builds an in-memory ZIP archive with this structure:
//!
//! ```text
//! output.ofd (ZIP)
//! ├── OFD.xml                    ← entry point
//! └── Doc_0/
//!     ├── Document.xml           ← document structure
//!     ├── DocumentRes.xml        ← document resources (images, fonts)
//!     ├── Pages/
//!     │   ├── Page_0.xml         ← page content
//!     │   ├── Page_1.xml
//!     │   └── ...
//!     └── Res/                   ← embedded resources
//!         ├── Image_0.jpeg
//!         └── ...
//! ```

mod stream_writer;
mod editor;
mod helpers;
mod font;
mod xml_builder;

pub use stream_writer::OfdStreamWriter;
pub use editor::OfdEditor;
pub use font::{EmbeddedFont, FontFormat};

use std::io::{Cursor, Write};

use chrono::Utc;
use easyofd_core::{ContentObject, ImageFormat, OfdMetadata, OfdPage, OfdResult};
use easyofd_package::atomic_write;
use zip::write::{SimpleFileOptions, ZipWriter};
use crate::helpers::{zip_err, io_err};

// ─── Public API ──────────────────────────────────────────────────────────────

/// Write options for OFD generation.
#[derive(Debug, Clone)]
pub struct WriteOptions {
    /// Document metadata.
    pub metadata: OfdMetadata,
}

impl Default for WriteOptions {
    fn default() -> Self {
        Self {
            metadata: OfdMetadata {
                version: "1.0".to_string(),
                title: Some("EasyOFD Document".to_string()),
                author: Some("easyofd-rust".to_string()),
                creator: Some("easyofd-rust".to_string()),
                creation_date: Some(Utc::now().naive_utc()),
            },
        }
    }
}

/// The OFD writer. Collects pages and writes them to a ZIP archive.
pub struct OfdWriter {
    pages: Vec<OfdPage>,
    options: WriteOptions,
}

impl OfdWriter {
    /// Create a new OFD writer with default options.
    #[must_use]
    pub fn new() -> Self {
        Self {
            pages: Vec::new(),
            options: WriteOptions::default(),
        }
    }

    /// Create a new OFD writer with custom options.
    #[must_use]
    pub fn with_options(options: WriteOptions) -> Self {
        Self {
            pages: Vec::new(),
            options,
        }
    }

    /// Add a page to the document.
    pub fn add_page(&mut self, page: OfdPage) {
        self.pages.push(page);
    }

    /// Add multiple pages to the document.
    pub fn add_pages(&mut self, pages: Vec<OfdPage>) {
        self.pages.extend(pages);
    }

    /// Build the OFD file and return the raw bytes.
    ///
    /// # Errors
    ///
    /// Returns an error if ZIP creation fails.
    pub fn build(&self) -> OfdResult<Vec<u8>> {
        let cursor = Cursor::new(Vec::with_capacity(4096));
        let cursor = self.write_to(cursor)?;
        Ok(cursor.into_inner())
    }

    /// 将 OFD 直接写入支持定位的输出，不额外构造完整字节数组。
    ///
    /// # Errors
    ///
    /// ZIP 创建或输出写入失败时返回错误。
    pub fn write_to<W: Write + std::io::Seek>(&self, output: W) -> OfdResult<W> {
        let mut zip = ZipWriter::new(output);
        let options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
        self.write_zip(&mut zip, &options)?;
        zip.finish().map_err(zip_err)
    }

    /// Build the OFD file and write it to a file path.
    ///
    /// # Errors
    ///
    /// Returns an error if ZIP creation or file I/O fails.
    pub fn build_to_file(&self, path: impl AsRef<std::path::Path>) -> OfdResult<()> {
        atomic_write(path, |file| {
            self.write_to(file)?;
            Ok(())
        })
    }

    fn write_zip<W: Write + std::io::Seek>(
        &self,
        zip: &mut ZipWriter<W>,
        options: &SimpleFileOptions,
    ) -> OfdResult<()> {
        // Collect all image resources across all pages.
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

        // 1. Write OFD.xml
        let ofd_xml = self.build_ofd_xml();
        zip.start_file("OFD.xml", *options).map_err(zip_err)?;
        zip.write_all(ofd_xml.as_bytes()).map_err(io_err)?;

        // 2. Write Document.xml
        let doc_xml = self.build_document_xml(&image_resources);
        zip.start_file("Doc_0/Document.xml", *options)
            .map_err(zip_err)?;
        zip.write_all(doc_xml.as_bytes()).map_err(io_err)?;

        // 3. Write DocumentRes.xml
        let doc_res_xml = self.build_document_res_xml(&image_resources);
        zip.start_file("Doc_0/DocumentRes.xml", *options)
            .map_err(zip_err)?;
        zip.write_all(doc_res_xml.as_bytes()).map_err(io_err)?;

        // PublicRes is referenced by Document.xml even when no custom font is embedded.
        zip.start_file("Doc_0/PublicRes.xml", *options)
            .map_err(zip_err)?;
        zip.write_all(self.build_public_res_xml().as_bytes())
            .map_err(io_err)?;

        // 4. Write each page
        let mut page_image_start = 0;
        for (i, page) in self.pages.iter().enumerate() {
            let page_xml = self.build_page_xml(page, i, page_image_start);
            zip.start_file(format!("Doc_0/Pages/Page_{i}.xml"), *options)
                .map_err(zip_err)?;
            zip.write_all(page_xml.as_bytes()).map_err(io_err)?;
            page_image_start += page
                .content
                .iter()
                .filter(|object| matches!(object, ContentObject::Image(_)))
                .count();
        }

        // 5. Write image resources
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

#[cfg(test)]
mod font_tests {
    use super::*;

    #[test]
    fn test_embedded_font_clone_debug() {
        let font = EmbeddedFont {
            name: "SimHei".into(),
            data: vec![0x00, 0x01, 0x02],
            format: FontFormat::TrueType,
        };
        let f2 = font.clone();
        assert_eq!(f2.name, "SimHei");
        assert!(format!("{font:?}").contains("EmbeddedFont"));
    }

    #[test]
    fn test_font_format_enum() {
        assert_ne!(FontFormat::TrueType, FontFormat::OpenType);
        assert_eq!(FontFormat::TrueType, FontFormat::TrueType);
    }

    #[test]
    fn test_embed_font_accepts() {
        let mut writer = OfdWriter::new();
        writer.embed_font(EmbeddedFont {
            name: "TestFont".into(),
            data: vec![0; 100],
            format: FontFormat::OpenType,
        });
        // verify writer still works
        let bytes = writer.build().unwrap();
        assert_eq!(&bytes[0..2], b"PK");
    }
}

#[cfg(test)]
mod tests {
    use crate::helpers::xml_escape;
    use super::*;
    use easyofd_core::{ImageObject, PathObject, TextObject};

    // ── WriteOptions ──────────────────────────────────────────────────────────

    #[test]
    fn test_write_options_default() {
        let opts = WriteOptions::default();
        assert_eq!(opts.metadata.version, "1.0");
        assert_eq!(opts.metadata.title.as_deref(), Some("EasyOFD Document"));
        assert_eq!(opts.metadata.author.as_deref(), Some("easyofd-rust"));
        assert_eq!(opts.metadata.creator.as_deref(), Some("easyofd-rust"));
        assert!(opts.metadata.creation_date.is_some());
    }

    #[test]
    fn test_write_options_clone_debug() {
        let opts = WriteOptions::default();
        let opts2 = opts.clone();
        assert_eq!(opts2.metadata.version, "1.0");
        assert!(format!("{opts:?}").contains("WriteOptions"));
    }

    // ── OfdWriter constructors ────────────────────────────────────────────────

    #[test]
    fn test_ofd_writer_new() {
        let w = OfdWriter::new();
        assert!(w.pages.is_empty());
        assert_eq!(w.options.metadata.version, "1.0");
    }

    #[test]
    fn test_ofd_writer_default() {
        let w = OfdWriter::default();
        assert!(w.pages.is_empty());
    }

    #[test]
    fn test_ofd_writer_with_options() {
        let mut opts = WriteOptions::default();
        opts.metadata.title = Some("Custom".into());
        let w = OfdWriter::with_options(opts);
        assert_eq!(w.options.metadata.title.as_deref(), Some("Custom"));
    }

    // ── add_page / add_pages ──────────────────────────────────────────────────

    #[test]
    fn test_add_page() {
        let mut w = OfdWriter::new();
        w.add_page(OfdPage::new(210.0, 297.0));
        assert_eq!(w.pages.len(), 1);
    }

    #[test]
    fn test_add_pages() {
        let mut w = OfdWriter::new();
        w.add_pages(vec![OfdPage::new(210.0, 297.0), OfdPage::new(297.0, 210.0)]);
        assert_eq!(w.pages.len(), 2);
    }

    // ── build ─────────────────────────────────────────────────────────────────

    #[test]
    fn test_build_empty() {
        let bytes = OfdWriter::new().build().unwrap();
        assert!(!bytes.is_empty());
        assert_eq!(&bytes[0..2], b"PK");
    }

    #[test]
    fn test_build_single_text_page() {
        let mut w = OfdWriter::new();
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_text(TextObject::new(20.0, 30.0, "Hello, OFD!"));
        w.add_page(page);
        let bytes = w.build().unwrap();
        let names = zip_entry_names(&bytes);
        assert!(names.contains(&"OFD.xml".to_string()));
        assert!(names.contains(&"Doc_0/Document.xml".to_string()));
        assert!(names.contains(&"Doc_0/Pages/Page_0.xml".to_string()));
    }

    #[test]
    fn test_build_multi_page() {
        let mut w = OfdWriter::new();
        for i in 0..3 {
            let mut page = OfdPage::new(210.0, 297.0);
            page.add_text(TextObject::new(20.0, 30.0, format!("Page {i}")));
            w.add_page(page);
        }
        let bytes = w.build().unwrap();
        let names = zip_entry_names(&bytes);
        assert!(names.contains(&"Doc_0/Pages/Page_0.xml".to_string()));
        assert!(names.contains(&"Doc_0/Pages/Page_1.xml".to_string()));
        assert!(names.contains(&"Doc_0/Pages/Page_2.xml".to_string()));
    }

    // ── build_to_file ─────────────────────────────────────────────────────────

    #[test]
    fn test_build_to_file() {
        let dir = std::env::temp_dir().join("easyofd_test");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test_build_to_file.ofd");

        let mut w = OfdWriter::new();
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_text(TextObject::new(0.0, 0.0, "file test"));
        w.add_page(page);
        w.build_to_file(&path).unwrap();

        let bytes = std::fs::read(&path).unwrap();
        assert_eq!(&bytes[0..2], b"PK");
        let _ = std::fs::remove_file(&path);
    }

    // ── Metadata variations ───────────────────────────────────────────────────

    #[test]
    fn test_ofd_xml_all_metadata() {
        let mut opts = WriteOptions::default();
        opts.metadata.title = Some("T".into());
        opts.metadata.author = Some("A".into());
        opts.metadata.creator = Some("C".into());
        let w = OfdWriter::with_options(opts);
        let xml = w.build_ofd_xml();
        assert!(xml.contains("<ofd:Title>T</ofd:Title>"));
        assert!(xml.contains("<ofd:Author>A</ofd:Author>"));
        assert!(xml.contains("<ofd:Creator>C</ofd:Creator>"));
        assert!(xml.contains("<ofd:CreationDate>"));
    }

    #[test]
    fn test_ofd_xml_no_optional_metadata() {
        let mut opts = WriteOptions::default();
        opts.metadata.title = None;
        opts.metadata.author = None;
        opts.metadata.creator = None;
        opts.metadata.creation_date = None;
        let w = OfdWriter::with_options(opts);
        let xml = w.build_ofd_xml();
        assert!(!xml.contains("<ofd:T"));
        assert!(!xml.contains("<ofd:Author>"));
        assert!(!xml.contains("<ofd:Creator>"));
        assert!(!xml.contains("<ofd:CreationDate>"));
    }

    // ── XML special chars in metadata ─────────────────────────────────────────

    #[test]
    fn test_ofd_xml_special_chars_in_title() {
        let mut opts = WriteOptions::default();
        opts.metadata.title = Some("A<B&C\"D'E".into());
        let w = OfdWriter::with_options(opts);
        let xml = w.build_ofd_xml();
        assert!(xml.contains("&lt;"));
        assert!(xml.contains("&amp;"));
        assert!(xml.contains("&quot;"));
        assert!(xml.contains("&apos;"));
    }

    // ── Document.xml variations ───────────────────────────────────────────────

    #[test]
    fn test_document_xml_with_images() {
        let mut w = OfdWriter::new();
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_image(ImageObject::jpeg(0.0, 0.0, 10.0, 10.0, vec![0xFF]));
        w.add_page(page);
        let bytes = w.build().unwrap();
        assert!(bytes.windows(6).any(|w| w == b"Image_"));
    }

    #[test]
    fn test_document_xml_without_images() {
        let mut w = OfdWriter::new();
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_text(TextObject::new(0.0, 0.0, "no images"));
        w.add_page(page);
        let bytes = w.build().unwrap();
        let names = zip_entry_names(&bytes);
        // No image resource files in the ZIP
        assert!(!names.iter().any(|n| n.contains("Image_")));
        // DocumentRes.xml is always present but has no MultiMedia entries
        assert!(names.contains(&"Doc_0/DocumentRes.xml".to_string()));
    }

    // ── DocumentRes.xml with all image formats ────────────────────────────────

    #[test]
    fn test_document_res_png() {
        let mut w = OfdWriter::new();
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_image(ImageObject::png(0.0, 0.0, 10.0, 10.0, vec![0x89]));
        w.add_page(page);
        let bytes = w.build().unwrap();
        let names = zip_entry_names(&bytes);
        assert!(names.contains(&"Doc_0/Res/Image_0.png".to_string()));
    }

    #[test]
    fn test_document_res_bmp() {
        let mut w = OfdWriter::new();
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_image(ImageObject::new(
            0.0,
            0.0,
            10.0,
            10.0,
            vec![0x42],
            ImageFormat::Bmp,
        ));
        w.add_page(page);
        let bytes = w.build().unwrap();
        let names = zip_entry_names(&bytes);
        assert!(names.contains(&"Doc_0/Res/Image_0.bmp".to_string()));
    }

    #[test]
    fn test_document_res_tiff() {
        let mut w = OfdWriter::new();
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_image(ImageObject::new(
            0.0,
            0.0,
            10.0,
            10.0,
            vec![0x49],
            ImageFormat::Tiff,
        ));
        w.add_page(page);
        let bytes = w.build().unwrap();
        let names = zip_entry_names(&bytes);
        assert!(names.contains(&"Doc_0/Res/Image_0.tiff".to_string()));
    }

    // ── Page content: Text ────────────────────────────────────────────────────

    #[test]
    fn test_page_text_with_custom_size() {
        let mut w = OfdWriter::new();
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_text(
            TextObject::new(10.0, 20.0, "styled")
                .font("SimHei")
                .size(24.0)
                .bold()
                .italic()
                .color(0xFF_0000),
        );
        w.add_page(page);
        let bytes = w.build().unwrap();
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_page_text_with_explicit_width_height() {
        let mut w = OfdWriter::new();
        let mut page = OfdPage::new(210.0, 297.0);
        let mut t = TextObject::new(10.0, 20.0, "sized");
        t.width = Some(100.0);
        t.height = Some(20.0);
        page.add_text(t);
        w.add_page(page);
        let bytes = w.build().unwrap();
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_page_text_special_chars() {
        let mut w = OfdWriter::new();
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_text(TextObject::new(0.0, 0.0, "a<b&c\"d'e"));
        w.add_page(page);
        let bytes = w.build().unwrap();
        assert!(!bytes.is_empty());
    }

    // ── Page content: Image ───────────────────────────────────────────────────

    #[test]
    fn test_page_image_jpeg() {
        let mut w = OfdWriter::new();
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_image(ImageObject::jpeg(50.0, 50.0, 30.0, 30.0, vec![0xFF, 0xD8]));
        w.add_page(page);
        let bytes = w.build().unwrap();
        let names = zip_entry_names(&bytes);
        assert!(names.contains(&"Doc_0/Res/Image_0.jpeg".to_string()));
    }

    #[test]
    fn test_page_multiple_images() {
        let mut w = OfdWriter::new();
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_image(ImageObject::jpeg(0.0, 0.0, 10.0, 10.0, vec![0xFF]));
        page.add_image(ImageObject::png(20.0, 0.0, 10.0, 10.0, vec![0x89]));
        w.add_page(page);
        let bytes = w.build().unwrap();
        let names = zip_entry_names(&bytes);
        assert!(names.contains(&"Doc_0/Res/Image_0.jpeg".to_string()));
        assert!(names.contains(&"Doc_0/Res/Image_1.png".to_string()));
    }

    // ── Page content: Path ────────────────────────────────────────────────────

    #[test]
    fn test_page_path_hline() {
        let mut w = OfdWriter::new();
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_path(PathObject::hline(20.0, 40.0, 190.0));
        w.add_page(page);
        let bytes = w.build().unwrap();
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_page_path_rect_with_fill() {
        let mut w = OfdWriter::new();
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_path(
            PathObject::rect(10.0, 10.0, 100.0, 50.0)
                .stroke_color(0xFF_0000)
                .stroke_width(1.0)
                .fill_color(0x00_FF00),
        );
        w.add_page(page);
        let bytes = w.build().unwrap();
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_page_path_special_chars_in_data() {
        let mut w = OfdWriter::new();
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_path(PathObject::new(0.0, 0.0, "M0&0<10"));
        w.add_page(page);
        let bytes = w.build().unwrap();
        assert!(!bytes.is_empty());
    }

    // ── Mixed content on one page ─────────────────────────────────────────────

    #[test]
    fn test_page_mixed_content() {
        let mut w = OfdWriter::new();
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_text(TextObject::new(20.0, 30.0, "Invoice"));
        page.add_text(TextObject::new(20.0, 50.0, "$100.00"));
        page.add_image(ImageObject::jpeg(150.0, 30.0, 30.0, 30.0, vec![0xFF]));
        page.add_path(PathObject::hline(20.0, 45.0, 190.0));
        w.add_page(page);
        let bytes = w.build().unwrap();
        let names = zip_entry_names(&bytes);
        assert!(names.contains(&"Doc_0/Res/Image_0.jpeg".to_string()));
        assert!(names.contains(&"Doc_0/Pages/Page_0.xml".to_string()));
    }

    // ── Page dimensions ───────────────────────────────────────────────────────

    #[test]
    fn test_custom_page_dimensions() {
        let mut w = OfdWriter::new();
        let mut page = OfdPage::new(297.0, 420.0); // A3
        page.add_text(TextObject::new(0.0, 0.0, "A3 page"));
        w.add_page(page);
        let bytes = w.build().unwrap();
        assert!(!bytes.is_empty());
    }

    // ── xml_escape ────────────────────────────────────────────────────────────

    #[test]
    fn test_xml_escape_empty() {
        assert_eq!(xml_escape(""), "");
    }

    #[test]
    fn test_xml_escape_no_special() {
        assert_eq!(xml_escape("hello world"), "hello world");
    }

    #[test]
    fn test_xml_escape_all_special() {
        assert_eq!(
            xml_escape("a&b<c>d\"e'f"),
            "a&amp;b&lt;c&gt;d&quot;e&apos;f"
        );
    }

    #[test]
    fn test_xml_escape_only_ampersand() {
        assert_eq!(xml_escape("&"), "&amp;");
    }

    #[test]
    fn test_xml_escape_only_lt() {
        assert_eq!(xml_escape("<"), "&lt;");
    }

    #[test]
    fn test_xml_escape_only_gt() {
        assert_eq!(xml_escape(">"), "&gt;");
    }

    #[test]
    fn test_xml_escape_only_quote() {
        assert_eq!(xml_escape("\""), "&quot;");
    }

    #[test]
    fn test_xml_escape_only_apos() {
        assert_eq!(xml_escape("'"), "&apos;");
    }

    // ── Error helpers ─────────────────────────────────────────────────────────

    #[test]
    fn test_zip_err() {
        let zip_err = zip::result::ZipError::FileNotFound;
        let err = super::zip_err(zip_err);
        assert!(format!("{err}").contains("ZIP error"));
    }

    #[test]
    fn test_io_err() {
        let io_err = std::io::Error::new(std::io::ErrorKind::Other, "test");
        let err = super::io_err(io_err);
        assert!(format!("{err}").contains("I/O error"));
    }

    #[test]
    fn test_images_on_multiple_pages_use_distinct_resources() {
        let mut writer = OfdWriter::new();
        let mut first = OfdPage::new(210.0, 297.0);
        first.add_image(ImageObject::png(0.0, 0.0, 10.0, 10.0, vec![1]));
        let mut second = OfdPage::new(210.0, 297.0);
        second.add_image(ImageObject::png(0.0, 0.0, 10.0, 10.0, vec![2]));
        writer.add_pages(vec![first, second]);
        let bytes = writer.build().unwrap();
        let reader = easyofd_reader::OfdReader::from_bytes(&bytes).unwrap();
        let ContentObject::Image(first_image) = &reader.pages()[0].content[0] else {
            panic!("expected first image");
        };
        let ContentObject::Image(second_image) = &reader.pages()[1].content[0] else {
            panic!("expected second image");
        };
        assert_eq!(first_image.data, vec![1]);
        assert_eq!(second_image.data, vec![2]);
    }

    // ── Helpers ───────────────────────────────────────────────────────────────

    /// Extract all entry names from a ZIP byte slice.
    fn zip_entry_names(bytes: &[u8]) -> Vec<String> {
        let cursor = std::io::Cursor::new(bytes);
        let mut archive = zip::ZipArchive::new(cursor).unwrap();
        (0..archive.len())
            .map(|i| archive.by_index(i).unwrap().name().to_string())
            .collect()
    }
}

