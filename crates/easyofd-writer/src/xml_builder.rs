//! OFD XML 文档生成。
//!
//! 包含 OfdWriter 的 XML 构建方法，用于生成 GB/T 33190-2016 标准的 XML 文件。

use crate::OfdWriter;
use easyofd_core::xml_element::XmlNode;
use easyofd_core::{ContentObject, ImageFormat, OfdPage};

/// Format a number as integer when it has no fractional part, otherwise as float.
/// This matches ofdrw's convention: `210` not `210.00`, `3.175` stays as `3.175`.
#[allow(clippy::cast_possible_truncation)]
fn format_number(val: f64) -> String {
    if (val - val.round()).abs() < f64::EPSILON
        && val >= f64::from(i32::MIN)
        && val <= f64::from(i32::MAX)
    {
        format!("{}", val as i64)
    } else {
        format!("{val}")
    }
}

/// Strip the doc directory prefix from an archive path, ignoring case
/// (a roundtrip resource may keep its original spelling, e.g.
/// "DOC_0/Res/Image_4.JPEG" while the doc directory is "Doc_0").
fn strip_doc_dir_prefix<'a>(res_name: &'a str, doc_dir: &str) -> &'a str {
    let prefix = format!("{doc_dir}/");
    if res_name
        .get(..prefix.len())
        .is_some_and(|head| head.eq_ignore_ascii_case(&prefix))
    {
        &res_name[prefix.len()..]
    } else {
        res_name
    }
}

impl OfdWriter {
    pub(crate) fn build_ofd_xml(&self) -> String {
        // Helper: create a text child XmlNode with ofd: prefix.
        fn ofd_text(name: &str, text: &str) -> XmlNode {
            let mut node = XmlNode::element(format!("ofd:{name}"));
            node.push_child(XmlNode::text_node(text));
            node
        }

        // ── Build the entire OFD.xml as a single XmlNode tree ──
        let mut ofd = XmlNode::element("ofd:OFD")
            .attr("xmlns:ofd", "http://www.ofdspec.org/2016")
            .attr("Version", &self.options.metadata.version)
            .attr("DocType", "OFD");

        // ── DocBody ──
        let mut doc_body = XmlNode::element("ofd:DocBody");

        // ── DocInfo ──
        let mut doc_info = XmlNode::element("ofd:DocInfo");

        if let Some(ref doc_id) = self.options.metadata.doc_id {
            doc_info.push_child(ofd_text("DocID", doc_id));
        }
        if let Some(ref title) = self.options.metadata.title {
            doc_info.push_child(ofd_text("Title", title));
        }
        if let Some(ref author) = self.options.metadata.author {
            doc_info.push_child(ofd_text("Author", author));
        }
        if let Some(ref creator) = self.options.metadata.creator {
            doc_info.push_child(ofd_text("Creator", creator));
        }
        if let Some(ref creator_version) = self.options.metadata.creator_version {
            doc_info.push_child(ofd_text("CreatorVersion", creator_version));
        }
        if let Some(dt) = self.options.metadata.creation_date {
            let mut node = XmlNode::element("ofd:CreationDate");
            node.push_child(XmlNode::text_node(
                dt.format("%Y-%m-%dT%H:%M:%S").to_string(),
            ));
            doc_info.push_child(node);
        }
        if let Some(dt) = self.options.metadata.mod_date {
            let mut node = XmlNode::element("ofd:ModDate");
            node.push_child(XmlNode::text_node(
                dt.format("%Y-%m-%dT%H:%M:%S").to_string(),
            ));
            doc_info.push_child(node);
        }
        if let Some(ref doc_usage) = self.options.metadata.doc_usage {
            doc_info.push_child(ofd_text("DocUsage", doc_usage));
        }
        if let Some(ref keywords) = self.options.metadata.keywords {
            doc_info.push_child(ofd_text("Keywords", keywords));
        }

        // CustomDatas
        if let Some(ref custom_datas) = self.options.metadata.custom_datas {
            if !custom_datas.is_empty() {
                let mut custom_datas_node = XmlNode::element("ofd:CustomDatas");
                for item in &custom_datas.items {
                    let mut cd_node = XmlNode::element("ofd:CustomData").attr("Name", &item.name);
                    cd_node.push_child(XmlNode::text_node(&item.value));
                    custom_datas_node.push_child(cd_node);
                }
                doc_info.push_child(custom_datas_node);
            }
        }

        doc_body.push_child(doc_info);

        // DocRoot
        doc_body.push_child(ofd_text(
            "DocRoot",
            &format!(
                "{doc_dir}/{doc_file}",
                doc_dir = self.options.metadata.doc_dir,
                doc_file = self.options.metadata.document_file,
            ),
        ));

        // Signatures container reference (ofdrw writes it inside DocBody,
        // after DocRoot, pointing at the signatures list).
        if let Some(ref signatures_path) = self.options.metadata.signatures_path {
            doc_body.push_child(ofd_text("Signatures", signatures_path));
        }

        ofd.push_child(doc_body);

        // ── Serialize the complete tree ──
        let mut xml = String::with_capacity(512);
        xml.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
        xml.push('\n');
        xml.push_str(&ofd.to_xml_string());
        xml.push('\n');
        xml
    }

    pub(crate) fn build_document_xml(
        &self,
        image_resources: &[(String, &[u8], ImageFormat)],
    ) -> String {
        use easyofd_core::doc::pages::{PageEntry, Pages};
        use easyofd_core::xml_element::{XmlElement, XmlNode};
        use easyofd_core::{CT_TemplatePage, CTDest, DestType};

        // Helper: create a text child XmlNode with ofd: prefix.
        fn ofd_text(name: &str, text: &str) -> XmlNode {
            let mut node = XmlNode::element(format!("ofd:{name}"));
            node.push_child(XmlNode::text_node(text));
            node
        }

        // ── Build the entire Document as a single XmlNode tree ──
        // Using XmlNode (core XmlElement infrastructure) with ofd: prefix
        // to maintain structural consistency with ofdrw output.
        // XmlElement types (Pages, PageEntry, CT_TemplatePage, CTDest) are
        // used for data extraction where their to_xml() structure matches.

        let mut doc =
            XmlNode::element("ofd:Document").attr("xmlns:ofd", "http://www.ofdspec.org/2016");

        // ── CommonData ──
        // NOTE: CT_CommonData / CT_PageArea XmlElement types put PhysicalBox
        // as an *attribute*, while the current output (and ofdrw) uses a
        // *child element*. Using those types would change element counts in
        // the roundtrip_diff test, so CommonData is built as a XmlNode tree.
        let mut common_data = XmlNode::element("ofd:CommonData");

        // MaxUnitID
        let max_unit_id =
            self.pages.len() + self.pages.iter().map(|p| p.content.len()).sum::<usize>();
        common_data.push_child(ofd_text("MaxUnitID", &max_unit_id.to_string()));

        // PageArea with PhysicalBox as a child element (matching ofdrw output).
        // CT_PageArea XmlElement puts PhysicalBox as an attribute — different
        // element count — so we build the XmlNode manually here.
        if self.options.metadata.page_area_present {
            let (pw, ph) = self
                .pages
                .first()
                .map_or((210.0, 297.0), |p| (p.width, p.height));
            let width_str = format_number(pw);
            let height_str = format_number(ph);
            let mut page_area = XmlNode::element("ofd:PageArea");
            page_area.push_child(ofd_text(
                "PhysicalBox",
                &format!("0 0 {width_str} {height_str}"),
            ));
            common_data.push_child(page_area);
        }

        // Optional box elements (siblings of PageArea inside CommonData).
        if let Some(ref v) = self.options.metadata.application_box {
            common_data.push_child(ofd_text("ApplicationBox", v));
        }
        if let Some(ref v) = self.options.metadata.content_box {
            common_data.push_child(ofd_text("ContentBox", v));
        }
        if let Some(ref v) = self.options.metadata.clip_box {
            common_data.push_child(ofd_text("ClipBox", v));
        }
        if let Some(ref v) = self.options.metadata.bleed_box {
            common_data.push_child(ofd_text("BleedBox", v));
        }
        if let Some(ref v) = self.options.metadata.trim_box {
            common_data.push_child(ofd_text("TrimBox", v));
        }

        // PublicRes (font declarations — only when source had one).
        if self.options.metadata.public_res_element_present {
            common_data.push_child(ofd_text("PublicRes", "PublicRes.xml"));
        }

        // DocumentRes (image/media resources).
        let doc_res_ref = self
            .options
            .metadata
            .document_res
            .as_deref()
            .filter(|s| !s.is_empty())
            .unwrap_or("DocumentRes.xml");
        if self.options.metadata.document_res_element_present
            && (!image_resources.is_empty() || self.options.metadata.document_res.is_some())
        {
            common_data.push_child(ofd_text("DocumentRes", doc_res_ref));
        }

        // TemplatePage entries — use CT_TemplatePage XmlElement for attribute data.
        for tpl in &self.options.metadata.template_pages {
            let tpl_id: u32 = tpl.id.parse().unwrap_or(0);
            let ct_tpl = CT_TemplatePage::new(tpl_id).base_loc(&tpl.base_loc);
            // Convert XmlElement attributes to ofd:-prefixed XmlNode.
            let mut tpl_node = XmlNode::element("ofd:TemplatePage");
            for (k, v) in ct_tpl.attributes() {
                tpl_node.attrs.push((k, v));
            }
            common_data.push_child(tpl_node);
        }

        doc.push_child(common_data);

        // ── Pages — use Pages/PageEntry XmlElement types for data ──
        let mut pages_tree = Pages::new();
        let doc_dir_prefix = format!("{}/", self.options.metadata.doc_dir);
        for (i, page) in self.pages.iter().enumerate() {
            let base_loc = page.base_path.as_deref().map_or_else(
                || format!("Pages/Page_{i}/Content.xml"),
                |p| {
                    p.trim_start_matches('/')
                        .strip_prefix(&doc_dir_prefix)
                        .unwrap_or_else(|| p.trim_start_matches('/'))
                        .to_string()
                },
            );
            pages_tree.add_page(PageEntry::new(u32::try_from(i + 1).unwrap_or(1), base_loc));
        }
        // Convert Pages/PageEntry XmlElement attributes to ofd:-prefixed XmlNodes.
        let mut pages_node = XmlNode::element("ofd:Pages");
        for pe in &pages_tree.pages {
            let mut entry_node = XmlNode::element("ofd:Page");
            for (k, v) in pe.attributes() {
                entry_node.attrs.push((k, v));
            }
            pages_node.push_child(entry_node);
        }
        doc.push_child(pages_node);

        // ── Bookmarks ──
        // Bookmark/Dest elements are emitted without ofd: prefix (matching
        // ofdrw output). CTDest XmlElement is used for Dest attribute data.
        if let Some(ref bookmarks) = self.options.metadata.bookmarks {
            if !bookmarks.is_empty() {
                let mut bm_node = XmlNode::element("ofd:Bookmarks");
                for bm in &bookmarks.items {
                    let mut item = XmlNode::element("Bookmark").attr("Name", &bm.name);
                    if let Some(ref target) = bm.goto_target {
                        let page_id: u32 = target.parse().unwrap_or(0);
                        let dest = CTDest::new(page_id).dest_type(DestType::XYZ);
                        let mut dest_node = XmlNode::element("Dest");
                        for (k, v) in dest.attributes() {
                            dest_node.attrs.push((k, v));
                        }
                        item.push_child(dest_node);
                    }
                    bm_node.push_child(item);
                }
                doc.push_child(bm_node);
            }
        }

        // ── Outlines ──
        // CT_OutlineElem XmlElement doesn't match the Actions/Action/Goto/Dest
        // nesting that ofdrw produces, so outlines are built as XmlNodes.
        // CTDest XmlElement is used for Dest attribute data.
        if let Some(ref outlines) = self.options.metadata.outlines {
            if outlines.is_empty() {
                doc.push_child(XmlNode::element("ofd:Outlines"));
            } else {
                let mut ol_node = XmlNode::element("ofd:Outlines");
                for item in &outlines.items {
                    let mut elem = XmlNode::element("ofd:OutlineElem")
                        .attr("Title", &item.name)
                        .attr("Expanded", "true");
                    let mut actions = XmlNode::element("ofd:Actions");
                    let mut action = XmlNode::element("ofd:Action").attr("Event", "CLICK");
                    let mut goto = XmlNode::element("ofd:Goto");
                    if let Some(ref target) = item.goto_target {
                        let page_id: u32 = target.parse().unwrap_or(0);
                        let dest = CTDest::new(page_id).dest_type(DestType::XYZ);
                        let mut dest_node = XmlNode::element("ofd:Dest");
                        for (k, v) in dest.attributes() {
                            dest_node.attrs.push((k, v));
                        }
                        goto.push_child(dest_node);
                    }
                    action.push_child(goto);
                    actions.push_child(action);
                    elem.push_child(actions);
                    ol_node.push_child(elem);
                }
                doc.push_child(ol_node);
            }
        }

        // ── Container references ──
        if let Some(ref p) = self.options.metadata.annotations_path {
            doc.push_child(ofd_text("Annotations", p));
        }
        if let Some(ref p) = self.options.metadata.attachments_path {
            doc.push_child(ofd_text("Attachments", p));
        }
        if let Some(ref p) = self.options.metadata.custom_tags_path {
            doc.push_child(ofd_text("CustomTags", p));
        }

        // ── Permissions ──
        if let Some(ref perms) = self.options.metadata.permissions {
            let mut perms_node = XmlNode::element("ofd:Permissions");
            let add_bool = |node: &mut XmlNode, tag: &str, value: Option<bool>| {
                if let Some(v) = value {
                    let mut child = XmlNode::element(format!("ofd:{tag}"));
                    child.push_child(XmlNode::text_node(v.to_string()));
                    node.push_child(child);
                }
            };
            add_bool(&mut perms_node, "Edit", perms.edit);
            add_bool(&mut perms_node, "Annot", perms.annot);
            add_bool(&mut perms_node, "Export", perms.export);
            add_bool(&mut perms_node, "Signature", perms.signature);
            add_bool(&mut perms_node, "Watermark", perms.watermark);
            add_bool(&mut perms_node, "PrintScreen", perms.print_screen);
            if let Some(v) = perms.print {
                perms_node
                    .push_child(XmlNode::element("ofd:Print").attr("Printable", v.to_string()));
            }
            add_bool(&mut perms_node, "CopyText", perms.copy_text);
            add_bool(&mut perms_node, "ContentRegist", perms.content_regist);
            doc.push_child(perms_node);
        }

        // ── Serialize the complete tree ──
        let mut xml = String::with_capacity(2048);
        xml.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
        xml.push('\n');
        xml.push_str(&doc.to_xml_string());
        xml.push('\n');
        xml
    }

    #[allow(clippy::unused_self)]
    pub(crate) fn build_document_res_xml(
        &self,
        image_resources: &[(String, &[u8], ImageFormat)],
    ) -> String {
        // ── Build DocumentRes as a XmlNode tree ──
        let mut doc_res =
            XmlNode::element("ofd:DocumentRes").attr("xmlns:ofd", "http://www.ofdspec.org/2016");

        let mut multi_medias = XmlNode::element("ofd:MultiMedias");

        for (i, (res_name, _, fmt)) in image_resources.iter().enumerate() {
            let type_str = match fmt {
                ImageFormat::Jpeg => "JPEG",
                ImageFormat::Png => "PNG",
                ImageFormat::Bmp => "BMP",
                ImageFormat::Tiff => "TIFF",
            };
            // The BaseLoc is relative to the doc directory.
            let relative = strip_doc_dir_prefix(res_name, &self.options.metadata.doc_dir);
            let mut media_node = XmlNode::element("ofd:MultiMedia")
                .attr("ID", (100 + i).to_string())
                .attr("Type", type_str);
            let mut media_file = XmlNode::element("ofd:MediaFile");
            media_file.push_child(XmlNode::text_node(relative));
            media_node.push_child(media_file);
            multi_medias.push_child(media_node);
        }

        doc_res.push_child(multi_medias);

        // ── Serialize the complete tree ──
        let mut xml = String::with_capacity(512);
        xml.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
        xml.push('\n');
        xml.push_str(&doc_res.to_xml_string());
        xml.push('\n');
        xml
    }

    pub(crate) fn build_public_res_xml(
        &self,
        image_resources: &[(String, &[u8], ImageFormat)],
    ) -> String {
        // Collect unique font names from all text objects across pages.
        let mut fonts: Vec<String> = Vec::new();
        for page in &self.pages {
            for obj in &page.content {
                if let ContentObject::Text(text) = obj {
                    let name = text.font.clone();
                    if !fonts.contains(&name) {
                        fonts.push(name);
                    }
                }
            }
        }

        // ── Build Res as a XmlNode tree ──
        let mut res = XmlNode::element("ofd:Res")
            .attr("xmlns:ofd", "http://www.ofdspec.org/2016")
            .attr("BaseLoc", "Res");

        // Fonts container (always present, even if empty)
        let mut fonts_node = XmlNode::element("ofd:Fonts");
        for (i, font_name) in fonts.iter().enumerate() {
            fonts_node.push_child(
                XmlNode::element("ofd:Font")
                    .attr("ID", (400 + i).to_string())
                    .attr("FontName", font_name),
            );
        }
        res.push_child(fonts_node);

        // MultiMedias container for image resources
        if !image_resources.is_empty() {
            let mut multi_medias = XmlNode::element("ofd:MultiMedias");
            for (i, (res_name, _, fmt)) in image_resources.iter().enumerate() {
                let type_str = match fmt {
                    ImageFormat::Jpeg => "JPEG",
                    ImageFormat::Png => "PNG",
                    ImageFormat::Bmp => "BMP",
                    ImageFormat::Tiff => "TIFF",
                };
                let relative = strip_doc_dir_prefix(res_name, &self.options.metadata.doc_dir);
                let mut media_node = XmlNode::element("ofd:MultiMedia")
                    .attr("ID", (100 + i).to_string())
                    .attr("Type", type_str);
                let mut media_file = XmlNode::element("ofd:MediaFile");
                media_file.push_child(XmlNode::text_node(relative));
                media_node.push_child(media_file);
                multi_medias.push_child(media_node);
            }
            res.push_child(multi_medias);
        }

        // ── Serialize the complete tree ──
        let mut xml = String::with_capacity(256);
        xml.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
        xml.push('\n');
        xml.push_str(&res.to_xml_string());
        xml.push('\n');
        xml
    }

    #[allow(clippy::unused_self)]
    pub(crate) fn build_page_xml(
        &self,
        page: &OfdPage,
        page_index: usize,
        page_image_start: usize,
    ) -> String {
        // ── Build Page as a XmlNode tree ──
        let mut page_node =
            XmlNode::element("ofd:Page").attr("xmlns:ofd", "http://www.ofdspec.org/2016");

        // Page area (ofdrw uses integer format when value is whole number)
        let width_fmt = format_number(page.width);
        let height_fmt = format_number(page.height);
        let mut area = XmlNode::element("ofd:Area");
        let mut physical_box = XmlNode::element("ofd:PhysicalBox");
        physical_box.push_child(XmlNode::text_node(format!("0 0 {width_fmt} {height_fmt}")));
        area.push_child(physical_box);
        page_node.push_child(area);

        // Content layer wrapped in Layer element (ofdrw pattern)
        let mut content = XmlNode::element("ofd:Content");
        let mut layer = XmlNode::element("ofd:Layer");

        // Collect image indices for this page.
        let mut image_counter = 0usize;

        for (object_index, obj) in page.content.iter().enumerate() {
            match obj {
                ContentObject::Text(text) => {
                    let x = text.x;
                    let y = text.y;
                    let character_count =
                        f64::from(u32::try_from(text.text.chars().count()).unwrap_or(u32::MAX));
                    let est_width = text.width.unwrap_or(character_count * text.size * 0.06);
                    let est_height = text.height.unwrap_or(text.size * 0.4);
                    let fill_color = format!("{:06X}", text.color);
                    let idx = page_index * 1000 + object_index;

                    let mut text_obj = XmlNode::element("ofd:TextObject")
                        .attr("ID", format!("t_{page_index}_{idx}"))
                        .attr(
                            "Boundary",
                            format!("{x:.2} {y:.2} {est_width:.2} {est_height:.2}"),
                        )
                        .attr("Font", &text.font)
                        .attr("Size", format!("{:.1}", text.size))
                        .attr("FillColor", &fill_color)
                        .attr("Weight", text.weight.to_string());

                    if text.italic {
                        text_obj
                            .attrs
                            .push(("Italic".to_string(), "true".to_string()));
                    }

                    // TextCode child
                    let mut text_code = XmlNode::element("ofd:TextCode")
                        .attr("X", "0")
                        .attr("Y", format!("{:.2}", text.size * 0.8));
                    text_code.push_child(XmlNode::text_node(&text.text));
                    text_obj.push_child(text_code);

                    layer.push_child(text_obj);
                }
                ContentObject::Image(img) => {
                    let global_image_index = page_image_start + image_counter;
                    let res_id = 100 + global_image_index;
                    let idx = page_index * 1000 + object_index;

                    layer.push_child(
                        XmlNode::element("ofd:ImageObject")
                            .attr("ID", format!("i_{page_index}_{idx}"))
                            .attr(
                                "Boundary",
                                format!(
                                    "{x:.2} {y:.2} {w:.2} {h:.2}",
                                    x = img.x,
                                    y = img.y,
                                    w = img.width,
                                    h = img.height,
                                ),
                            )
                            .attr("ResourceID", res_id.to_string()),
                    );
                    image_counter += 1;
                }
                ContentObject::Path(path) => {
                    let stroke = format!("{:06X}", path.stroke_color);
                    let idx = page_index * 1000 + object_index;

                    let mut path_obj = XmlNode::element("ofd:PathObject")
                        .attr("ID", format!("p_{page_index}_{idx}"))
                        .attr(
                            "Boundary",
                            format!("{x:.2} {y:.2} 0 0", x = path.x, y = path.y),
                        )
                        .attr("StrokeColor", &stroke)
                        .attr("LineWidth", format!("{:.2}", path.stroke_width));

                    if let Some(fc) = path.fill_color {
                        path_obj
                            .attrs
                            .push(("FillColor".to_string(), format!("{fc:06X}")));
                    }

                    let mut abbreviated = XmlNode::element("ofd:AbbreviatedData");
                    abbreviated.push_child(XmlNode::text_node(&path.path_data));
                    path_obj.push_child(abbreviated);

                    layer.push_child(path_obj);
                }
            }
        }

        content.push_child(layer);
        page_node.push_child(content);

        // ── Serialize the complete tree ──
        let mut xml = String::with_capacity(2048);
        xml.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
        xml.push('\n');
        xml.push_str(&page_node.to_xml_string());
        xml.push('\n');
        xml
    }
}
