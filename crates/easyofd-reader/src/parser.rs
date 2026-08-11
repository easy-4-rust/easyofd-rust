//! OFD XML 解析函数。
//!
//! 包含从 ZIP 归档中解析 OFD 文档结构的各个函数。

use std::collections::HashMap;
use std::io::{BufReader, Cursor, Read, Seek};

use easyofd_core::{
    ContentObject, ImageFormat, ImageObject, OfdError, OfdPage, OfdResult, PathObject, TextObject,
};
use quick_xml::Reader as XmlReader;
use quick_xml::events::Event;

pub(crate) fn parse_ofd_entry<R: Read + std::io::Seek>(
    archive: &mut zip::ZipArchive<R>,
) -> OfdResult<String> {
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
pub(crate) fn parse_document_entry<R: Read + std::io::Seek>(
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

/// 对应 Java: ofdrw/ofdrw-parser/ResourceParser
///
/// A parsed resource entry from DocumentRes.xml.  The `location` field
/// already incorporates the `BaseLoc` attribute from the enclosing
/// `<ofd:Res>` element (per GB/T 33190-2016).  Child
/// `<ofd:MediaFile>` paths are resolved relative to this base location.
#[derive(Debug, Clone)]
pub(crate) struct ResourceEntry {
    /// Final resolved path relative to the document directory (e.g.
    /// `"Res/qrcode.png"`).  This already incorporates `base_loc`.
    pub(crate) location: String,
    pub(crate) format: ImageFormat,
}

/// 解析 `DocumentRes.xml`，提取所有多媒体资源。
///
/// 对应 Java: ofdrw/ofdrw-parser/ResourceParser#parseBaseLoc
///
/// Honours the `BaseLoc` attribute on `<ofd:Res>` (GB/T 33190-2016
/// §12.3).  Child `<ofd:MediaFile>` paths are resolved relative to
/// the enclosing `BaseLoc`.
pub(crate) fn parse_document_resources<R: Read + Seek>(
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
    // Stack of BaseLoc values for nested `<ofd:Res>` elements.
    // For a flat document this will be at most one entry.
    let mut base_loc_stack: Vec<String> = Vec::new();
    let mut resources = HashMap::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref event)) if event.name().as_ref() == b"ofd:Res" => {
                // 对应 Java: ResourceParser#parseBaseLoc
                let mut base_loc = String::new();
                for attribute in event.attributes().flatten() {
                    if attribute.key.as_ref() == b"BaseLoc" {
                        base_loc = attribute
                            .decoded_and_normalized_value(
                                quick_xml::XmlVersion::Explicit1_0,
                                reader.decoder(),
                            )
                            .unwrap_or_default()
                            .to_string();
                    }
                }
                base_loc_stack.push(base_loc);
            }
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
                    let raw_path = event
                        .xml10_content()
                        .map(|value| value.into_owned())
                        .unwrap_or_default();
                    // Prepend the current BaseLoc (if any) to the
                    // MediaFile path so that the location stored in
                    // ResourceEntry is already relative to the
                    // document directory.
                    let base = base_loc_stack.last().map_or("", String::as_str);
                    let location = if base.is_empty() {
                        raw_path
                    } else {
                        format!("{}/{}", base.trim_end_matches('/'), raw_path)
                    };
                    resources.insert(id, ResourceEntry { location, format });
                }
            }
            Ok(Event::End(ref event)) if event.name().as_ref() == b"ofd:MediaFile" => {
                in_media_file = false;
            }
            Ok(Event::End(ref event)) if event.name().as_ref() == b"ofd:Res" => {
                base_loc_stack.pop();
            }
            Ok(Event::Eof) => break,
            Err(error) => return Err(OfdError::Xml(format!("{path}: {error}"))),
            _ => {}
        }
        buf.clear();
    }
    Ok(resources)
}

pub(crate) fn parse_image_format(value: &str) -> ImageFormat {
    match value.to_ascii_uppercase().as_str() {
        "PNG" => ImageFormat::Png,
        "BMP" => ImageFormat::Bmp,
        "TIFF" | "TIF" => ImageFormat::Tiff,
        _ => ImageFormat::Jpeg,
    }
}

/// Parse Page_N.xml → return `OfdPage` with dimensions and content objects.
pub(crate) fn parse_page_entry<R: Read + std::io::Seek>(
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

pub(crate) struct TextObjectBuilder {
    x: f64,
    y: f64,
    text: String,
    font: Option<String>,
    size: Option<f64>,
    width: Option<f64>,
    height: Option<f64>,
}

pub(crate) fn parse_text_object_attrs(
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

pub(crate) struct PathObjectBuilder {
    x: f64,
    y: f64,
    stroke_color: u32,
    stroke_width: f64,
    fill_color: Option<u32>,
    path_data: String,
}

pub(crate) fn parse_path_object_attrs(
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

pub(crate) fn parse_hex_color(value: &str) -> Option<u32> {
    u32::from_str_radix(value.trim_start_matches('#'), 16).ok()
}

pub(crate) fn resolve_xml_reference(value: &str) -> Option<char> {
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

pub(crate) struct ImageObjectBuilder {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    format: ImageFormat,
    resource_id: String,
}

#[allow(clippy::many_single_char_names)]
pub(crate) fn parse_image_object_attrs(
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

pub(crate) fn read_zip_entry<R: Read + std::io::Seek>(
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

pub(crate) fn resolve_resource_path(doc_dir: &str, location: &str) -> OfdResult<String> {
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
