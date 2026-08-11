//! OFD XML 文档生成。
//!
//! 包含 OfdWriter 的 XML 构建方法，用于生成 GB/T 33190-2016 标准的 XML 文件。

use crate::OfdWriter;
use crate::helpers::xml_escape;
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
        let mut xml = String::with_capacity(512);
        xml.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
        xml.push('\n');
        xml.push_str(&format!(
            r#"<ofd:OFD xmlns:ofd="http://www.ofdspec.org/2016" Version="{}" DocType="OFD">"#,
            self.options.metadata.version
        ));
        xml.push('\n');
        xml.push_str(r"  <ofd:DocBody>");
        xml.push('\n');
        xml.push_str(r"    <ofd:DocInfo>");
        xml.push('\n');

        if let Some(ref doc_id) = self.options.metadata.doc_id {
            xml.push_str(&format!(
                "      <ofd:DocID>{}</ofd:DocID>",
                xml_escape(doc_id)
            ));
            xml.push('\n');
        }
        if let Some(ref title) = self.options.metadata.title {
            xml.push_str(&format!(
                "      <ofd:Title>{}</ofd:Title>",
                xml_escape(title)
            ));
            xml.push('\n');
        }
        if let Some(ref author) = self.options.metadata.author {
            xml.push_str(&format!(
                "      <ofd:Author>{}</ofd:Author>",
                xml_escape(author)
            ));
            xml.push('\n');
        }
        if let Some(ref creator) = self.options.metadata.creator {
            xml.push_str(&format!(
                "      <ofd:Creator>{}</ofd:Creator>",
                xml_escape(creator)
            ));
            xml.push('\n');
        }
        if let Some(ref creator_version) = self.options.metadata.creator_version {
            xml.push_str(&format!(
                "      <ofd:CreatorVersion>{}</ofd:CreatorVersion>",
                xml_escape(creator_version)
            ));
            xml.push('\n');
        }
        if let Some(dt) = self.options.metadata.creation_date {
            xml.push_str(&format!(
                "      <ofd:CreationDate>{}</ofd:CreationDate>",
                dt.format("%Y-%m-%dT%H:%M:%S")
            ));
            xml.push('\n');
        }
        if let Some(dt) = self.options.metadata.mod_date {
            xml.push_str(&format!(
                "      <ofd:ModDate>{}</ofd:ModDate>",
                dt.format("%Y-%m-%dT%H:%M:%S")
            ));
            xml.push('\n');
        }
        if let Some(ref doc_usage) = self.options.metadata.doc_usage {
            xml.push_str(&format!(
                "      <ofd:DocUsage>{}</ofd:DocUsage>",
                xml_escape(doc_usage)
            ));
            xml.push('\n');
        }
        if let Some(ref keywords) = self.options.metadata.keywords {
            xml.push_str(&format!(
                "      <ofd:Keywords>{}</ofd:Keywords>",
                xml_escape(keywords)
            ));
            xml.push('\n');
        }

        // CustomDatas
        if let Some(ref custom_datas) = self.options.metadata.custom_datas {
            if !custom_datas.is_empty() {
                xml.push_str(r"      <ofd:CustomDatas>");
                xml.push('\n');
                for item in &custom_datas.items {
                    xml.push_str(&format!(
                        r#"        <ofd:CustomData Name="{}">{}</ofd:CustomData>"#,
                        xml_escape(&item.name),
                        xml_escape(&item.value),
                    ));
                    xml.push('\n');
                }
                xml.push_str(r"      </ofd:CustomDatas>");
                xml.push('\n');
            }
        }

        xml.push_str(r"    </ofd:DocInfo>");
        xml.push('\n');
        xml.push_str(&format!(
            r"    <ofd:DocRoot>{doc_dir}/{doc_file}</ofd:DocRoot>",
            doc_dir = self.options.metadata.doc_dir,
            doc_file = self.options.metadata.document_file,
        ));
        xml.push('\n');

        // Signatures container reference (ofdrw writes it inside DocBody,
        // after DocRoot, pointing at the signatures list).
        if let Some(ref signatures_path) = self.options.metadata.signatures_path {
            xml.push_str(&format!(
                "  <ofd:Signatures>{}</ofd:Signatures>",
                xml_escape(signatures_path)
            ));
            xml.push('\n');
        }

        xml.push_str(r"  </ofd:DocBody>");
        xml.push('\n');
        xml.push_str(r"</ofd:OFD>");
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
        let mut xml = String::with_capacity(512);
        xml.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
        xml.push('\n');
        xml.push_str(r#"<ofd:DocumentRes xmlns:ofd="http://www.ofdspec.org/2016">"#);
        xml.push('\n');
        xml.push_str(r"  <ofd:MultiMedias>");
        xml.push('\n');

        for (i, (res_name, _, fmt)) in image_resources.iter().enumerate() {
            let type_str = match fmt {
                ImageFormat::Jpeg => "JPEG",
                ImageFormat::Png => "PNG",
                ImageFormat::Bmp => "BMP",
                ImageFormat::Tiff => "TIFF",
            };
            // The BaseLoc is relative to the doc directory.
            let relative = strip_doc_dir_prefix(res_name, &self.options.metadata.doc_dir);
            xml.push_str(&format!(
                r#"    <ofd:MultiMedia ID="{}" Type="{}"><ofd:MediaFile>{}</ofd:MediaFile></ofd:MultiMedia>"#,
                100 + i,
                type_str,
                relative,
            ));
            xml.push('\n');
        }

        xml.push_str(r"  </ofd:MultiMedias>");
        xml.push('\n');
        xml.push_str(r"</ofd:DocumentRes>");
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

        let mut xml = String::with_capacity(256);
        xml.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
        xml.push('\n');
        xml.push_str(r#"<ofd:Res xmlns:ofd="http://www.ofdspec.org/2016" BaseLoc="Res">"#);
        xml.push('\n');

        // Fonts container (always present, even if empty)
        xml.push_str(r"  <ofd:Fonts>");
        xml.push('\n');
        for (i, font_name) in fonts.iter().enumerate() {
            xml.push_str(&format!(
                r#"    <ofd:Font ID="{id}" FontName="{name}"/>"#,
                id = 400 + i,
                name = xml_escape(font_name),
            ));
            xml.push('\n');
        }
        xml.push_str(r"  </ofd:Fonts>");
        xml.push('\n');

        // MultiMedias container for image resources
        if !image_resources.is_empty() {
            xml.push_str(r"  <ofd:MultiMedias>");
            xml.push('\n');
            for (i, (res_name, _, fmt)) in image_resources.iter().enumerate() {
                let type_str = match fmt {
                    ImageFormat::Jpeg => "JPEG",
                    ImageFormat::Png => "PNG",
                    ImageFormat::Bmp => "BMP",
                    ImageFormat::Tiff => "TIFF",
                };
                let relative = strip_doc_dir_prefix(res_name, &self.options.metadata.doc_dir);
                xml.push_str(&format!(
                    r#"    <ofd:MultiMedia ID="{}" Type="{}"><ofd:MediaFile>{}</ofd:MediaFile></ofd:MultiMedia>"#,
                    100 + i,
                    type_str,
                    relative,
                ));
                xml.push('\n');
            }
            xml.push_str(r"  </ofd:MultiMedias>");
            xml.push('\n');
        }

        xml.push_str(r"</ofd:Res>");
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
        let mut xml = String::with_capacity(2048);
        xml.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
        xml.push('\n');
        xml.push_str(r#"<ofd:Page xmlns:ofd="http://www.ofdspec.org/2016">"#);
        xml.push('\n');

        // Page area (ofdrw uses integer format when value is whole number)
        let width_fmt = format_number(page.width);
        let height_fmt = format_number(page.height);
        xml.push_str(&format!(
            r"  <ofd:Area><ofd:PhysicalBox>0 0 {width_fmt} {height_fmt}</ofd:PhysicalBox></ofd:Area>"
        ));
        xml.push('\n');

        // Content layer wrapped in Layer element (ofdrw pattern)
        xml.push_str(r"  <ofd:Content>");
        xml.push('\n');
        xml.push_str(r"    <ofd:Layer>");
        xml.push('\n');

        // Collect image indices for this page.
        let mut image_counter = 0usize;

        for (object_index, obj) in page.content.iter().enumerate() {
            match obj {
                ContentObject::Text(text) => {
                    // mm to OFD units (1 mm = ~3.543307 pixels at 96dpi, but OFD uses mm directly)
                    let x = text.x;
                    let y = text.y;
                    // Estimate text width: ~0.3mm per character for 12pt SimSun (rough heuristic)
                    let character_count =
                        f64::from(u32::try_from(text.text.chars().count()).unwrap_or(u32::MAX));
                    let est_width = text.width.unwrap_or(character_count * text.size * 0.06);
                    let est_height = text.height.unwrap_or(text.size * 0.4);
                    let fill_color = format!("{:06X}", text.color);

                    xml.push_str(&format!(
                        r#"    <ofd:TextObject ID="t_{page_index}_{idx}" Boundary="{x:.2} {y:.2} {w:.2} {h:.2}" Font="{font}" Size="{size:.1}" FillColor="{fill_color}" Weight="{weight}""#,
                        idx = page_index * 1000 + object_index,
                        w = est_width,
                        h = est_height,
                        font = text.font,
                        size = text.size,
                        fill_color = fill_color,
                        weight = text.weight,
                    ));
                    if text.italic {
                        xml.push_str(r#" Italic="true">"#);
                    } else {
                        xml.push('>');
                    }
                    xml.push('\n');

                    // TextCode
                    xml.push_str(&format!(
                        r#"      <ofd:TextCode X="0" Y="{y:.2}">{text}</ofd:TextCode>"#,
                        y = text.size * 0.8,
                        text = xml_escape(&text.text),
                    ));
                    xml.push('\n');

                    xml.push_str(r"    </ofd:TextObject>");
                    xml.push('\n');
                }
                ContentObject::Image(img) => {
                    // Find the resource ID for this image.
                    let global_image_index = page_image_start + image_counter;
                    let res_id = 100 + global_image_index;

                    xml.push_str(&format!(
                        r#"    <ofd:ImageObject ID="i_{page_index}_{idx}" Boundary="{x:.2} {y:.2} {w:.2} {h:.2}" ResourceID="{res_id}"/>"#,
                        idx = page_index * 1000 + object_index,
                        x = img.x,
                        y = img.y,
                        w = img.width,
                        h = img.height,
                    ));
                    xml.push('\n');
                    image_counter += 1;
                }
                ContentObject::Path(path) => {
                    let stroke = format!("{:06X}", path.stroke_color);
                    xml.push_str(&format!(
                        r#"    <ofd:PathObject ID="p_{page_index}_{idx}" Boundary="{x:.2} {y:.2} 0 0" StrokeColor="{stroke}" LineWidth="{lw:.2}""#,
                        idx = page_index * 1000 + object_index,
                        x = path.x,
                        y = path.y,
                        lw = path.stroke_width,
                    ));
                    if let Some(fc) = path.fill_color {
                        xml.push_str(&format!(r#" FillColor="{fc:06X}""#));
                    }
                    xml.push('>');
                    xml.push('\n');
                    xml.push_str(&format!(
                        r"      <ofd:AbbreviatedData>{}</ofd:AbbreviatedData>",
                        xml_escape(&path.path_data),
                    ));
                    xml.push('\n');
                    xml.push_str(r"    </ofd:PathObject>");
                    xml.push('\n');
                }
            }
        }

        xml.push_str(r"    </ofd:Layer>");
        xml.push('\n');
        xml.push_str(r"  </ofd:Content>");
        xml.push('\n');
        xml.push_str(r"</ofd:Page>");
        xml.push('\n');
        xml
    }
}
