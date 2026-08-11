//! OFD XML 解析函数。
//!
//! 包含从 ZIP 归档中解析 OFD 文档结构的各个函数。

use std::collections::HashMap;
use std::io::{BufReader, Cursor, Read, Seek};

use easyofd_core::model::bookmark::Bookmark;
use easyofd_core::model::bookmarks::Bookmarks;
use easyofd_core::model::custom_data::CustomData;
use easyofd_core::model::custom_datas::CustomDatas;
use easyofd_core::model::permissions::Permissions;
use easyofd_core::model::template_page::TemplatePage;
use easyofd_core::{
    ContentObject, ImageFormat, ImageObject, OfdError, OfdPage, OfdResult, PathObject, TextObject,
};
use quick_xml::Reader as XmlReader;
use quick_xml::events::Event;

/// Parsed result from OFD.xml.
pub(crate) struct OfdEntry {
    /// Document directory path (e.g. "Doc_0").
    pub(crate) doc_dir: String,
    /// Document XML file name inside `doc_dir` (usually "Document.xml",
    /// but ofdrw samples may use e.g. "Document_0.xml").
    pub(crate) document_file: String,
    /// Document identifier (ofd:DocID), if present.
    pub(crate) doc_id: Option<String>,
    /// Document title (ofd:Title), if present.
    pub(crate) title: Option<String>,
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
    /// Signature container path (ofd:Signatures), if present.
    pub(crate) signatures_path: Option<String>,
    /// Document usage (ofd:DocUsage), if present.
    pub(crate) doc_usage: Option<String>,
    /// Document keywords (ofd:Keywords), if present.
    pub(crate) keywords: Option<String>,
}

/// Parse OFD.xml using the [`easyofd_core::XmlNode`] tree (via
/// `parse_xml_to_nodes`) instead of a flat quick-xml event stream.
pub(crate) fn parse_ofd_entry<R: Read + std::io::Seek>(
    archive: &mut zip::ZipArchive<R>,
) -> OfdResult<OfdEntry> {
    let xml_bytes = read_zip_entry(archive, "OFD.xml")?;
    let xml_str = std::str::from_utf8(&xml_bytes)
        .map_err(|e| OfdError::Xml(format!("OFD.xml: invalid UTF-8: {e}")))?;

    let root = easyofd_core::parse_xml_to_nodes(xml_str)
        .map_err(|e| OfdError::Xml(format!("OFD.xml: {e}")))?;

    // ── DocRoot (required) ──
    let doc_root = find_text_deep(&root, "DocRoot")
        .filter(|s| !s.is_empty())
        .ok_or_else(|| OfdError::InvalidDocument("missing DocRoot".into()))?;

    // DocRoot points at the Document XML file directly (e.g.
    // "Doc_0/Document.xml" or "Doc_0/Document_0.xml").  Normalize a leading
    // slash ("/Doc_0/Document.xml") and split into the document directory
    // (doc_dir) and the Document file name (document_file).
    let doc_root = doc_root.trim_start_matches('/').to_string();
    let (doc_dir, document_file) = match doc_root.rfind('/') {
        Some(idx) => (doc_root[..idx].to_string(), doc_root[idx + 1..].to_string()),
        None => (String::new(), doc_root),
    };

    // ── Optional metadata (present-but-empty → Some(""), absent → None) ──
    let doc_id = find_optional_text_deep(&root, "DocID");
    let title = find_optional_text_deep(&root, "Title");
    let author = find_optional_text_deep(&root, "Author");
    let creator = find_optional_text_deep(&root, "Creator");
    let creator_version = find_optional_text_deep(&root, "CreatorVersion");
    let mod_date = find_optional_text_deep(&root, "ModDate");
    let creation_date = find_optional_text_deep(&root, "CreationDate");
    let signatures_path = find_optional_text_deep(&root, "Signatures");
    let doc_usage = find_optional_text_deep(&root, "DocUsage");
    let keywords = find_optional_text_deep(&root, "Keywords");

    // ── MaxUnitID ──
    let max_unit_id: u32 = find_text_deep(&root, "MaxUnitID")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    // ── CustomDatas ──
    let custom_datas = find_node_deep(&root, "CustomDatas").and_then(|cds_node| {
        let mut datas = CustomDatas::new();
        for child in &cds_node.children {
            if child.name == "CustomData" {
                let name = child.get_attr("Name").unwrap_or_default().to_string();
                let value = child.text.clone().unwrap_or_default().trim().to_string();
                datas.push(CustomData::new(name, &value));
            }
        }
        if datas.is_empty() { None } else { Some(datas) }
    });

    Ok(OfdEntry {
        doc_dir,
        document_file,
        doc_id,
        title,
        author,
        creator,
        creator_version,
        mod_date,
        creation_date,
        max_unit_id,
        custom_datas,
        signatures_path,
        doc_usage,
        keywords,
    })
}

/// Parsed result from Document.xml.
pub(crate) struct DocumentEntry {
    /// Page BaseLoc paths (e.g. "Pages/Page_0.xml").
    pub(crate) pages: Vec<String>,
    /// Bookmarks (ofd:Bookmarks), if present.
    pub(crate) bookmarks: Option<Bookmarks>,
    /// Outlines (ofd:Outlines), if present.
    pub(crate) outlines: Option<Bookmarks>,
    /// Application area (ofd:ApplicationBox), if present.
    pub(crate) application_box: Option<String>,
    /// Content area (ofd:ContentBox), if present.
    pub(crate) content_box: Option<String>,
    /// Clip area (ofd:ClipBox), if present.
    pub(crate) clip_box: Option<String>,
    /// Bleed area (ofd:BleedBox), if present.
    pub(crate) bleed_box: Option<String>,
    /// Trim area (ofd:TrimBox), if present.
    pub(crate) trim_box: Option<String>,
    /// Template pages (ofd:TemplatePage) declared in CommonData.
    pub(crate) template_pages: Vec<TemplatePage>,
    /// Annotations container path (ofd:Annotations), if present.
    pub(crate) annotations_path: Option<String>,
    /// Attachments container path (ofd:Attachments), if present.
    pub(crate) attachments_path: Option<String>,
    /// Custom tags container path (ofd:CustomTags), if present.
    pub(crate) custom_tags_path: Option<String>,
    /// Whether CommonData declared ofd:PageArea (ofdrw omits it when the
    /// page size is not explicitly configured).
    pub(crate) page_area_present: bool,
    /// DocumentRes reference from CommonData (ofd:DocumentRes text, e.g.
    /// "DocumentRes.xml" or the non-standard "DocumentRes_0.xml").
    pub(crate) document_res: Option<String>,
    /// Whether CommonData declared an ofd:DocumentRes element.
    pub(crate) document_res_element_present: bool,
    /// Document permissions (ofd:Permissions), if present.
    pub(crate) permissions: Option<Permissions>,
    /// Whether CommonData declared an ofd:PublicRes reference.
    pub(crate) public_res_element_present: bool,
}

/// Parse Document.xml → page BaseLoc paths, the bookmark collection
/// (ofd:Bookmarks, located in Document.xml per GB/T 33190-2016 §7.3),
/// and the box elements in CommonData (ofd:ApplicationBox etc.).
///
/// Uses the [`easyofd_core::XmlNode`] tree (via `parse_xml_to_nodes`) instead
/// of a flat quick-xml event stream, so that the parsing is structurally
/// aligned with the rest of the XmlElement framework.
pub(crate) fn parse_document_entry<R: Read + std::io::Seek>(
    archive: &mut zip::ZipArchive<R>,
    doc_dir: &str,
    document_file: &str,
) -> OfdResult<DocumentEntry> {
    let path = doc_path(doc_dir, document_file);
    let xml_bytes = read_zip_entry(archive, &path)?;
    let xml_str = std::str::from_utf8(&xml_bytes)
        .map_err(|e| OfdError::Xml(format!("Document.xml: invalid UTF-8: {e}")))?;

    let root = easyofd_core::parse_xml_to_nodes(xml_str)
        .map_err(|e| OfdError::Xml(format!("Document.xml: {e}")))?;

    // ── Pages ──
    let pages: Vec<String> = root
        .child("Pages")
        .map(|pn| {
            pn.children_named("Page")
                .filter_map(|pe| pe.get_attr("BaseLoc").map(String::from))
                .collect()
        })
        .unwrap_or_default();

    // ── Bookmarks ──
    let bookmarks = root.child("Bookmarks").map(|bn| {
        let mut bms = Bookmarks::new();
        for child in &bn.children {
            if child.name == "Bookmark" {
                let name = child.get_attr("Name").or_else(|| child.get_attr("Title"));
                let goto = child
                    .get_attr("GoTo")
                    .or_else(|| child.child("Dest").and_then(|d| d.get_attr("PageID")));
                if let Some(n) = name {
                    if !n.is_empty() {
                        let mut bm = Bookmark::new(n);
                        if let Some(g) = goto {
                            bm = bm.with_goto(g);
                        }
                        bms.push(bm);
                    }
                }
            }
        }
        bms
    });

    // ── Outlines ──
    let outlines = root.child("Outlines").map(|on| {
        let mut ols = Bookmarks::new();
        for child in &on.children {
            if child.name == "OutlineElem" {
                let title = child.get_attr("Title");
                // Navigate: Actions -> Action -> Goto -> Dest -> PageID
                let goto = child.get_attr("GoTo").or_else(|| {
                    child
                        .child("Actions")
                        .and_then(|a| a.child("Action"))
                        .and_then(|a| a.child("Goto"))
                        .and_then(|g| g.child("Dest"))
                        .and_then(|d| d.get_attr("PageID"))
                });
                if let Some(t) = title {
                    if !t.is_empty() {
                        let mut bm = Bookmark::new(t);
                        if let Some(g) = goto {
                            bm = bm.with_goto(g);
                        }
                        ols.push(bm);
                    }
                }
            }
        }
        ols
    });

    // ── Boxes (search deep in tree to match flat event-stream behaviour) ──
    let application_box = find_text_deep(&root, "ApplicationBox");
    let content_box = find_text_deep(&root, "ContentBox");
    let clip_box = find_text_deep(&root, "ClipBox");
    let bleed_box = find_text_deep(&root, "BleedBox");
    let trim_box = find_text_deep(&root, "TrimBox");

    // ── Template pages (inside CommonData) ──
    let template_pages: Vec<TemplatePage> = find_all_nodes_deep(&root, "TemplatePage")
        .iter()
        .filter_map(|node| {
            let id = node.get_attr("ID").unwrap_or_default().to_string();
            let base_loc = node.get_attr("BaseLoc")?;
            if base_loc.is_empty() {
                None
            } else {
                Some(TemplatePage::new(id, base_loc))
            }
        })
        .collect();

    // ── Container paths ──
    let annotations_path = find_text_deep(&root, "Annotations");
    let attachments_path = find_text_deep(&root, "Attachments");
    let custom_tags_path = find_text_deep(&root, "CustomTags");

    // ── PageArea presence ──
    let page_area_present = find_node_deep(&root, "PageArea").is_some();

    // ── DocumentRes ──
    let document_res_node = find_node_deep(&root, "DocumentRes");
    let document_res_element_present = document_res_node.is_some();
    let document_res = document_res_node
        .and_then(|n| n.text.clone())
        .map(|s| s.trim().to_string());

    // ── PublicRes presence ──
    let public_res_element_present = find_node_deep(&root, "PublicRes").is_some();

    // ── Permissions ──
    let permissions = root.child("Permissions").map(|pn| {
        let mut perms = Permissions::new();
        for child in &pn.children {
            let key = child.name.as_str();
            let value = match key {
                "Print" => child
                    .get_attr("Printable")
                    .and_then(|v| v.parse::<bool>().ok())
                    .unwrap_or(false),
                _ => child
                    .text
                    .as_deref()
                    .unwrap_or("")
                    .trim()
                    .parse::<bool>()
                    .unwrap_or(false),
            };
            match key {
                "Edit" => perms.edit = Some(value),
                "Annot" => perms.annot = Some(value),
                "Export" => perms.export = Some(value),
                "Signature" => perms.signature = Some(value),
                "Watermark" => perms.watermark = Some(value),
                "PrintScreen" => perms.print_screen = Some(value),
                "Print" => perms.print = Some(value),
                "CopyText" => perms.copy_text = Some(value),
                "ContentRegist" => perms.content_regist = Some(value),
                _ => {}
            }
        }
        perms
    });

    Ok(DocumentEntry {
        pages,
        bookmarks,
        outlines,
        application_box,
        content_box,
        clip_box,
        bleed_box,
        trim_box,
        template_pages,
        annotations_path,
        attachments_path,
        custom_tags_path,
        page_area_present,
        document_res,
        document_res_element_present,
        permissions,
        public_res_element_present,
    })
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
    document_res: Option<&str>,
) -> OfdResult<HashMap<String, ResourceEntry>> {
    // The DocumentRes file name comes from the <ofd:DocumentRes> reference in
    // Document.xml (usually "DocumentRes.xml", but non-standard files may use
    // e.g. "DocumentRes_0.xml").
    let path = match document_res {
        Some(res) if !res.is_empty() => doc_path(doc_dir, res),
        _ => format!("{doc_dir}/DocumentRes.xml"),
    };
    let xml = match read_zip_entry(archive, &path) {
        Ok(xml) => xml,
        Err(_) => return Ok(HashMap::new()),
    };
    let mut reader = XmlReader::from_reader(BufReader::new(Cursor::new(&xml)));
    reader.config_mut().trim_text(false);
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
                    let raw_path = raw_path.trim().to_string();
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
///
/// Uses the [`easyofd_core::XmlNode`] tree (via `parse_xml_to_nodes`) instead
/// of a flat quick-xml event stream, so that the parsing is structurally
/// aligned with the rest of the XmlElement framework.
///
/// Entity references inside TextCode / AbbreviatedData are resolved by the
/// tree parser, and text from multiple TextCode children within a single
/// TextObject is concatenated.
pub(crate) fn parse_page_entry<R: Read + std::io::Seek>(
    archive: &mut zip::ZipArchive<R>,
    page_path: &str,
    doc_dir: &str,
    resources: &HashMap<String, ResourceEntry>,
) -> OfdResult<OfdPage> {
    let xml_bytes = read_zip_entry(archive, page_path)?;
    let xml_str = std::str::from_utf8(&xml_bytes)
        .map_err(|e| OfdError::Xml(format!("{page_path}: invalid UTF-8: {e}")))?;

    let root = easyofd_core::parse_xml_to_nodes(xml_str)
        .map_err(|e| OfdError::Xml(format!("{page_path}: {e}")))?;

    // ── PhysicalBox → page dimensions ──
    let (width, height) = find_node_deep(&root, "PhysicalBox")
        .and_then(|n| n.text.clone())
        .map_or((210.0, 297.0), |s| parse_physical_box_dims(&s));

    // ── Content objects (document-order walk) ──
    let content = collect_content_objects(&root, archive, doc_dir, resources)?;

    Ok(OfdPage {
        width,
        height,
        content,
        base_path: Some(page_path.to_string()),
    })
}

/// Walk the XmlNode tree depth-first in document order, collecting content
/// objects (TextObject, PathObject, ImageObject).
fn collect_content_objects<R: Read + std::io::Seek>(
    node: &easyofd_core::XmlNode,
    archive: &mut zip::ZipArchive<R>,
    doc_dir: &str,
    resources: &HashMap<String, ResourceEntry>,
) -> OfdResult<Vec<ContentObject>> {
    let mut result = Vec::new();
    for child in &node.children {
        match child.name.as_str() {
            "TextObject" => {
                let obj = build_text_object_from_node(child);
                result.push(ContentObject::Text(obj));
            }
            "PathObject" => {
                let obj = build_path_object_from_node(child);
                result.push(ContentObject::Path(obj));
            }
            "ImageObject" => {
                if let Some(img) = build_image_object_from_node(child, archive, doc_dir, resources)?
                {
                    result.push(ContentObject::Image(img));
                }
                // Resource missing (broken reference in non-standard
                // samples): drop the image object so the writer does not
                // emit an empty resource file.
            }
            _ => {
                result.extend(collect_content_objects(child, archive, doc_dir, resources)?);
            }
        }
    }
    Ok(result)
}

/// Build a [`TextObject`] from an [`XmlNode`], extracting Boundary/Font/Size
/// attributes and concatenating text from all TextCode children.
#[allow(clippy::many_single_char_names)]
fn build_text_object_from_node(node: &easyofd_core::XmlNode) -> TextObject {
    let mut x = 0.0_f64;
    let mut y = 0.0_f64;
    let mut font = None;
    let mut size = None;
    let mut width = None;
    let mut height = None;

    for (key, value) in &node.attrs {
        match key.as_str() {
            "Boundary" => {
                let parts: Vec<f64> = value
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
            "Font" => font = Some(value.clone()),
            "Size" => size = value.parse().ok(),
            _ => {}
        }
    }

    // Concatenate text from all TextCode children (document order).
    // Entity references are already resolved by `parse_xml_to_nodes`.
    let text: String = node
        .children
        .iter()
        .filter(|c| c.name == "TextCode")
        .filter_map(|c| c.text.clone())
        .collect();

    let mut obj = TextObject::new(x, y, text);
    if let Some(f) = font {
        obj = obj.font(f);
    }
    if let Some(s) = size {
        obj = obj.size(s);
    }
    obj.width = width;
    obj.height = height;
    obj
}

/// Build a [`PathObject`] from an [`XmlNode`], extracting stroke/fill
/// attributes and AbbreviatedData text.
fn build_path_object_from_node(node: &easyofd_core::XmlNode) -> PathObject {
    let mut x = 0.0_f64;
    let mut y = 0.0_f64;
    let mut stroke_color = 0u32;
    let mut stroke_width = 0.35_f64;
    let mut fill_color = None;

    for (key, value) in &node.attrs {
        match key.as_str() {
            "Boundary" => {
                let parts: Vec<f64> = value
                    .split_whitespace()
                    .filter_map(|s| s.parse().ok())
                    .collect();
                if parts.len() >= 2 {
                    x = parts[0];
                    y = parts[1];
                }
            }
            "StrokeColor" => stroke_color = parse_hex_color(value).unwrap_or(0),
            "FillColor" => fill_color = parse_hex_color(value),
            "LineWidth" => stroke_width = value.parse().unwrap_or(0.35),
            _ => {}
        }
    }

    let path_data = node
        .children
        .iter()
        .find(|c| c.name == "AbbreviatedData")
        .and_then(|c| c.text.clone())
        .unwrap_or_default();

    let mut object = PathObject::new(x, y, path_data)
        .stroke_color(stroke_color)
        .stroke_width(stroke_width);
    if let Some(fc) = fill_color {
        object = object.fill_color(fc);
    }
    object
}

/// Build an [`ImageObject`] from an [`XmlNode`], looking up the resource
/// by ResourceID and reading image data from the archive.
#[allow(clippy::many_single_char_names)]
fn build_image_object_from_node<R: Read + std::io::Seek>(
    node: &easyofd_core::XmlNode,
    archive: &mut zip::ZipArchive<R>,
    doc_dir: &str,
    resources: &HashMap<String, ResourceEntry>,
) -> OfdResult<Option<ImageObject>> {
    let mut x = 0.0_f64;
    let mut y = 0.0_f64;
    let mut w = 0.0_f64;
    let mut h = 0.0_f64;
    let mut resource_id = String::new();

    for (key, value) in &node.attrs {
        match key.as_str() {
            "Boundary" => {
                let parts: Vec<f64> = value
                    .split_whitespace()
                    .filter_map(|s| s.parse().ok())
                    .collect();
                if parts.len() >= 4 {
                    x = parts[0];
                    y = parts[1];
                    w = parts[2];
                    h = parts[3];
                }
            }
            "ResourceID" => resource_id.clone_from(value),
            _ => {}
        }
    }

    if let Some(resource) = resources.get(&resource_id) {
        let resource_path = resolve_resource_path(doc_dir, &resource.location)?;
        // Keep the actual archive entry name (case may differ,
        // e.g. "DOC_0/Res/Image_4.JPEG") so a roundtrip writes
        // the resource under the same name as the source.
        let actual_name = find_zip_entry_name(archive, &resource_path).unwrap_or(resource_path);
        let data = read_zip_entry(archive, &actual_name)?;
        Ok(Some(
            ImageObject::new(x, y, w, h, data, resource.format).with_res_name(actual_name),
        ))
    } else {
        Ok(None)
    }
}

/// Parse "x y w h" PhysicalBox text into `(width, height)`.
fn parse_physical_box_dims(text: &str) -> (f64, f64) {
    let parts: Vec<f64> = text
        .split_whitespace()
        .filter_map(|s| s.parse().ok())
        .collect();
    if parts.len() >= 4 {
        (parts[2], parts[3])
    } else {
        (210.0, 297.0)
    }
}

fn parse_hex_color(value: &str) -> Option<u32> {
    u32::from_str_radix(value.trim_start_matches('#'), 16).ok()
}

// ─── ZIP Helper ──────────────────────────────────────────────────────────────

/// 在归档中定位条目，返回条目在归档中的实际名称。
///
/// 先尝试精确匹配；失败时做大小写不敏感扫描——部分 ofdrw 样本把资源放在
/// 不同大小写的目录下（如 `DOC_0/Res/Image_4.JPEG`，而 DocRoot 使用
/// `Doc_0`）。ZIP 条目查找本身是大小写敏感的，因此按忽略大小写扫描。
fn find_zip_entry_name<R: Read + std::io::Seek>(
    archive: &mut zip::ZipArchive<R>,
    name: &str,
) -> Option<String> {
    if archive.by_name(name).is_ok() {
        return Some(name.to_string());
    }
    let lower = name.to_lowercase();
    for i in 0..archive.len() {
        if archive.by_index(i).ok()?.name().to_lowercase() == lower {
            let actual = archive.by_index(i).ok()?.name().to_string();
            return Some(actual);
        }
    }
    None
}

pub(crate) fn read_zip_entry<R: Read + std::io::Seek>(
    archive: &mut zip::ZipArchive<R>,
    name: &str,
) -> OfdResult<Vec<u8>> {
    let actual = find_zip_entry_name(archive, name)
        .ok_or_else(|| OfdError::Zip(format!("{name}: specified file not found in archive")))?;
    let mut file = archive
        .by_name(&actual)
        .map_err(|e| OfdError::Zip(format!("{actual}: {e}")))?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).map_err(OfdError::Io)?;
    Ok(buf)
}

/// Join the document directory and a file name into an archive path.
///
/// Tolerates absolute file names: a leading "/" is stripped, and a file name
/// that already starts with `doc_dir` (case-insensitively, e.g.
/// "/Doc_0/Pages/Page_0/Content.xml") is not prefixed again.
pub(crate) fn doc_path(doc_dir: &str, file_name: &str) -> String {
    let file_name = file_name.trim_start_matches('/');
    if doc_dir.is_empty() {
        return file_name.to_string();
    }
    let head = &file_name[..file_name.len().min(doc_dir.len())];
    if head.eq_ignore_ascii_case(doc_dir) && file_name.as_bytes().get(doc_dir.len()) == Some(&b'/')
    {
        format!("{doc_dir}{}", &file_name[doc_dir.len()..])
    } else {
        format!("{doc_dir}/{file_name}")
    }
}

pub(crate) fn resolve_resource_path(doc_dir: &str, location: &str) -> OfdResult<String> {
    let location = location.trim_start_matches('/');
    // The location may already start with the doc directory (case may differ,
    // e.g. "DOC_0/Res/Image_4.JPEG" from a non-standard source); keep its
    // original spelling instead of re-prefixing.
    let doc_dir_prefix = format!("{doc_dir}/");
    let path = if location
        .get(..doc_dir_prefix.len())
        .is_some_and(|head| head.eq_ignore_ascii_case(&doc_dir_prefix))
    {
        location.to_string()
    } else {
        format!("{doc_dir}/{location}")
    };
    easyofd_package::validate_entry_name(&path)?;
    Ok(path)
}

// ─── XmlNode tree helpers for Document.xml parsing ───────────────────────────

/// Find a node by name anywhere in the tree (depth-first, first match).
fn find_node_deep<'a>(
    node: &'a easyofd_core::XmlNode,
    name: &str,
) -> Option<&'a easyofd_core::XmlNode> {
    if node.name == name {
        return Some(node);
    }
    for child in &node.children {
        if let Some(found) = find_node_deep(child, name) {
            return Some(found);
        }
    }
    None
}

/// Find a node by name and return its text content (trimmed).
fn find_text_deep(node: &easyofd_core::XmlNode, name: &str) -> Option<String> {
    find_node_deep(node, name)
        .and_then(|n| n.text.clone())
        .map(|s| s.trim().to_string())
}

/// Find a node by name; if present return its text (trimmed, defaulting to ""
/// for empty/self-closing elements); if absent return `None`.
///
/// This distinguishes "element present but empty" (`Some("")`) from "element
/// not present" (`None`), matching the event-stream parser behaviour for
/// self-closing metadata elements like `<ofd:DocID/>`.
fn find_optional_text_deep(node: &easyofd_core::XmlNode, name: &str) -> Option<String> {
    find_node_deep(node, name).map(|n| n.text.clone().unwrap_or_default().trim().to_string())
}

/// Find all nodes with a given name (depth-first).
fn find_all_nodes_deep<'a>(
    node: &'a easyofd_core::XmlNode,
    name: &str,
) -> Vec<&'a easyofd_core::XmlNode> {
    let mut result = Vec::new();
    collect_nodes_deep(node, name, &mut result);
    result
}

fn collect_nodes_deep<'a>(
    node: &'a easyofd_core::XmlNode,
    name: &str,
    result: &mut Vec<&'a easyofd_core::XmlNode>,
) {
    if node.name == name {
        result.push(node);
    }
    for child in &node.children {
        collect_nodes_deep(child, name, result);
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────
