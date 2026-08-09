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

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Cursor, Read, Seek};

use easyofd_core::{
    ContentObject, ImageFormat, ImageObject, OfdError, OfdPage, OfdResult, PathObject, TextObject,
};
use easyofd_package::{PackageLimits, validate_archive};
use quick_xml::Reader as XmlReader;
use quick_xml::events::Event;

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
fn parse_ofd_entry<R: Read + std::io::Seek>(archive: &mut zip::ZipArchive<R>) -> OfdResult<String> {
    let xml = read_zip_entry(archive, "OFD.xml")?;
    let mut reader = XmlReader::from_reader(BufReader::new(Cursor::new(&xml)));
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    let mut doc_root = String::new();
    let mut in_target = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) if e.name().as_ref() == b"ofd:DocRoot" => {
                in_target = true;
            }
            Ok(Event::Text(ref e)) if in_target => {
                doc_root = e
                    .xml10_content()
                    .map(|c| c.into_owned())
                    .unwrap_or_default();
            }
            Ok(Event::End(ref e)) if e.name().as_ref() == b"ofd:DocRoot" => {
                in_target = false;
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(OfdError::Xml(format!("OFD.xml: {e}"))),
            _ => {}
        }
        buf.clear();
    }

    if doc_root.is_empty() {
        return Err(OfdError::InvalidDocument("missing DocRoot".into()));
    }

    // Strip "/Document.xml" suffix to get the doc directory
    Ok(doc_root
        .strip_suffix("/Document.xml")
        .unwrap_or(&doc_root)
        .to_string())
}

/// Parse Document.xml → return list of page BaseLoc paths (e.g. "Pages/Page_0.xml").
fn parse_document_entry<R: Read + std::io::Seek>(
    archive: &mut zip::ZipArchive<R>,
    doc_dir: &str,
) -> OfdResult<Vec<String>> {
    let path = format!("{doc_dir}/Document.xml");
    let xml = read_zip_entry(archive, &path)?;
    let mut reader = XmlReader::from_reader(BufReader::new(Cursor::new(&xml)));
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    let mut pages = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Empty(ref e) | Event::Start(ref e)) if e.name().as_ref() == b"ofd:Page" => {
                for attr in e.attributes().flatten() {
                    if attr.key.as_ref() == b"BaseLoc" {
                        let val = attr
                            .decoded_and_normalized_value(
                                quick_xml::XmlVersion::Explicit1_0,
                                reader.decoder(),
                            )
                            .unwrap_or_default();
                        pages.push(val.to_string());
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(OfdError::Xml(format!("Document.xml: {e}"))),
            _ => {}
        }
        buf.clear();
    }
    Ok(pages)
}

#[derive(Debug, Clone)]
struct ResourceEntry {
    location: String,
    format: ImageFormat,
}

fn parse_document_resources<R: Read + Seek>(
    archive: &mut zip::ZipArchive<R>,
    doc_dir: &str,
) -> OfdResult<HashMap<String, ResourceEntry>> {
    let path = format!("{doc_dir}/DocumentRes.xml");
    let xml = match read_zip_entry(archive, &path) {
        Ok(xml) => xml,
        Err(_) => return Ok(HashMap::new()),
    };
    let mut reader = XmlReader::from_reader(BufReader::new(Cursor::new(&xml)));
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    let mut current: Option<(String, ImageFormat)> = None;
    let mut in_media_file = false;
    let mut resources = HashMap::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref event)) if event.name().as_ref() == b"ofd:MultiMedia" => {
                let mut id = None;
                let mut format = ImageFormat::Jpeg;
                for attribute in event.attributes().flatten() {
                    let value = attribute
                        .decoded_and_normalized_value(
                            quick_xml::XmlVersion::Explicit1_0,
                            reader.decoder(),
                        )
                        .unwrap_or_default();
                    match attribute.key.as_ref() {
                        b"ID" => id = Some(value.to_string()),
                        b"Type" => format = parse_image_format(&value),
                        _ => {}
                    }
                }
                current = id.map(|id| (id, format));
            }
            Ok(Event::Start(ref event)) if event.name().as_ref() == b"ofd:MediaFile" => {
                in_media_file = true;
            }
            Ok(Event::Text(ref event)) if in_media_file => {
                if let Some((id, format)) = current.take() {
                    let location = event
                        .xml10_content()
                        .map(|value| value.into_owned())
                        .unwrap_or_default();
                    resources.insert(id, ResourceEntry { location, format });
                }
            }
            Ok(Event::End(ref event)) if event.name().as_ref() == b"ofd:MediaFile" => {
                in_media_file = false;
            }
            Ok(Event::Eof) => break,
            Err(error) => return Err(OfdError::Xml(format!("{path}: {error}"))),
            _ => {}
        }
        buf.clear();
    }
    Ok(resources)
}

fn parse_image_format(value: &str) -> ImageFormat {
    match value.to_ascii_uppercase().as_str() {
        "PNG" => ImageFormat::Png,
        "BMP" => ImageFormat::Bmp,
        "TIFF" | "TIF" => ImageFormat::Tiff,
        _ => ImageFormat::Jpeg,
    }
}

/// Parse Page_N.xml → return `OfdPage` with dimensions and content objects.
fn parse_page_entry<R: Read + std::io::Seek>(
    archive: &mut zip::ZipArchive<R>,
    page_path: &str,
    doc_dir: &str,
    resources: &HashMap<String, ResourceEntry>,
) -> OfdResult<OfdPage> {
    let xml = read_zip_entry(archive, page_path)?;
    let mut reader = XmlReader::from_reader(BufReader::new(Cursor::new(&xml)));
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();

    let mut width = 210.0_f64;
    let mut height = 297.0_f64;
    let mut content = Vec::new();

    let mut current_text: Option<TextObjectBuilder> = None;
    let mut current_path: Option<PathObjectBuilder> = None;
    let mut in_text_code = false;
    let mut in_path_data = false;
    let mut in_physical_box = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e) | Event::Empty(ref e)) => match e.name().as_ref() {
                b"ofd:PhysicalBox" => in_physical_box = true,
                b"ofd:TextObject" => {
                    current_text = Some(parse_text_object_attrs(e, reader.decoder())?)
                }
                b"ofd:TextCode" => in_text_code = true,
                b"ofd:PathObject" => {
                    current_path = Some(parse_path_object_attrs(e, reader.decoder())?)
                }
                b"ofd:AbbreviatedData" => in_path_data = true,
                b"ofd:ImageObject" => {
                    let img = parse_image_object_attrs(e, reader.decoder())?;
                    let (data, format) = if let Some(resource) = resources.get(&img.resource_id) {
                        let resource_path = resolve_resource_path(doc_dir, &resource.location)?;
                        (read_zip_entry(archive, &resource_path)?, resource.format)
                    } else {
                        (Vec::new(), img.format)
                    };
                    content.push(ContentObject::Image(ImageObject::new(
                        img.x, img.y, img.width, img.height, data, format,
                    )));
                }
                _ => {}
            },
            Ok(Event::Text(ref e)) => {
                let text = e
                    .xml10_content()
                    .map(|c| c.into_owned())
                    .unwrap_or_default();
                if in_physical_box {
                    let parts: Vec<f64> = text
                        .split_whitespace()
                        .filter_map(|s| s.parse().ok())
                        .collect();
                    if parts.len() >= 4 {
                        width = parts[2];
                        height = parts[3];
                    }
                }
                if in_text_code {
                    if let Some(ref mut t) = current_text {
                        t.text.push_str(&text);
                    }
                } else if in_path_data {
                    if let Some(ref mut path) = current_path {
                        path.path_data.push_str(&text);
                    }
                }
            }
            Ok(Event::GeneralRef(ref reference)) => {
                let name = reference
                    .xml10_content()
                    .map(|value| value.into_owned())
                    .unwrap_or_default();
                let value = resolve_xml_reference(&name).ok_or_else(|| {
                    OfdError::Xml(format!("{page_path}: unresolved entity &{name};"))
                })?;
                if in_text_code {
                    if let Some(ref mut text) = current_text {
                        text.text.push(value);
                    }
                } else if in_path_data {
                    if let Some(ref mut path) = current_path {
                        path.path_data.push(value);
                    }
                }
            }
            Ok(Event::End(ref e)) => match e.name().as_ref() {
                b"ofd:PhysicalBox" => in_physical_box = false,
                b"ofd:TextObject" => {
                    if let Some(t) = current_text.take() {
                        let mut obj = TextObject::new(t.x, t.y, t.text);
                        if let Some(f) = t.font {
                            obj = obj.font(f);
                        }
                        if let Some(s) = t.size {
                            obj = obj.size(s);
                        }
                        obj.width = t.width;
                        obj.height = t.height;
                        content.push(ContentObject::Text(obj));
                    }
                }
                b"ofd:TextCode" => in_text_code = false,
                b"ofd:PathObject" => {
                    if let Some(path) = current_path.take() {
                        let mut object = PathObject::new(path.x, path.y, path.path_data)
                            .stroke_color(path.stroke_color)
                            .stroke_width(path.stroke_width);
                        if let Some(fill_color) = path.fill_color {
                            object = object.fill_color(fill_color);
                        }
                        content.push(ContentObject::Path(object));
                    }
                }
                b"ofd:AbbreviatedData" => in_path_data = false,
                _ => {}
            },
            Ok(Event::Eof) => break,
            Err(e) => return Err(OfdError::Xml(format!("{page_path}: {e}"))),
            _ => {}
        }
        buf.clear();
    }

    Ok(OfdPage {
        width,
        height,
        content,
    })
}

// ─── Attribute Parsing Helpers ───────────────────────────────────────────────

struct TextObjectBuilder {
    x: f64,
    y: f64,
    text: String,
    font: Option<String>,
    size: Option<f64>,
    width: Option<f64>,
    height: Option<f64>,
}

fn parse_text_object_attrs(
    e: &quick_xml::events::BytesStart,
    decoder: quick_xml::encoding::Decoder,
) -> OfdResult<TextObjectBuilder> {
    let mut x = 0.0_f64;
    let mut y = 0.0_f64;
    let mut font = None;
    let mut size = None;
    let mut width = None;
    let mut height = None;

    for attr in e.attributes().flatten() {
        match attr.key.as_ref() {
            b"Boundary" => {
                let parts: Vec<f64> = attr
                    .decoded_and_normalized_value(quick_xml::XmlVersion::Explicit1_0, decoder)
                    .unwrap_or_default()
                    .split_whitespace()
                    .filter_map(|s| s.parse().ok())
                    .collect();
                if parts.len() >= 2 {
                    x = parts[0];
                    y = parts[1];
                }
                if parts.len() >= 4 {
                    width = Some(parts[2]);
                    height = Some(parts[3]);
                }
            }
            b"Font" => {
                font = Some(
                    attr.decoded_and_normalized_value(quick_xml::XmlVersion::Explicit1_0, decoder)
                        .unwrap_or_default()
                        .to_string(),
                );
            }
            b"Size" => {
                size = attr
                    .decoded_and_normalized_value(quick_xml::XmlVersion::Explicit1_0, decoder)
                    .unwrap_or_default()
                    .parse()
                    .ok();
            }
            _ => {}
        }
    }

    Ok(TextObjectBuilder {
        x,
        y,
        text: String::new(),
        font,
        size,
        width,
        height,
    })
}

struct PathObjectBuilder {
    x: f64,
    y: f64,
    stroke_color: u32,
    stroke_width: f64,
    fill_color: Option<u32>,
    path_data: String,
}

fn parse_path_object_attrs(
    event: &quick_xml::events::BytesStart,
    decoder: quick_xml::encoding::Decoder,
) -> OfdResult<PathObjectBuilder> {
    let mut builder = PathObjectBuilder {
        x: 0.0,
        y: 0.0,
        stroke_color: 0,
        stroke_width: 0.35,
        fill_color: None,
        path_data: String::new(),
    };
    for attribute in event.attributes().flatten() {
        let value = attribute
            .decoded_and_normalized_value(quick_xml::XmlVersion::Explicit1_0, decoder)
            .unwrap_or_default();
        match attribute.key.as_ref() {
            b"Boundary" => {
                let parts: Vec<f64> = value
                    .split_whitespace()
                    .filter_map(|part| part.parse().ok())
                    .collect();
                if parts.len() >= 2 {
                    builder.x = parts[0];
                    builder.y = parts[1];
                }
            }
            b"StrokeColor" => builder.stroke_color = parse_hex_color(&value).unwrap_or(0),
            b"FillColor" => builder.fill_color = parse_hex_color(&value),
            b"LineWidth" => builder.stroke_width = value.parse().unwrap_or(0.35),
            _ => {}
        }
    }
    Ok(builder)
}

fn parse_hex_color(value: &str) -> Option<u32> {
    u32::from_str_radix(value.trim_start_matches('#'), 16).ok()
}

fn resolve_xml_reference(value: &str) -> Option<char> {
    if let Some(entity) = quick_xml::escape::resolve_xml_entity(value) {
        return entity.chars().next();
    }
    let number = if let Some(hex) = value.strip_prefix("#x") {
        u32::from_str_radix(hex, 16).ok()
    } else if let Some(decimal) = value.strip_prefix('#') {
        decimal.parse().ok()
    } else {
        None
    }?;
    char::from_u32(number)
}

struct ImageObjectBuilder {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    format: ImageFormat,
    resource_id: String,
}

#[allow(clippy::many_single_char_names)]
fn parse_image_object_attrs(
    e: &quick_xml::events::BytesStart,
    decoder: quick_xml::encoding::Decoder,
) -> OfdResult<ImageObjectBuilder> {
    let mut x = 0.0_f64;
    let mut y = 0.0_f64;
    let mut w = 0.0_f64;
    let mut h = 0.0_f64;
    let mut resource_id = String::new();

    for attr in e.attributes().flatten() {
        if attr.key.as_ref() == b"Boundary" {
            let parts: Vec<f64> = attr
                .decoded_and_normalized_value(quick_xml::XmlVersion::Explicit1_0, decoder)
                .unwrap_or_default()
                .split_whitespace()
                .filter_map(|s| s.parse().ok())
                .collect();
            if parts.len() >= 4 {
                x = parts[0];
                y = parts[1];
                w = parts[2];
                h = parts[3];
            }
        } else if attr.key.as_ref() == b"ResourceID" {
            resource_id = attr
                .decoded_and_normalized_value(quick_xml::XmlVersion::Explicit1_0, decoder)
                .unwrap_or_default()
                .to_string();
        }
    }

    Ok(ImageObjectBuilder {
        x,
        y,
        width: w,
        height: h,
        format: ImageFormat::Jpeg,
        resource_id,
    })
}

// ─── ZIP Helper ──────────────────────────────────────────────────────────────

fn read_zip_entry<R: Read + std::io::Seek>(
    archive: &mut zip::ZipArchive<R>,
    name: &str,
) -> OfdResult<Vec<u8>> {
    let mut file = archive
        .by_name(name)
        .map_err(|e| OfdError::Zip(format!("{name}: {e}")))?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).map_err(OfdError::Io)?;
    Ok(buf)
}

fn resolve_resource_path(doc_dir: &str, location: &str) -> OfdResult<String> {
    let location = location.trim_start_matches('/');
    let path = if location.starts_with(doc_dir) {
        location.to_string()
    } else {
        format!("{doc_dir}/{location}")
    };
    easyofd_package::validate_entry_name(&path)?;
    Ok(path)
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use easyofd_core::OfdPage;
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
