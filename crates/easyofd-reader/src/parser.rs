//! OFD XML 解析函数。
//!
//! 包含从 ZIP 归档中解析 OFD 文档结构的各个函数。

use std::collections::HashMap;
use std::io::{BufReader, Cursor, Read, Seek};

use easyofd_core::model::bookmark::Bookmark;
use easyofd_core::model::bookmarks::Bookmarks;
use easyofd_core::model::custom_data::CustomData;
use easyofd_core::model::custom_datas::CustomDatas;
use easyofd_core::{
    ContentObject, ImageFormat, ImageObject, OfdError, OfdPage, OfdResult, PathObject, TextObject,
};
use quick_xml::Reader as XmlReader;
use quick_xml::events::Event;

/// Parsed result from OFD.xml.
pub(crate) struct OfdEntry {
    /// Document directory path (e.g. "Doc_0").
    pub(crate) doc_root: String,
    /// Document identifier (ofd:DocID), if present.
    pub(crate) doc_id: Option<String>,
    /// Document author (ofd:Author), if present.
    pub(crate) author: Option<String>,
    /// Creator application name (ofd:Creator), if present.
    pub(crate) creator: Option<String>,
    /// Creator application version (ofd:CreatorVersion), if present.
    pub(crate) creator_version: Option<String>,
    /// Last modification date (ofd:ModDate), if present.
    pub(crate) mod_date: Option<String>,
    /// Creation date (ofd:CreationDate), if present.
    pub(crate) creation_date: Option<String>,
    /// Maximum unit identifier (ofd:MaxUnitID).
    pub(crate) max_unit_id: u32,
    /// Custom data collection (ofd:CustomDatas).
    pub(crate) custom_datas: Option<CustomDatas>,
    /// Bookmarks (ofd:Bookmarks).
    pub(crate) bookmarks: Option<Bookmarks>,
}

pub(crate) fn parse_ofd_entry<R: Read + std::io::Seek>(
    archive: &mut zip::ZipArchive<R>,
) -> OfdResult<OfdEntry> {
    let xml = read_zip_entry(archive, "OFD.xml")?;
    let mut reader = XmlReader::from_reader(BufReader::new(Cursor::new(&xml)));
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    let mut doc_root = String::new();
    let mut doc_id = None;
    let mut author = None;
    let mut creator = None;
    let mut creator_version = None;
    let mut mod_date = None;
    let mut creation_date = None;
    let mut max_unit_id = 0_u32;
    let mut custom_datas: Option<CustomDatas> = None;
    let mut bookmarks: Option<Bookmarks> = None;
    let mut current_text_tag: Option<Vec<u8>> = None;
    let mut current_text = String::new();
    // Nested element tracking for CustomDatas and Bookmarks
    let mut in_custom_datas = false;
    let mut in_custom_data = false;
    let mut current_custom_data_name: Option<String> = None;
    let mut in_bookmarks = false;
    let mut in_bookmark = false;
    let mut current_bookmark_name: Option<String> = None;
    let mut current_bookmark_goto: Option<String> = None;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => match e.name().as_ref() {
                b"ofd:DocRoot" => {
                    current_text_tag = Some(b"ofd:DocRoot".to_vec());
                    current_text.clear();
                }
                b"ofd:DocID" => {
                    current_text_tag = Some(b"ofd:DocID".to_vec());
                    current_text.clear();
                }
                b"ofd:Author" => {
                    current_text_tag = Some(b"ofd:Author".to_vec());
                    current_text.clear();
                }
                b"ofd:Creator" => {
                    current_text_tag = Some(b"ofd:Creator".to_vec());
                    current_text.clear();
                }
                b"ofd:CreatorVersion" => {
                    current_text_tag = Some(b"ofd:CreatorVersion".to_vec());
                    current_text.clear();
                }
                b"ofd:ModDate" => {
                    current_text_tag = Some(b"ofd:ModDate".to_vec());
                    current_text.clear();
                }
                b"ofd:CreationDate" => {
                    current_text_tag = Some(b"ofd:CreationDate".to_vec());
                    current_text.clear();
                }
                b"ofd:MaxUnitID" => {
                    current_text_tag = Some(b"ofd:MaxUnitID".to_vec());
                    current_text.clear();
                }
                b"ofd:CustomDatas" => {
                    in_custom_datas = true;
                }
                b"ofd:CustomData" if in_custom_datas => {
                    in_custom_data = true;
                    current_custom_data_name = None;
                    for attr in e.attributes().flatten() {
                        if attr.key.as_ref() == b"Name" {
                            current_custom_data_name = Some(
                                attr.decoded_and_normalized_value(
                                    quick_xml::XmlVersion::Explicit1_0,
                                    reader.decoder(),
                                )
                                .unwrap_or_default()
                                .to_string(),
                            );
                        }
                    }
                    current_text.clear();
                }
                b"ofd:Bookmarks" => {
                    in_bookmarks = true;
                }
                b"ofd:Bookmark" if in_bookmarks => {
                    in_bookmark = true;
                    current_bookmark_name = None;
                    current_bookmark_goto = None;
                    for attr in e.attributes().flatten() {
                        let val = attr
                            .decoded_and_normalized_value(
                                quick_xml::XmlVersion::Explicit1_0,
                                reader.decoder(),
                            )
                            .unwrap_or_default()
                            .to_string();
                        match attr.key.as_ref() {
                            b"Name" => current_bookmark_name = Some(val),
                            b"GoTo" => current_bookmark_goto = Some(val),
                            _ => {}
                        }
                    }
                    current_text.clear();
                }
                _ => {}
            },
            Ok(Event::Text(ref e)) => {
                let text = e
                    .xml10_content()
                    .map(|c| c.into_owned())
                    .unwrap_or_default();
                if current_text_tag.is_some() {
                    current_text = text;
                } else if in_custom_data {
                    current_text.push_str(&text);
                }
            }
            Ok(Event::End(ref end)) => {
                let tag_name = end.name();
                match tag_name.as_ref() {
                    b"ofd:DocRoot" => {
                        doc_root.clone_from(&current_text);
                        current_text_tag = None;
                    }
                    b"ofd:DocID" => {
                        doc_id = Some(current_text.clone());
                        current_text_tag = None;
                    }
                    b"ofd:Author" => {
                        author = Some(current_text.clone());
                        current_text_tag = None;
                    }
                    b"ofd:Creator" => {
                        creator = Some(current_text.clone());
                        current_text_tag = None;
                    }
                    b"ofd:CreatorVersion" => {
                        creator_version = Some(current_text.clone());
                        current_text_tag = None;
                    }
                    b"ofd:ModDate" => {
                        mod_date = Some(current_text.clone());
                        current_text_tag = None;
                    }
                    b"ofd:CreationDate" => {
                        creation_date = Some(current_text.clone());
                        current_text_tag = None;
                    }
                    b"ofd:MaxUnitID" => {
                        max_unit_id = current_text.trim().parse().unwrap_or(0);
                        current_text_tag = None;
                    }
                    b"ofd:CustomData" if in_custom_data => {
                        if let Some(name) = current_custom_data_name.take() {
                            let datas = custom_datas.get_or_insert_with(CustomDatas::new);
                            datas.push(CustomData::new(name, current_text.trim()));
                        }
                        in_custom_data = false;
                        current_text.clear();
                    }
                    b"ofd:CustomDatas" => {
                        in_custom_datas = false;
                    }
                    b"ofd:Bookmark" if in_bookmark => {
                        let name = current_bookmark_name
                            .take()
                            .unwrap_or_else(|| current_text.trim().to_string());
                        if !name.is_empty() {
                            let mut bm = Bookmark::new(name);
                            if let Some(target) = current_bookmark_goto.take() {
                                bm = bm.with_goto(target);
                            }
                            let bms = bookmarks.get_or_insert_with(Bookmarks::new);
                            bms.push(bm);
                        }
                        in_bookmark = false;
                        current_text.clear();
                    }
                    b"ofd:Bookmarks" => {
                        in_bookmarks = false;
                    }
                    _ => {
                        // Generic text tag end handling
                        if current_text_tag.is_some() {
                            current_text_tag = None;
                            current_text.clear();
                        }
                    }
                }
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
    let doc_root = doc_root
        .strip_suffix("/Document.xml")
        .unwrap_or(&doc_root)
        .to_string();

    Ok(OfdEntry {
        doc_root,
        doc_id,
        author,
        creator,
        creator_version,
        mod_date,
        creation_date,
        max_unit_id,
        custom_datas,
        bookmarks,
    })
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
    /// Original path as stored in the XML (e.g. `"qrcode.png"`),
    /// before `BaseLoc` is prepended.
    #[allow(dead_code)]
    pub(crate) original_path: String,
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
                    let original_path = raw_path.clone();
                    let location = if base.is_empty() {
                        raw_path
                    } else {
                        format!("{}/{}", base.trim_end_matches('/'), raw_path)
                    };
                    resources.insert(
                        id,
                        ResourceEntry {
                            location,
                            original_path,
                            format,
                        },
                    );
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
