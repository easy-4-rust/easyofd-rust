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

mod ofd_reader;
mod parser;
mod read_options;

pub use ofd_reader::OfdReader;
pub use read_options::ReadOptions;

#[cfg(test)]
mod tests {
    use super::*;
    use easyofd_core::{ImageObject, PathObject, TextObject};
    use easyofd_writer::OfdWriter;

    use easyofd_core::OfdPage;

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
        let easyofd_core::ContentObject::Image(image) = &reader.pages()[0].content[1] else {
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
                visited.push((number, ofd_reader::page_text(&page)));
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
            easyofd_core::ContentObject::Path(_)
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

    // ─── BaseLoc resource resolution tests ─────────────────────────────────

    /// Build a minimal OFD ZIP whose `Doc_0/DocumentRes.xml` is the given
    /// XML string.  Returns the raw bytes of the ZIP archive.
    fn build_zip_with_document_res(document_res_xml: &str) -> Vec<u8> {
        use std::io::Write;

        let mut buf = std::io::Cursor::new(Vec::new());
        {
            let mut zip = zip::ZipWriter::new(&mut buf);
            let options = zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Stored);

            // OFD.xml
            zip.start_file("OFD.xml", options).unwrap();
            zip.write_all(
                br#"<?xml version="1.0" encoding="UTF-8"?>
<ofd:OFD xmlns:ofd="http://www.ofdspec.org/2016">
  <ofd:DocBody><ofd:DocRoot>Doc_0/Document.xml</ofd:DocRoot></ofd:DocBody>
</ofd:OFD>"#,
            )
            .unwrap();

            // Doc_0/Document.xml (single empty page)
            zip.start_file("Doc_0/Document.xml", options).unwrap();
            zip.write_all(
                br#"<?xml version="1.0" encoding="UTF-8"?>
<ofd:Document xmlns:ofd="http://www.ofdspec.org/2016">
  <ofd:CommonData>
    <ofd:PageArea><ofd:PhysicalBox>0 0 210 297</ofd:PhysicalBox></ofd:PageArea>
    <ofd:DocumentRes>DocumentRes.xml</ofd:DocumentRes>
  </ofd:CommonData>
  <ofd:Pages><ofd:Page BaseLoc="Pages/Page_0/Content.xml"/></ofd:Pages>
</ofd:Document>"#,
            )
            .unwrap();

            // Doc_0/Pages/Page_0/Content.xml (empty page)
            zip.start_file("Doc_0/Pages/Page_0/Content.xml", options)
                .unwrap();
            zip.write_all(
                br#"<?xml version="1.0" encoding="UTF-8"?>
<ofd:Page xmlns:ofd="http://www.ofdspec.org/2016">
  <ofd:Content/>
</ofd:Page>"#,
            )
            .unwrap();

            // Doc_0/DocumentRes.xml (caller-supplied content)
            zip.start_file("Doc_0/DocumentRes.xml", options).unwrap();
            zip.write_all(document_res_xml.as_bytes()).unwrap();

            zip.finish().unwrap();
        }
        buf.into_inner()
    }

    /// Helper: parse the DocumentRes.xml inside a test ZIP and return
    /// the `ResourceEntry` map keyed by resource ID.
    fn parse_resources_from_zip(bytes: &[u8]) -> std::collections::HashMap<String, String> {
        use parser::parse_document_resources;

        let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes)).unwrap();
        let resources = parse_document_resources(&mut archive, "Doc_0").unwrap();
        resources
            .into_iter()
            .map(|(id, entry)| (id, entry.location))
            .collect()
    }

    /// 对应 Java: ofdrw/ofdrw-parser/ResourceParser#parseBaseLoc
    ///
    /// When `<ofd:Res BaseLoc="Res">` is present, MediaFile paths must be
    /// resolved relative to the `Res/` subdirectory.
    #[test]
    fn parse_document_resources_respects_base_loc() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<ofd:Res xmlns:ofd="http://www.ofdspec.org/2016" BaseLoc="Res">
  <ofd:MultiMedias>
    <ofd:MultiMedia ID="6" Type="Image" Format="PNG">
      <ofd:MediaFile>qrcode.png</ofd:MediaFile>
    </ofd:MultiMedia>
  </ofd:MultiMedias>
</ofd:Res>"#;
        let bytes = build_zip_with_document_res(xml);
        let resources = parse_resources_from_zip(&bytes);

        assert_eq!(
            resources.get("6").map(String::as_str),
            Some("Res/qrcode.png"),
            "MediaFile path must include BaseLoc prefix"
        );
    }

    /// When there is no `BaseLoc` attribute, paths are stored as-is
    /// (backward-compatible with the pre-fix behaviour).
    #[test]
    fn parse_document_resources_no_base_loc_uses_default() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<ofd:Res xmlns:ofd="http://www.ofdspec.org/2016">
  <ofd:MultiMedias>
    <ofd:MultiMedia ID="10" Type="Image" Format="JPEG">
      <ofd:MediaFile>photo.jpg</ofd:MediaFile>
    </ofd:MultiMedia>
  </ofd:MultiMedias>
</ofd:Res>"#;
        let bytes = build_zip_with_document_res(xml);
        let resources = parse_resources_from_zip(&bytes);

        assert_eq!(
            resources.get("10").map(String::as_str),
            Some("photo.jpg"),
            "without BaseLoc, raw MediaFile path is kept"
        );
    }

    /// Nested `<ofd:Res>` elements must each maintain their own BaseLoc
    /// on the stack.  Resources inside the inner Res use the inner
    /// BaseLoc; resources after the inner Res close tag use the outer.
    #[test]
    fn parse_document_resources_nested_res() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<ofd:Res xmlns:ofd="http://www.ofdspec.org/2016" BaseLoc="Outer">
  <ofd:MultiMedias>
    <ofd:MultiMedia ID="1" Type="Image">
      <ofd:MediaFile>outer.png</ofd:MediaFile>
    </ofd:MultiMedia>
  </ofd:MultiMedias>
  <ofd:Res BaseLoc="Inner">
    <ofd:MultiMedias>
      <ofd:MultiMedia ID="2" Type="Image">
        <ofd:MediaFile>inner.png</ofd:MediaFile>
      </ofd:MultiMedia>
    </ofd:MultiMedias>
  </ofd:Res>
  <ofd:MultiMedias>
    <ofd:MultiMedia ID="3" Type="Image">
      <ofd:MediaFile>back_outer.png</ofd:MediaFile>
    </ofd:MultiMedia>
  </ofd:MultiMedias>
</ofd:Res>"#;
        let bytes = build_zip_with_document_res(xml);
        let resources = parse_resources_from_zip(&bytes);

        assert_eq!(
            resources.get("1").map(String::as_str),
            Some("Outer/outer.png"),
            "outer resource uses outer BaseLoc"
        );
        assert_eq!(
            resources.get("2").map(String::as_str),
            Some("Inner/inner.png"),
            "inner resource uses inner BaseLoc"
        );
        assert_eq!(
            resources.get("3").map(String::as_str),
            Some("Outer/back_outer.png"),
            "resource after inner Res closes uses outer BaseLoc again"
        );
    }
}
