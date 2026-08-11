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

pub(crate) fn parse_ofd_entry<R: Read + std::io::Seek>(
    archive: &mut zip::ZipArchive<R>,
) -> OfdResult<OfdEntry> {
    let xml = read_zip_entry(archive, "OFD.xml")?;
    let mut reader = XmlReader::from_reader(BufReader::new(Cursor::new(&xml)));
    reader.config_mut().trim_text(false);
    let mut buf = Vec::new();
    let mut doc_root = String::new();
    let mut doc_id = None;
    let mut title = None;
    let mut author = None;
    let mut creator = None;
    let mut creator_version = None;
    let mut mod_date = None;
    let mut creation_date = None;
    let mut max_unit_id = 0_u32;
    let mut custom_datas: Option<CustomDatas> = None;
    let mut signatures_path: Option<String> = None;
    let mut doc_usage: Option<String> = None;
    let mut keywords: Option<String> = None;
    let mut current_text_tag: Option<Vec<u8>> = None;
    let mut current_text = String::new();
    // Nested element tracking for CustomDatas
    let mut in_custom_datas = false;
    let mut in_custom_data = false;
    let mut current_custom_data_name: Option<String> = None;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Empty(ref e)) => match e.name().as_ref() {
                // Self-closing metadata elements (e.g. <ofd:DocID/>) carry an
                // empty value; record them so a roundtrip keeps the element.
                b"ofd:DocID" => doc_id = Some(String::new()),
                b"ofd:Title" => title = Some(String::new()),
                b"ofd:Author" => author = Some(String::new()),
                b"ofd:Creator" => creator = Some(String::new()),
                b"ofd:CreatorVersion" => creator_version = Some(String::new()),
                b"ofd:ModDate" => mod_date = Some(String::new()),
                b"ofd:CreationDate" => creation_date = Some(String::new()),
                b"ofd:Signatures" => signatures_path = Some(String::new()),
                b"ofd:DocUsage" => doc_usage = Some(String::new()),
                b"ofd:Keywords" => keywords = Some(String::new()),
                b"ofd:CustomData" if in_custom_datas => {
                    let mut name = String::new();
                    for attr in e.attributes().flatten() {
                        if attr.key.as_ref() == b"Name" {
                            name = attr
                                .decoded_and_normalized_value(
                                    quick_xml::XmlVersion::Explicit1_0,
                                    reader.decoder(),
                                )
                                .unwrap_or_default()
                                .to_string();
                        }
                    }
                    let datas = custom_datas.get_or_insert_with(CustomDatas::new);
                    datas.push(CustomData::new(name, ""));
                }
                _ => {}
            },
            Ok(Event::Start(ref e)) => match e.name().as_ref() {
                b"ofd:DocRoot" => {
                    current_text_tag = Some(b"ofd:DocRoot".to_vec());
                    current_text.clear();
                }
                b"ofd:DocID" => {
                    current_text_tag = Some(b"ofd:DocID".to_vec());
                    current_text.clear();
                }
                b"ofd:Title" => {
                    current_text_tag = Some(b"ofd:Title".to_vec());
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
                b"ofd:Signatures" => {
                    current_text_tag = Some(b"ofd:Signatures".to_vec());
                    current_text.clear();
                }
                b"ofd:DocUsage" => {
                    current_text_tag = Some(b"ofd:DocUsage".to_vec());
                    current_text.clear();
                }
                b"ofd:Keywords" => {
                    current_text_tag = Some(b"ofd:Keywords".to_vec());
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
                _ => {}
            },
            Ok(Event::Text(ref e)) => {
                let text = e
                    .xml10_content()
                    .map(|c| c.into_owned())
                    .unwrap_or_default();
                // Append, not assign: quick_xml may split a text node into
                // multiple Text events around entities (e.g. "&apos;" splits
                // "D:2022...+02'34'" into "D:2022...+02" + "34").
                if current_text_tag.is_some() || in_custom_data {
                    current_text.push_str(&text);
                }
            }
            Ok(Event::End(ref end)) => {
                let tag_name = end.name();
                match tag_name.as_ref() {
                    b"ofd:DocRoot" => {
                        doc_root = current_text.trim().to_string();
                        current_text_tag = None;
                    }
                    b"ofd:DocID" => {
                        doc_id = Some(current_text.trim().to_string());
                        current_text_tag = None;
                    }
                    b"ofd:Title" => {
                        title = Some(current_text.trim().to_string());
                        current_text_tag = None;
                    }
                    b"ofd:Author" => {
                        author = Some(current_text.trim().to_string());
                        current_text_tag = None;
                    }
                    b"ofd:Creator" => {
                        creator = Some(current_text.trim().to_string());
                        current_text_tag = None;
                    }
                    b"ofd:CreatorVersion" => {
                        creator_version = Some(current_text.trim().to_string());
                        current_text_tag = None;
                    }
                    b"ofd:ModDate" => {
                        mod_date = Some(current_text.trim().to_string());
                        current_text_tag = None;
                    }
                    b"ofd:CreationDate" => {
                        creation_date = Some(current_text.trim().to_string());
                        current_text_tag = None;
                    }
                    b"ofd:MaxUnitID" => {
                        max_unit_id = current_text.trim().parse().unwrap_or(0);
                        current_text_tag = None;
                    }
                    b"ofd:Signatures" => {
                        signatures_path = Some(current_text.trim().to_string());
                        current_text_tag = None;
                    }
                    b"ofd:DocUsage" => {
                        doc_usage = Some(current_text.trim().to_string());
                        current_text_tag = None;
                    }
                    b"ofd:Keywords" => {
                        keywords = Some(current_text.trim().to_string());
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

    // DocRoot points at the Document XML file directly (e.g.
    // "Doc_0/Document.xml" or "Doc_0/Document_0.xml").  Normalize a leading
    // slash ("/Doc_0/Document.xml") and split into the document directory
    // (doc_dir) and the Document file name (document_file).
    let doc_root = doc_root.trim_start_matches('/').to_string();
    let (doc_dir, document_file) = match doc_root.rfind('/') {
        Some(idx) => (doc_root[..idx].to_string(), doc_root[idx + 1..].to_string()),
        None => (String::new(), doc_root),
    };

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
pub(crate) fn parse_page_entry<R: Read + std::io::Seek>(
    archive: &mut zip::ZipArchive<R>,
    page_path: &str,
    doc_dir: &str,
    resources: &HashMap<String, ResourceEntry>,
) -> OfdResult<OfdPage> {
    let xml = read_zip_entry(archive, page_path)?;
    let mut reader = XmlReader::from_reader(BufReader::new(Cursor::new(&xml)));
    reader.config_mut().trim_text(false);
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
                    if let Some(resource) = resources.get(&img.resource_id) {
                        let resource_path = resolve_resource_path(doc_dir, &resource.location)?;
                        // Keep the actual archive entry name (case may differ,
                        // e.g. "DOC_0/Res/Image_4.JPEG") so a roundtrip writes
                        // the resource under the same name as the source.
                        let actual_name =
                            find_zip_entry_name(archive, &resource_path).unwrap_or(resource_path);
                        let data = read_zip_entry(archive, &actual_name)?;
                        content.push(ContentObject::Image(
                            ImageObject::new(
                                img.x,
                                img.y,
                                img.width,
                                img.height,
                                data,
                                resource.format,
                            )
                            .with_res_name(actual_name),
                        ));
                    }
                    // Resource missing (broken reference in non-standard
                    // samples): drop the image object so the writer does not
                    // emit an empty resource file.
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
        base_path: Some(page_path.to_string()),
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
        resource_id,
    })
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
