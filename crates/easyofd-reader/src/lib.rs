#![allow(clippy::too_many_lines)]
//! # easyofd-reader
//!
//! OFD file reader that parses GB/T 33190-2016 compliant ZIP archives.
//!
//! ## Architecture
//!
//! ```text
//! input.ofd (ZIP)
//! ├── OFD.xml                    → find DocRoot
//! └── Doc_0/
//!     ├── Document.xml           → read page list
//!     └── Pages/
//!         ├── Page_0.xml         → parse content
//!         └── Page_N.xml
//! ```

mod parser;

use std::fs::File;
use std::io::{Cursor, Read, Seek};

use parser::{
    parse_document_entry, parse_document_resources, parse_ofd_entry, parse_page_entry,
};
use easyofd_core::{
    ContentObject, OfdError, OfdPage, OfdResult,
};
use easyofd_package::{PackageLimits, validate_archive};

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

/// An OFD document reader.
pub struct OfdReader {
    pages: Vec<OfdPage>,
}

impl OfdReader {
    /// Open and parse an OFD file from a path.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or contains invalid OFD data.
    pub fn open(path: impl AsRef<std::path::Path>) -> OfdResult<Self> {
        Self::open_with_options(path, ReadOptions::default())
    }

    /// 使用指定选项打开 OFD 文件。
    ///
    /// # Errors
    ///
    /// 文件、ZIP 包或 XML 无效时返回错误。
    pub fn open_with_options(
        path: impl AsRef<std::path::Path>,
        options: ReadOptions,
    ) -> OfdResult<Self> {
        let file = File::open(path)?;
        Self::from_seek(file, options)
    }

    /// Parse an OFD file from in-memory bytes.
    ///
    /// # Errors
    ///
    /// Returns an error if the data is invalid.
    pub fn from_bytes(data: &[u8]) -> OfdResult<Self> {
        Self::from_seek(Cursor::new(data), ReadOptions::default())
    }

    /// 从实现 `Read + Seek` 的输入读取文档。
    ///
    /// # Errors
    ///
    /// ZIP 包或 XML 无效时返回错误。
    pub fn from_seek<R: Read + Seek>(source: R, options: ReadOptions) -> OfdResult<Self> {
        let mut pages = Vec::new();
        visit_archive(source, options, |_, page| {
            pages.push(page);
            Ok(())
        })?;
        Ok(Self { pages })
    }

    /// 逐页访问文件，不在内存中保留已经处理过的页面。
    ///
    /// 回调页码从 1 开始。回调返回错误时立即停止解析。
    ///
    /// # Errors
    ///
    /// 文件、ZIP、XML 或页面回调失败时返回错误。
    pub fn visit_path(
        path: impl AsRef<std::path::Path>,
        options: ReadOptions,
        visitor: impl FnMut(usize, OfdPage) -> OfdResult<()>,
    ) -> OfdResult<usize> {
        visit_archive(File::open(path)?, options, visitor)
    }

    /// Number of pages in the document.
    #[must_use]
    pub fn page_count(&self) -> usize {
        self.pages.len()
    }

    /// All parsed pages.
    #[must_use]
    pub fn pages(&self) -> &[OfdPage] {
        &self.pages
    }

    /// Extract text from all pages, one `String` per page.
    #[must_use]
    pub fn extract_text(&self) -> Vec<String> {
        self.pages.iter().map(page_text).collect()
    }

    /// Extract all text joined into a single string with page separators.
    #[must_use]
    pub fn extract_all_text(&self) -> String {
        self.extract_text().join("\n---\n")
    }
}

fn visit_archive<R: Read + Seek>(
    source: R,
    options: ReadOptions,
    mut visitor: impl FnMut(usize, OfdPage) -> OfdResult<()>,
) -> OfdResult<usize> {
    let mut archive = zip::ZipArchive::new(source).map_err(|e| OfdError::Zip(e.to_string()))?;
    validate_archive(&mut archive, options.package_limits)?;
    let doc_root = parse_ofd_entry(&mut archive)?;
    let page_refs = parse_document_entry(&mut archive, &doc_root)?;
    let resources = parse_document_resources(&mut archive, &doc_root)?;
    let mut visited = 0;
    for (index, page_loc) in page_refs.iter().enumerate() {
        let page_number = index + 1;
        if options.first_page.is_some_and(|first| page_number < first)
            || options.last_page.is_some_and(|last| page_number > last)
        {
            continue;
        }
        let page_path = format!("{doc_root}/{page_loc}");
        let page = parse_page_entry(&mut archive, &page_path, &doc_root, &resources)?;
        visitor(page_number, page)?;
        visited += 1;
    }
    Ok(visited)
}

/// Join all text objects on a page into one string.
fn page_text(page: &OfdPage) -> String {
    page.content
        .iter()
        .filter_map(|obj| {
            if let ContentObject::Text(t) = obj {
                Some(t.text.as_str())
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

// ─── XML Parsing ─────────────────────────────────────────────────────────────

/// Parse OFD.xml → return the DocRoot directory (e.g. "Doc_0").
#[cfg(test)]
mod tests {
    use super::*;
    use easyofd_core::{TextObject, ImageObject, PathObject};
    use easyofd_writer::OfdWriter;

    fn roundtrip(pages: Vec<OfdPage>) -> Vec<u8> {
        let mut writer = OfdWriter::new();
        for page in pages {
            writer.add_page(page);
        }
        writer.build().unwrap()
    }

    #[test]
    fn test_empty_document() {
        let bytes = OfdWriter::new().build().unwrap();
        let reader = OfdReader::from_bytes(&bytes).unwrap();
        assert_eq!(reader.page_count(), 0);
    }

    #[test]
    fn test_single_text_page() {
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_text(TextObject::new(20.0, 30.0, "Hello OFD Reader!"));
        let bytes = roundtrip(vec![page]);

        let reader = OfdReader::from_bytes(&bytes).unwrap();
        assert_eq!(reader.page_count(), 1);
        assert_eq!(reader.pages()[0].content.len(), 1);

        let text = reader.extract_text();
        assert_eq!(text.len(), 1);
        assert!(text[0].contains("Hello OFD Reader!"));
    }

    #[test]
    fn test_multiple_pages() {
        let mut pages = Vec::new();
        for i in 1..=3 {
            let mut page = OfdPage::new(210.0, 297.0);
            page.add_text(TextObject::new(10.0, 20.0, format!("Page {i} text")));
            pages.push(page);
        }
        let bytes = roundtrip(pages);

        let reader = OfdReader::from_bytes(&bytes).unwrap();
        assert_eq!(reader.page_count(), 3);
        let text = reader.extract_text();
        assert_eq!(text.len(), 3);
        assert!(text[0].contains("Page 1"));
        assert!(text[2].contains("Page 3"));
    }

    #[test]
    fn test_extract_all_text() {
        let mut p1 = OfdPage::new(210.0, 297.0);
        p1.add_text(TextObject::new(10.0, 20.0, "First"));
        let mut p2 = OfdPage::new(210.0, 297.0);
        p2.add_text(TextObject::new(10.0, 20.0, "Second"));
        let bytes = roundtrip(vec![p1, p2]);

        let reader = OfdReader::from_bytes(&bytes).unwrap();
        let all = reader.extract_all_text();
        assert!(all.contains("First"));
        assert!(all.contains("Second"));
        assert!(all.contains("---"));
    }

    #[test]
    fn test_text_and_image() {
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_text(TextObject::new(20.0, 30.0, "Invoice"));
        page.add_image(ImageObject::jpeg(150.0, 30.0, 30.0, 30.0, vec![0xFF, 0xD8]));
        let bytes = roundtrip(vec![page]);

        let reader = OfdReader::from_bytes(&bytes).unwrap();
        assert_eq!(reader.pages()[0].content.len(), 2);
        let ContentObject::Image(image) = &reader.pages()[0].content[1] else {
            panic!("expected image");
        };
        assert_eq!(image.data, vec![0xFF, 0xD8]);
    }

    #[test]
    fn test_visit_selected_pages_without_collecting() {
        let mut pages = Vec::new();
        for number in 1..=4 {
            let mut page = OfdPage::new(210.0, 297.0);
            page.add_text(TextObject::new(10.0, 10.0, format!("page {number}")));
            pages.push(page);
        }
        let bytes = roundtrip(pages);
        let path = std::env::temp_dir().join("easyofd_visit_pages.ofd");
        std::fs::write(&path, bytes).unwrap();
        let mut visited = Vec::new();
        let count = OfdReader::visit_path(
            &path,
            ReadOptions {
                first_page: Some(2),
                last_page: Some(3),
                ..ReadOptions::default()
            },
            |number, page| {
                visited.push((number, page_text(&page)));
                Ok(())
            },
        )
        .unwrap();
        assert_eq!(count, 2);
        assert_eq!(visited[0], (2, "page 2".to_string()));
        assert_eq!(visited[1], (3, "page 3".to_string()));
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn test_path_roundtrip_is_not_silently_lost() {
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_path(PathObject::hline(10.0, 20.0, 50.0));
        let bytes = roundtrip(vec![page]);
        let reader = OfdReader::from_bytes(&bytes).unwrap();
        assert!(matches!(
            reader.pages()[0].content[0],
            ContentObject::Path(_)
        ));
    }

    #[test]
    fn test_from_file() {
        let dir = std::env::temp_dir().join("easyofd_reader");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.ofd");

        let mut page = OfdPage::new(210.0, 297.0);
        page.add_text(TextObject::new(10.0, 20.0, "File test"));
        let mut w = OfdWriter::new();
        w.add_page(page);
        w.build_to_file(&path).unwrap();

        let reader = OfdReader::open(&path).unwrap();
        assert_eq!(reader.page_count(), 1);
        assert!(reader.extract_all_text().contains("File test"));
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_invalid_data() {
        assert!(OfdReader::from_bytes(b"not a zip file").is_err());
    }

    #[test]
    fn test_styled_text() {
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_text(
            TextObject::new(10.0, 20.0, "Styled")
                .font("SimHei")
                .size(18.0)
                .bold(),
        );
        let bytes = roundtrip(vec![page]);
        let reader = OfdReader::from_bytes(&bytes).unwrap();
        assert_eq!(reader.page_count(), 1);
        assert!(reader.extract_all_text().contains("Styled"));
    }
}
