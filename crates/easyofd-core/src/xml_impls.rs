//! XmlElement trait 实现集合。
//!
//! 为 easyofd-core 中的 OFD 元素类型实现 [`XmlElement`] trait，
//! 使其可 [`to_xml`](XmlElement::to_xml) 序列化、
//! [`from_xml`](XmlElement::from_xml) 反序列化。
//!
//! 对应 Java: ofdrw 中各 OFDElement 子类的 toXML /代理解析。

use crate::OfdMetadata;
use crate::action::{CTDest, DestType, Goto};
use crate::annotation::{Annot, AnnotType, Appearance};
use crate::attachment::{Attachments, CTAttachment};
use crate::basic_type::ST_Loc;
use crate::doc::doc_body::DocBody;
use crate::doc::document::{Document, PageRef};
use crate::doc::keywords::Keywords;
use crate::doc::outlines::{CT_OutlineElem, Outlines};
use crate::doc::pages::{PageEntry, Pages};
use crate::doc::res::Res;
use crate::image::CT_Image;
use crate::page_obj::{
    CT_CommonData, CT_Layer, CT_PageArea, CT_TemplatePage, LayerType, TemplateZOrder,
};
use crate::page_obj::{
    CT_PageBlock, PageBlockImageObject, PageBlockPathObject, PageBlockTextObject,
};
use crate::text::{CT_CGTransform, CT_Text, TextCode};
use crate::xml_element::{XmlElement, XmlElementError, XmlNode};
use crate::xml_parse::parse_xml_to_nodes;

// ═══════════════════════════════════════════════════════════════
// 辅助函数
// ═══════════════════════════════════════════════════════════════

/// 将 [`XmlElement`] 实例转为 [`XmlNode`]（用于 `child_nodes` 构建嵌套树）。
fn to_xml_node<E: XmlElement>(el: &E) -> XmlNode {
    let mut node = XmlNode::element(el.element_name());
    for (k, v) in el.attributes() {
        node.attrs.push((k, v));
    }
    for child in el.child_nodes() {
        node.push_child(child);
    }
    if let Some(text) = el.text_content() {
        // 用 text_node 子节点表示文本内容，与 write_self_xml 的序列化路径一致。
        node.push_child(XmlNode::text_node(text));
    }
    node
}

/// 创建带文本内容的子元素节点。
///
/// 输出形如 `<name>text</name>`。
fn text_child(name: &str, text: &str) -> XmlNode {
    let mut node = XmlNode::element(name);
    node.push_child(XmlNode::text_node(text));
    node
}

/// 从节点读取属性并解析为 `u32`。
fn attr_u32(node: &XmlNode, key: &str) -> Option<u32> {
    node.get_attr(key).and_then(|s| s.parse().ok())
}

/// 从节点读取属性并解析为 `f64`。
fn attr_f64(node: &XmlNode, key: &str) -> Option<f64> {
    node.get_attr(key).and_then(|s| s.parse().ok())
}

/// 从节点读取属性并解析为 `bool`（缺省返回 `default`）。
fn attr_bool(node: &XmlNode, key: &str, default: bool) -> bool {
    match node.get_attr(key) {
        Some("true" | "1") => true,
        Some("false" | "0") => false,
        _ => default,
    }
}

/// 从首个匹配名字的子元素提取文本内容。
fn child_text(node: &XmlNode, name: &str) -> Option<String> {
    node.child(name).and_then(|c| c.text.clone())
}

/// 从首个匹配名字的子元素提取文本并解析为 `u32`。
fn child_u32(node: &XmlNode, name: &str) -> Option<u32> {
    child_text(node, name).and_then(|s| s.parse().ok())
}

/// 从节点属性解析 [`LayerType`]。
fn parse_layer_type(s: &str) -> LayerType {
    match s {
        "Foreground" => LayerType::Foreground,
        "Background" => LayerType::Background,
        _ => LayerType::Body,
    }
}

/// 从节点属性解析 [`TemplateZOrder`]。
fn parse_z_order(s: &str) -> Option<TemplateZOrder> {
    match s {
        "Back" => Some(TemplateZOrder::Back),
        "Front" => Some(TemplateZOrder::Front),
        _ => None,
    }
}

/// 从节点属性解析 [`AnnotType`]。
fn parse_annot_type(s: &str) -> AnnotType {
    AnnotType::from_str_opt(s).unwrap_or(AnnotType::Text)
}

/// 从节点属性解析 [`DestType`]。
fn parse_dest_type(s: &str) -> DestType {
    match s {
        "XYZ" => DestType::XYZ,
        "FitH" => DestType::FitH,
        "FitV" => DestType::FitV,
        "FitBH" => DestType::FitBH,
        "FitBV" => DestType::FitBV,
        _ => DestType::Fit,
    }
}

// ═══════════════════════════════════════════════════════════════
// doc 模块类型
// ═══════════════════════════════════════════════════════════════

impl XmlElement for PageEntry {
    /// 对应 Java: org.ofdrw.core.basicStructure.pageTree.Page
    fn element_name(&self) -> &'static str {
        "Page"
    }

    fn attributes(&self) -> Vec<(String, String)> {
        vec![
            ("ID".to_string(), self.id.to_string()),
            ("BaseLoc".to_string(), self.base_loc.clone()),
        ]
    }

    fn from_xml(node: &XmlNode) -> Result<Self, XmlElementError> {
        let id = attr_u32(node, "ID").unwrap_or(0);
        let base_loc = node.get_attr("BaseLoc").unwrap_or_default().to_string();
        Ok(Self { id, base_loc })
    }
}

impl XmlElement for Pages {
    /// 对应 Java: org.ofdrw.core.basicStructure.pageTree.Pages
    fn element_name(&self) -> &'static str {
        "Pages"
    }

    fn attributes(&self) -> Vec<(String, String)> {
        Vec::new()
    }

    fn child_nodes(&self) -> Vec<XmlNode> {
        self.pages.iter().map(to_xml_node).collect()
    }

    fn from_xml(node: &XmlNode) -> Result<Self, XmlElementError> {
        let pages = node
            .children_named("Page")
            .map(PageEntry::from_xml)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self { pages })
    }
}

impl XmlElement for Document {
    /// 对应 Java: org.ofdrw.core.basicStructure.doc.Document
    fn element_name(&self) -> &'static str {
        "Document"
    }

    fn attributes(&self) -> Vec<(String, String)> {
        Vec::new()
    }

    fn child_nodes(&self) -> Vec<XmlNode> {
        let mut nodes = Vec::new();
        if let Some(ref cd) = self.common_data {
            nodes.push(text_child("CommonData", cd));
        }
        if !self.pages.is_empty() {
            let pages_node = {
                let p = Pages {
                    pages: self
                        .pages
                        .iter()
                        .map(|pr| PageEntry {
                            id: pr.id,
                            base_loc: pr.base_loc.loc().to_string(),
                        })
                        .collect(),
                };
                to_xml_node(&p)
            };
            nodes.push(pages_node);
        }
        if let Some(ref ol) = self.outlines {
            nodes.push(text_child("Outlines", ol));
        }
        if let Some(ref pm) = self.permissions {
            nodes.push(text_child("Permissions", pm));
        }
        if let Some(ref ac) = self.actions {
            nodes.push(text_child("Actions", ac));
        }
        if let Some(ref vp) = self.v_preferences {
            nodes.push(text_child("VPreferences", vp));
        }
        if let Some(ref bm) = self.bookmarks {
            nodes.push(text_child("Bookmarks", bm));
        }
        if let Some(ref an) = self.annotations {
            nodes.push(text_child("Annotations", an.loc()));
        }
        if let Some(ref ct) = self.custom_tags {
            nodes.push(text_child("CustomTags", ct.loc()));
        }
        if let Some(ref at) = self.attachments {
            nodes.push(text_child("Attachments", at.loc()));
        }
        if let Some(ref ex) = self.extensions {
            nodes.push(text_child("Extensions", ex.loc()));
        }
        nodes
    }

    fn from_xml(node: &XmlNode) -> Result<Self, XmlElementError> {
        let common_data = child_text(node, "CommonData");
        let pages = node
            .child("Pages")
            .map(|pn| {
                pn.children_named("Page")
                    .map(|pe| {
                        Ok::<_, XmlElementError>(PageRef::new(
                            attr_u32(pe, "ID").unwrap_or(0),
                            ST_Loc::new(pe.get_attr("BaseLoc").unwrap_or_default()),
                        ))
                    })
                    .collect::<Result<Vec<_>, _>>()
            })
            .transpose()?
            .unwrap_or_default();
        let outlines = child_text(node, "Outlines");
        let permissions = child_text(node, "Permissions");
        let actions = child_text(node, "Actions");
        let v_preferences = child_text(node, "VPreferences");
        let bookmarks = child_text(node, "Bookmarks");
        let annotations = child_text(node, "Annotations").map(|s| ST_Loc::new(&s));
        let custom_tags = child_text(node, "CustomTags").map(|s| ST_Loc::new(&s));
        let attachments = child_text(node, "Attachments").map(|s| ST_Loc::new(&s));
        let extensions = child_text(node, "Extensions").map(|s| ST_Loc::new(&s));
        Ok(Self {
            common_data,
            pages,
            outlines,
            permissions,
            actions,
            v_preferences,
            bookmarks,
            annotations,
            custom_tags,
            attachments,
            extensions,
        })
    }
}

impl XmlElement for DocBody {
    /// 对应 Java: org.ofdrw.core.basicStructure.ofd.DocBody
    fn element_name(&self) -> &'static str {
        "DocBody"
    }

    fn attributes(&self) -> Vec<(String, String)> {
        Vec::new()
    }

    fn child_nodes(&self) -> Vec<XmlNode> {
        let mut nodes = vec![text_child("DocRoot", self.doc_root.loc())];
        if let Some(ref info) = self.doc_info {
            nodes.push(text_child("DocInfo", info));
        }
        if let Some(ref sig) = self.signatures {
            nodes.push(text_child("Signatures", sig.loc()));
        }
        nodes
    }

    fn from_xml(node: &XmlNode) -> Result<Self, XmlElementError> {
        let doc_root = child_text(node, "DocRoot")
            .ok_or_else(|| XmlElementError("DocBody 缺少 DocRoot".to_string()))?;
        let doc_info = child_text(node, "DocInfo");
        let signatures = child_text(node, "Signatures").map(|s| ST_Loc::new(&s));
        Ok(Self {
            doc_root: ST_Loc::new(&doc_root),
            doc_info,
            signatures,
        })
    }
}

impl XmlElement for Res {
    /// 对应 Java: org.ofdrw.core.basicStructure.res.Res
    fn element_name(&self) -> &'static str {
        "Res"
    }

    fn attributes(&self) -> Vec<(String, String)> {
        let mut attrs = Vec::new();
        if let Some(ref loc) = self.base_loc {
            attrs.push(("BaseLoc".to_string(), loc.loc().to_string()));
        }
        attrs
    }

    fn child_nodes(&self) -> Vec<XmlNode> {
        self.resources
            .iter()
            .filter_map(|r| parse_xml_to_nodes(r).ok())
            .collect()
    }

    fn from_xml(node: &XmlNode) -> Result<Self, XmlElementError> {
        let base_loc = node.get_attr("BaseLoc").map(ST_Loc::new);
        let resources = node.children.iter().map(|c| c.to_xml_string()).collect();
        Ok(Self {
            base_loc,
            resources,
        })
    }
}

impl XmlElement for CT_OutlineElem {
    /// 对应 Java: org.ofdrw.core.basicStructure.outlines.CT_OutlineElem
    fn element_name(&self) -> &'static str {
        "OutlineElem"
    }

    fn attributes(&self) -> Vec<(String, String)> {
        let mut attrs = vec![("Title".to_string(), self.title.clone())];
        if let Some(p) = self.page {
            attrs.push(("Page".to_string(), p.to_string()));
        }
        attrs
    }

    fn child_nodes(&self) -> Vec<XmlNode> {
        self.children.iter().map(to_xml_node).collect()
    }

    fn from_xml(node: &XmlNode) -> Result<Self, XmlElementError> {
        let title = node.get_attr("Title").unwrap_or_default().to_string();
        let page = attr_u32(node, "Page");
        let children = node
            .children_named("OutlineElem")
            .map(CT_OutlineElem::from_xml)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            title,
            page,
            children,
        })
    }
}

impl XmlElement for Outlines {
    /// 对应 Java: org.ofdrw.core.basicStructure.outlines.Outlines
    fn element_name(&self) -> &'static str {
        "Outlines"
    }

    fn attributes(&self) -> Vec<(String, String)> {
        Vec::new()
    }

    fn child_nodes(&self) -> Vec<XmlNode> {
        self.elements.iter().map(to_xml_node).collect()
    }

    fn from_xml(node: &XmlNode) -> Result<Self, XmlElementError> {
        let elements = node
            .children_named("OutlineElem")
            .map(CT_OutlineElem::from_xml)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self { elements })
    }
}

impl XmlElement for Keywords {
    /// 对应 Java: org.ofdrw.core.basicStructure.ofd.docInfo.Keywords
    fn element_name(&self) -> &'static str {
        "Keywords"
    }

    fn attributes(&self) -> Vec<(String, String)> {
        Vec::new()
    }

    fn child_nodes(&self) -> Vec<XmlNode> {
        self.keywords
            .iter()
            .map(|kw| text_child("Keyword", kw))
            .collect()
    }

    fn from_xml(node: &XmlNode) -> Result<Self, XmlElementError> {
        let keywords = node
            .children_named("Keyword")
            .filter_map(|c| c.text.clone())
            .collect();
        Ok(Self { keywords })
    }
}

// ═══════════════════════════════════════════════════════════════
// page_obj 模块类型
// ═══════════════════════════════════════════════════════════════

impl XmlElement for CT_TemplatePage {
    /// 对应 Java: org.ofdrw.core.basicStructure.pageObj.CT_TemplatePage
    fn element_name(&self) -> &'static str {
        "TemplatePage"
    }

    fn attributes(&self) -> Vec<(String, String)> {
        let mut attrs = vec![("ID".to_string(), self.id.to_string())];
        if let Some(ref name) = self.name {
            attrs.push(("TemplatePageName".to_string(), name.clone()));
        }
        if let Some(zo) = self.z_order {
            attrs.push(("ZOrder".to_string(), zo.as_str().to_string()));
        }
        if let Some(ref loc) = self.base_loc {
            attrs.push(("BaseLoc".to_string(), loc.clone()));
        }
        attrs
    }

    fn from_xml(node: &XmlNode) -> Result<Self, XmlElementError> {
        Ok(Self {
            id: attr_u32(node, "ID").unwrap_or(0),
            name: node.get_attr("TemplatePageName").map(String::from),
            z_order: node.get_attr("ZOrder").and_then(parse_z_order),
            base_loc: node.get_attr("BaseLoc").map(String::from),
        })
    }
}

impl XmlElement for CT_PageArea {
    /// 对应 Java: org.ofdrw.core.basicStructure.doc.CT_PageArea
    fn element_name(&self) -> &'static str {
        "PageArea"
    }

    fn attributes(&self) -> Vec<(String, String)> {
        Vec::new()
    }

    // GB/T 33190-2016：PhysicalBox/ApplicationBox 等是 PageArea 的**子元素**
    // （<ofd:PageArea><ofd:PhysicalBox>0 0 210 297</ofd:PhysicalBox>...），
    // 与 ofdrw 输出一致（不是属性）。
    fn child_nodes(&self) -> Vec<XmlNode> {
        let mut nodes = Vec::new();
        if let Some(ref pb) = self.physical_box {
            nodes.push(text_child("PhysicalBox", pb));
        }
        if let Some(ref ab) = self.application_box {
            nodes.push(text_child("ApplicationBox", ab));
        }
        if let Some(ref cb) = self.content_box {
            nodes.push(text_child("ContentBox", cb));
        }
        if let Some(ref bb) = self.bleed_box {
            nodes.push(text_child("BleedBox", bb));
        }
        nodes
    }

    fn from_xml(node: &XmlNode) -> Result<Self, XmlElementError> {
        Ok(Self {
            physical_box: node.child("PhysicalBox").and_then(|c| c.text.clone()),
            application_box: node.child("ApplicationBox").and_then(|c| c.text.clone()),
            content_box: node.child("ContentBox").and_then(|c| c.text.clone()),
            bleed_box: node.child("BleedBox").and_then(|c| c.text.clone()),
        })
    }
}

impl XmlElement for CT_CommonData {
    /// 对应 Java: org.ofdrw.core.basicStructure.doc.CT_CommonData
    fn element_name(&self) -> &'static str {
        "CommonData"
    }

    fn attributes(&self) -> Vec<(String, String)> {
        Vec::new()
    }

    fn child_nodes(&self) -> Vec<XmlNode> {
        let mut nodes = Vec::new();
        if let Some(id) = self.max_unit_id {
            nodes.push(text_child("MaxUnitID", &id.to_string()));
        }
        if let Some(ref pa) = self.page_area {
            nodes.push(to_xml_node(pa));
        }
        for res in &self.public_res {
            nodes.push(text_child("PublicRes", res));
        }
        for res in &self.document_res {
            nodes.push(text_child("DocumentRes", res));
        }
        for tpl in &self.template_pages {
            nodes.push(to_xml_node(tpl));
        }
        if let Some(cs) = self.default_cs {
            nodes.push(text_child("DefaultCS", &cs.to_string()));
        }
        nodes
    }

    fn from_xml(node: &XmlNode) -> Result<Self, XmlElementError> {
        let max_unit_id = child_u32(node, "MaxUnitID");
        let page_area = node
            .child("PageArea")
            .map(CT_PageArea::from_xml)
            .transpose()?;
        let public_res = node
            .children_named("PublicRes")
            .filter_map(|c| c.text.clone())
            .collect();
        let document_res = node
            .children_named("DocumentRes")
            .filter_map(|c| c.text.clone())
            .collect();
        let template_pages = node
            .children_named("TemplatePage")
            .map(CT_TemplatePage::from_xml)
            .collect::<Result<Vec<_>, _>>()?;
        let default_cs = child_u32(node, "DefaultCS");
        Ok(Self {
            max_unit_id,
            page_area,
            public_res,
            document_res,
            template_pages,
            default_cs,
        })
    }
}

impl XmlElement for PageBlockTextObject {
    /// 对应 Java: org.ofdrw.core.basicStructure.pageObj.layer.block.TextObject（简化版）
    fn element_name(&self) -> &'static str {
        "TextObject"
    }

    fn attributes(&self) -> Vec<(String, String)> {
        vec![
            ("ID".to_string(), self.id.to_string()),
            ("Boundary".to_string(), self.boundary.clone()),
        ]
    }

    fn text_content(&self) -> Option<&str> {
        Some(&self.content)
    }

    fn from_xml(node: &XmlNode) -> Result<Self, XmlElementError> {
        Ok(Self {
            id: attr_u32(node, "ID").unwrap_or(0),
            boundary: node.get_attr("Boundary").unwrap_or_default().to_string(),
            content: node.text.clone().unwrap_or_default(),
            font_size: 12.0,
        })
    }
}

impl XmlElement for PageBlockPathObject {
    /// 对应 Java: org.ofdrw.core.basicStructure.pageObj.layer.block.PathObject（简化版）
    fn element_name(&self) -> &'static str {
        "PathObject"
    }

    fn attributes(&self) -> Vec<(String, String)> {
        vec![
            ("ID".to_string(), self.id.to_string()),
            ("Boundary".to_string(), self.boundary.clone()),
        ]
    }

    fn child_nodes(&self) -> Vec<XmlNode> {
        vec![text_child("AbbreviatedData", &self.abbreviated_data)]
    }

    fn from_xml(node: &XmlNode) -> Result<Self, XmlElementError> {
        let abbreviated_data = child_text(node, "AbbreviatedData").unwrap_or_default();
        Ok(Self {
            id: attr_u32(node, "ID").unwrap_or(0),
            boundary: node.get_attr("Boundary").unwrap_or_default().to_string(),
            abbreviated_data,
        })
    }
}

impl XmlElement for PageBlockImageObject {
    /// 对应 Java: org.ofdrw.core.basicStructure.pageObj.layer.block.ImageObject（简化版）
    fn element_name(&self) -> &'static str {
        "ImageObject"
    }

    fn attributes(&self) -> Vec<(String, String)> {
        vec![
            ("ID".to_string(), self.id.to_string()),
            ("Boundary".to_string(), self.boundary.clone()),
            ("ResourceID".to_string(), self.resource_id.to_string()),
        ]
    }

    fn from_xml(node: &XmlNode) -> Result<Self, XmlElementError> {
        Ok(Self {
            id: attr_u32(node, "ID").unwrap_or(0),
            boundary: node.get_attr("Boundary").unwrap_or_default().to_string(),
            resource_id: attr_u32(node, "ResourceID").unwrap_or(0),
        })
    }
}

impl XmlElement for CT_PageBlock {
    /// 对应 Java: org.ofdrw.core.basicStructure.pageObj.layer.block.CT_PageBlock
    fn element_name(&self) -> &'static str {
        "PageBlock"
    }

    fn attributes(&self) -> Vec<(String, String)> {
        Vec::new()
    }

    fn child_nodes(&self) -> Vec<XmlNode> {
        let mut nodes = Vec::new();
        for obj in &self.text_objects {
            nodes.push(to_xml_node(obj));
        }
        for obj in &self.path_objects {
            nodes.push(to_xml_node(obj));
        }
        for obj in &self.image_objects {
            nodes.push(to_xml_node(obj));
        }
        for block in &self.page_blocks {
            nodes.push(to_xml_node(block));
        }
        nodes
    }

    fn from_xml(node: &XmlNode) -> Result<Self, XmlElementError> {
        let mut block = CT_PageBlock::new();
        for child in &node.children {
            match child.name.as_str() {
                "TextObject" => {
                    block
                        .text_objects
                        .push(PageBlockTextObject::from_xml(child)?);
                }
                "PathObject" => {
                    block
                        .path_objects
                        .push(PageBlockPathObject::from_xml(child)?);
                }
                "ImageObject" => {
                    block
                        .image_objects
                        .push(PageBlockImageObject::from_xml(child)?);
                }
                "PageBlock" => {
                    block.page_blocks.push(CT_PageBlock::from_xml(child)?);
                }
                _ => {}
            }
        }
        Ok(block)
    }
}

impl XmlElement for CT_Layer {
    /// 对应 Java: org.ofdrw.core.basicStructure.pageObj.layer.CT_Layer
    fn element_name(&self) -> &'static str {
        "Layer"
    }

    fn attributes(&self) -> Vec<(String, String)> {
        let mut attrs = vec![("Type".to_string(), self.layer_type.as_str().to_string())];
        if let Some(dp) = self.draw_param {
            attrs.push(("DrawParam".to_string(), dp.to_string()));
        }
        attrs
    }

    /// Layer 内联 PageBlock 的子内容（不包裹 PageBlock 标签），
    /// 与 ofdrw 的 toXML 行为一致。
    fn child_nodes(&self) -> Vec<XmlNode> {
        self.block.child_nodes()
    }

    fn from_xml(node: &XmlNode) -> Result<Self, XmlElementError> {
        let layer_type = node
            .get_attr("Type")
            .map_or(LayerType::Body, parse_layer_type);
        let draw_param = attr_u32(node, "DrawParam");
        let block = CT_PageBlock::from_xml(node)?;
        Ok(Self {
            layer_type,
            draw_param,
            block,
        })
    }
}

// ═══════════════════════════════════════════════════════════════
// text 模块类型
// ═══════════════════════════════════════════════════════════════

impl XmlElement for TextCode {
    /// 对应 Java: org.ofdrw.core.text.TextCode
    fn element_name(&self) -> &'static str {
        "TextCode"
    }

    fn attributes(&self) -> Vec<(String, String)> {
        let mut attrs = Vec::new();
        if let Some(x) = self.x {
            attrs.push(("X".to_string(), x.to_string()));
        }
        if let Some(y) = self.y {
            attrs.push(("Y".to_string(), y.to_string()));
        }
        if !self.delta_x.is_empty() {
            let s = self
                .delta_x
                .iter()
                .map(|d| d.to_string())
                .collect::<Vec<_>>()
                .join(" ");
            attrs.push(("DeltaX".to_string(), s));
        }
        if !self.delta_y.is_empty() {
            let s = self
                .delta_y
                .iter()
                .map(|d| d.to_string())
                .collect::<Vec<_>>()
                .join(" ");
            attrs.push(("DeltaY".to_string(), s));
        }
        attrs
    }

    fn text_content(&self) -> Option<&str> {
        Some(&self.content)
    }

    fn from_xml(node: &XmlNode) -> Result<Self, XmlElementError> {
        let content = node.text.clone().unwrap_or_default();
        let x = attr_f64(node, "X");
        let y = attr_f64(node, "Y");
        let delta_x = node
            .get_attr("DeltaX")
            .map(|s| {
                s.split_whitespace()
                    .filter_map(|v| v.parse().ok())
                    .collect()
            })
            .unwrap_or_default();
        let delta_y = node
            .get_attr("DeltaY")
            .map(|s| {
                s.split_whitespace()
                    .filter_map(|v| v.parse().ok())
                    .collect()
            })
            .unwrap_or_default();
        Ok(Self {
            content,
            x,
            y,
            delta_x,
            delta_y,
        })
    }
}

impl XmlElement for CT_CGTransform {
    /// 对应 Java: org.ofdrw.core.text.CT_CGTransform
    fn element_name(&self) -> &'static str {
        "CGTransform"
    }

    fn attributes(&self) -> Vec<(String, String)> {
        let mut attrs = Vec::new();
        if let Some(cp) = self.code_position {
            attrs.push(("CodePosition".to_string(), cp.to_string()));
        }
        if let Some(cc) = self.code_count {
            attrs.push(("CodeCount".to_string(), cc.to_string()));
        }
        if let Some(gc) = self.glyph_count {
            attrs.push(("GlyphCount".to_string(), gc.to_string()));
        }
        if !self.glyphs.is_empty() {
            let s = self
                .glyphs
                .iter()
                .map(|g| g.to_string())
                .collect::<Vec<_>>()
                .join(" ");
            attrs.push(("Glyphs".to_string(), s));
        }
        attrs
    }

    fn from_xml(node: &XmlNode) -> Result<Self, XmlElementError> {
        let code_position = attr_u32(node, "CodePosition");
        let code_count = attr_u32(node, "CodeCount");
        let glyph_count = attr_u32(node, "GlyphCount");
        let glyphs = node
            .get_attr("Glyphs")
            .map(|s| {
                s.split_whitespace()
                    .filter_map(|v| v.parse().ok())
                    .collect()
            })
            .unwrap_or_default();
        Ok(Self {
            code_position,
            code_count,
            glyph_count,
            glyphs,
        })
    }
}

impl XmlElement for CT_Text {
    /// 对应 Java: org.ofdrw.core.text.text.CT_Text
    fn element_name(&self) -> &'static str {
        "TextObject"
    }

    fn attributes(&self) -> Vec<(String, String)> {
        let mut attrs = vec![
            ("ID".to_string(), self.id.to_string()),
            ("Boundary".to_string(), self.boundary.clone()),
        ];
        if let Some(fr) = self.font_ref {
            attrs.push(("Font".to_string(), fr.to_string()));
        }
        if let Some(sz) = self.size {
            attrs.push(("Size".to_string(), sz.to_string()));
        }
        if self.stroke {
            attrs.push(("Stroke".to_string(), "true".to_string()));
        }
        if !self.fill {
            attrs.push(("Fill".to_string(), "false".to_string()));
        }
        if let Some(hs) = self.h_scale {
            attrs.push(("HScale".to_string(), hs.to_string()));
        }
        if let Some(rd) = self.read_direction {
            attrs.push(("ReadDirection".to_string(), rd.to_string()));
        }
        if let Some(cd) = self.char_direction {
            attrs.push(("CharDirection".to_string(), cd.to_string()));
        }
        if let Some(w) = self.weight {
            attrs.push(("Weight".to_string(), w.to_string()));
        }
        if self.italic {
            attrs.push(("Italic".to_string(), "true".to_string()));
        }
        if let Some(fc) = self.fill_color {
            attrs.push(("FillColor".to_string(), fc.to_string()));
        }
        if let Some(sc) = self.stroke_color {
            attrs.push(("StrokeColor".to_string(), sc.to_string()));
        }
        attrs
    }

    fn child_nodes(&self) -> Vec<XmlNode> {
        let mut nodes = Vec::new();
        for cg in &self.cg_transforms {
            nodes.push(to_xml_node(cg));
        }
        for tc in &self.text_codes {
            nodes.push(to_xml_node(tc));
        }
        nodes
    }

    fn from_xml(node: &XmlNode) -> Result<Self, XmlElementError> {
        let id = attr_u32(node, "ID").unwrap_or(0);
        let boundary = node.get_attr("Boundary").unwrap_or_default().to_string();
        let font_ref = attr_u32(node, "Font");
        let size = attr_f64(node, "Size");
        let stroke = attr_bool(node, "Stroke", false);
        let fill = attr_bool(node, "Fill", true);
        let h_scale = attr_f64(node, "HScale");
        let read_direction = attr_u32(node, "ReadDirection");
        let char_direction = attr_u32(node, "CharDirection");
        let weight = attr_u32(node, "Weight");
        let italic = attr_bool(node, "Italic", false);
        let fill_color = attr_u32(node, "FillColor");
        let stroke_color = attr_u32(node, "StrokeColor");
        let cg_transforms = node
            .children_named("CGTransform")
            .map(CT_CGTransform::from_xml)
            .collect::<Result<Vec<_>, _>>()?;
        let text_codes = node
            .children_named("TextCode")
            .map(TextCode::from_xml)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            id,
            boundary,
            font_ref,
            size,
            stroke,
            fill,
            h_scale,
            read_direction,
            char_direction,
            weight,
            italic,
            fill_color,
            stroke_color,
            cg_transforms,
            text_codes,
        })
    }
}

// ═══════════════════════════════════════════════════════════════
// image 模块类型
// ═══════════════════════════════════════════════════════════════

impl XmlElement for CT_Image {
    /// 对应 Java: org.ofdrw.core.image.CT_Image
    fn element_name(&self) -> &'static str {
        "Image"
    }

    fn attributes(&self) -> Vec<(String, String)> {
        let mut attrs = vec![
            ("ID".to_string(), self.id.to_string()),
            ("Boundary".to_string(), self.boundary.clone()),
            ("ResourceID".to_string(), self.resource_id.to_string()),
        ];
        if self.interpolate {
            attrs.push(("Interpolate".to_string(), "true".to_string()));
        }
        attrs
    }

    fn from_xml(node: &XmlNode) -> Result<Self, XmlElementError> {
        Ok(Self {
            id: attr_u32(node, "ID").unwrap_or(0),
            boundary: node.get_attr("Boundary").unwrap_or_default().to_string(),
            resource_id: attr_u32(node, "ResourceID").unwrap_or(0),
            interpolate: attr_bool(node, "Interpolate", false),
        })
    }
}

// ═══════════════════════════════════════════════════════════════
// action 模块类型
// ═══════════════════════════════════════════════════════════════

impl XmlElement for CTDest {
    /// 对应 Java: org.ofdrw.core.action.CT_Dest
    fn element_name(&self) -> &'static str {
        "Dest"
    }

    fn attributes(&self) -> Vec<(String, String)> {
        let mut attrs = vec![
            ("PageID".to_string(), self.page.to_string()),
            ("Type".to_string(), self.dest_type.to_string()),
        ];
        if let Some(left) = self.left {
            attrs.push(("Left".to_string(), left.to_string()));
        }
        if let Some(top) = self.top {
            attrs.push(("Top".to_string(), top.to_string()));
        }
        if let Some(zoom) = self.zoom {
            attrs.push(("Zoom".to_string(), zoom.to_string()));
        }
        attrs
    }

    fn from_xml(node: &XmlNode) -> Result<Self, XmlElementError> {
        let page = attr_u32(node, "PageID").unwrap_or(0);
        let dest_type = node.get_attr("Type").map_or(DestType::Fit, parse_dest_type);
        let left = attr_f64(node, "Left");
        let top = attr_f64(node, "Top");
        let zoom = attr_f64(node, "Zoom");
        Ok(Self {
            page,
            dest_type,
            left,
            top,
            zoom,
        })
    }
}

impl XmlElement for Goto {
    /// 对应 Java: org.ofdrw.core.action.actionType.Goto
    fn element_name(&self) -> &'static str {
        "Goto"
    }

    fn attributes(&self) -> Vec<(String, String)> {
        Vec::new()
    }

    fn child_nodes(&self) -> Vec<XmlNode> {
        vec![to_xml_node(&self.dest)]
    }

    fn from_xml(node: &XmlNode) -> Result<Self, XmlElementError> {
        let dest = node
            .child("Dest")
            .map(CTDest::from_xml)
            .ok_or_else(|| XmlElementError("Goto 缺少 Dest 子元素".to_string()))??;
        Ok(Self { dest })
    }
}

// ═══════════════════════════════════════════════════════════════
// annotation 模块类型
// ═══════════════════════════════════════════════════════════════

impl XmlElement for Appearance {
    /// 对应 Java: org.ofdrw.core.annotation.Appearance
    fn element_name(&self) -> &'static str {
        "Appearance"
    }

    fn attributes(&self) -> Vec<(String, String)> {
        let mut attrs = vec![
            ("ID".to_string(), self.id.clone()),
            ("Type".to_string(), self.appearance_type.clone()),
        ];
        if let Some(ref path) = self.resource_path {
            attrs.push(("ResourcePath".to_string(), path.clone()));
        }
        attrs
    }

    fn from_xml(node: &XmlNode) -> Result<Self, XmlElementError> {
        Ok(Self {
            id: node.get_attr("ID").unwrap_or_default().to_string(),
            appearance_type: node.get_attr("Type").unwrap_or_default().to_string(),
            resource_path: node.get_attr("ResourcePath").map(String::from),
        })
    }
}

impl XmlElement for Annot {
    /// 对应 Java: org.ofdrw.core.annotation.Annot
    fn element_name(&self) -> &'static str {
        "Annot"
    }

    fn attributes(&self) -> Vec<(String, String)> {
        let mut attrs = vec![
            ("ID".to_string(), self.id.clone()),
            ("Type".to_string(), self.annot_type.as_str().to_string()),
            ("Flags".to_string(), self.flags.to_string()),
        ];
        if let Some(ref creator) = self.creator {
            attrs.push(("Creator".to_string(), creator.clone()));
        }
        if let Some(ref date) = self.last_mod_date {
            attrs.push(("LastModDate".to_string(), date.clone()));
        }
        let [x, y, w, h] = self.location;
        attrs.push(("Location".to_string(), format!("{x} {y} {w} {h}")));
        attrs
    }

    fn child_nodes(&self) -> Vec<XmlNode> {
        self.appearances.iter().map(to_xml_node).collect()
    }

    fn from_xml(node: &XmlNode) -> Result<Self, XmlElementError> {
        let id = node.get_attr("ID").unwrap_or_default().to_string();
        let annot_type = node
            .get_attr("Type")
            .map_or(AnnotType::Text, parse_annot_type);
        let flags = attr_u32(node, "Flags").unwrap_or(0);
        let creator = node.get_attr("Creator").map(String::from);
        let last_mod_date = node.get_attr("LastModDate").map(String::from);
        let location = node
            .get_attr("Location")
            .and_then(|s| {
                let parts: Vec<f64> = s
                    .split_whitespace()
                    .filter_map(|v| v.parse().ok())
                    .collect();
                if parts.len() == 4 {
                    Some([parts[0], parts[1], parts[2], parts[3]])
                } else {
                    None
                }
            })
            .unwrap_or([0.0, 0.0, 0.0, 0.0]);
        let appearances = node
            .children_named("Appearance")
            .map(Appearance::from_xml)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            id,
            creator,
            annot_type,
            flags,
            last_mod_date,
            location,
            appearances,
        })
    }
}

// ═══════════════════════════════════════════════════════════════
// attachment 模块类型
// ═══════════════════════════════════════════════════════════════

impl XmlElement for CTAttachment {
    /// 对应 Java: org.ofdrw.core.attachment.CT_Attachment
    fn element_name(&self) -> &'static str {
        "Attachment"
    }

    fn attributes(&self) -> Vec<(String, String)> {
        let mut attrs = vec![
            ("ID".to_string(), self.id.clone()),
            ("Name".to_string(), self.name.clone()),
        ];
        if let Some(ref fmt) = self.format {
            attrs.push(("Format".to_string(), fmt.clone()));
        }
        if let Some(ref date) = self.creation_date {
            attrs.push(("CreationDate".to_string(), date.clone()));
        }
        if let Some(sz) = self.size {
            attrs.push(("Size".to_string(), sz.to_string()));
        }
        if !self.visible {
            attrs.push(("Visible".to_string(), "false".to_string()));
        }
        if let Some(ref file) = self.file {
            attrs.push(("File".to_string(), file.clone()));
        }
        attrs
    }

    fn from_xml(node: &XmlNode) -> Result<Self, XmlElementError> {
        Ok(Self {
            id: node.get_attr("ID").unwrap_or_default().to_string(),
            name: node.get_attr("Name").unwrap_or_default().to_string(),
            format: node.get_attr("Format").map(String::from),
            creation_date: node.get_attr("CreationDate").map(String::from),
            size: node.get_attr("Size").and_then(|s| s.parse().ok()),
            visible: attr_bool(node, "Visible", true),
            file: node.get_attr("File").map(String::from),
        })
    }
}

impl XmlElement for Attachments {
    /// 对应 Java: org.ofdrw.core.attachment.Attachments
    fn element_name(&self) -> &'static str {
        "Attachments"
    }

    fn attributes(&self) -> Vec<(String, String)> {
        Vec::new()
    }

    fn child_nodes(&self) -> Vec<XmlNode> {
        self.items.iter().map(to_xml_node).collect()
    }

    fn from_xml(node: &XmlNode) -> Result<Self, XmlElementError> {
        let items = node
            .children_named("Attachment")
            .map(CTAttachment::from_xml)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self { items })
    }
}

// ═══════════════════════════════════════════════════════════════
// model 模块类型
// ═══════════════════════════════════════════════════════════════

impl XmlElement for OfdMetadata {
    /// 对应 Java: org.ofdrw.core.basicStructure.ofd.docInfo.CT_DocInfo
    ///
    /// 仅序列化/反序列化 DocInfo 相关字段；非 XML 元素字段
    /// （如 `doc_dir`、`document_file` 等）保持默认值。
    fn element_name(&self) -> &'static str {
        "DocInfo"
    }

    fn attributes(&self) -> Vec<(String, String)> {
        Vec::new()
    }

    fn child_nodes(&self) -> Vec<XmlNode> {
        let mut nodes = Vec::new();
        if let Some(id) = &self.doc_id {
            nodes.push(text_child("DocID", id));
        }
        if let Some(title) = &self.title {
            nodes.push(text_child("Title", title));
        }
        if let Some(author) = &self.author {
            nodes.push(text_child("Author", author));
        }
        if let Some(creator) = &self.creator {
            nodes.push(text_child("Creator", creator));
        }
        if let Some(cv) = &self.creator_version {
            nodes.push(text_child("CreatorVersion", cv));
        }
        if let Some(dt) = &self.creation_date {
            nodes.push(text_child(
                "CreationDate",
                &dt.format("%Y-%m-%dT%H:%M:%S").to_string(),
            ));
        }
        if let Some(dt) = &self.mod_date {
            nodes.push(text_child(
                "ModDate",
                &dt.format("%Y-%m-%dT%H:%M:%S").to_string(),
            ));
        }
        if self.max_unit_id > 0 {
            nodes.push(text_child("MaxUnitID", &self.max_unit_id.to_string()));
        }
        if let Some(usage) = &self.doc_usage {
            nodes.push(text_child("DocUsage", usage));
        }
        if let Some(kw) = &self.keywords {
            nodes.push(text_child("Keywords", kw));
        }
        // 对应 ofdrw CT_DocInfo.Subject
        if let Some(subj) = &self.subject {
            nodes.push(text_child("Subject", subj));
        }
        nodes
    }

    fn from_xml(node: &XmlNode) -> Result<Self, XmlElementError> {
        Ok(Self {
            doc_id: child_text(node, "DocID"),
            title: child_text(node, "Title"),
            author: child_text(node, "Author"),
            creator: child_text(node, "Creator"),
            creator_version: child_text(node, "CreatorVersion"),
            creation_date: child_text(node, "CreationDate")
                .and_then(|s| chrono::NaiveDateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M:%S").ok()),
            mod_date: child_text(node, "ModDate")
                .and_then(|s| chrono::NaiveDateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M:%S").ok()),
            max_unit_id: child_u32(node, "MaxUnitID").unwrap_or(0),
            doc_usage: child_text(node, "DocUsage"),
            keywords: child_text(node, "Keywords"),
            subject: child_text(node, "Subject"),
            ..Default::default()
        })
    }
}

// ═══════════════════════════════════════════════════════════════
// 测试
// ═══════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    /// 辅助：to_xml → parse → from_xml，返回恢复后的类型。
    fn roundtrip_parse<E: XmlElement>(original: &E) -> E {
        let xml = original.to_xml();
        let node =
            parse_xml_to_nodes(&xml).unwrap_or_else(|e| panic!("parse 失败: {e}\nXML: {xml}"));
        E::from_xml(&node).unwrap_or_else(|e| panic!("from_xml 失败: {e}\nXML: {xml}"))
    }

    // ── PageEntry ──

    #[test]
    fn page_entry_roundtrip() {
        let pe = PageEntry::new(5, "Pages/Page_4.xml");
        let r = roundtrip_parse(&pe);
        assert_eq!(r.id, 5);
        assert_eq!(r.base_loc, "Pages/Page_4.xml");
    }

    #[test]
    fn page_entry_to_xml_attrs() {
        let pe = PageEntry::new(1, "Pages/Page_0.xml");
        let xml = pe.to_xml();
        assert!(xml.contains("ID=\"1\""));
        assert!(xml.contains("BaseLoc=\"Pages/Page_0.xml\""));
    }

    // ── Pages ──

    #[test]
    fn pages_roundtrip() {
        let mut p = Pages::new();
        p.add_page(PageEntry::new(1, "Pages/Page_0.xml"));
        p.add_page(PageEntry::new(2, "Pages/Page_1.xml"));
        let r = roundtrip_parse(&p);
        assert_eq!(r.pages.len(), 2);
        assert_eq!(r.pages[0].id, 1);
        assert_eq!(r.pages[1].id, 2);
    }

    #[test]
    fn pages_to_xml_children() {
        let mut p = Pages::new();
        p.add_page(PageEntry::new(1, "P0.xml"));
        let xml = p.to_xml();
        assert!(xml.contains("<Pages>"));
        assert!(xml.contains("<Page "));
        assert!(xml.contains("</Pages>"));
    }

    // ── Document ──

    #[test]
    fn document_roundtrip_basic() {
        let mut doc = Document::new();
        doc.common_data = Some("CommonData.xml".to_string());
        doc.add_page(PageRef::new(1, ST_Loc::new("Pages/Page_0.xml")));
        let r = roundtrip_parse(&doc);
        assert_eq!(r.common_data.as_deref(), Some("CommonData.xml"));
        assert_eq!(r.pages.len(), 1);
        assert_eq!(r.pages[0].id, 1);
    }

    #[test]
    fn document_to_xml_with_refs() {
        let doc = Document::new()
            .annotations(ST_Loc::new("Annots/Annotations.xml"))
            .attachments(ST_Loc::new("Attachs/Attachments.xml"));
        let xml = doc.to_xml();
        assert!(xml.contains("<Document>"));
        assert!(xml.contains("<Annotations>Annots/Annotations.xml</Annotations>"));
        assert!(xml.contains("<Attachments>Attachs/Attachments.xml</Attachments>"));
        assert!(xml.contains("</Document>"));
    }

    // ── DocBody ──

    #[test]
    fn doc_body_roundtrip() {
        let db = DocBody::new(ST_Loc::new("Doc_0/Document.xml"))
            .doc_info("test info")
            .signatures(ST_Loc::new("Doc_0/Signs/Signatures.xml"));
        let xml = db.to_xml();
        let node = parse_xml_to_nodes(&xml).unwrap();
        let restored = DocBody::from_xml(&node).unwrap();
        assert_eq!(restored.doc_root.loc(), "Doc_0/Document.xml");
        assert_eq!(restored.doc_info.as_deref(), Some("test info"));
        assert_eq!(
            restored.signatures.as_ref().map(|s| s.loc()),
            Some("Doc_0/Signs/Signatures.xml")
        );
    }

    #[test]
    fn doc_body_to_xml_children() {
        let db = DocBody::new(ST_Loc::new("Doc_0/Document.xml"));
        let xml = db.to_xml();
        assert!(xml.contains("<DocRoot>Doc_0/Document.xml</DocRoot>"));
    }

    // ── Res ──

    #[test]
    fn res_roundtrip() {
        let mut r = Res::new().base_loc(ST_Loc::new("./Res"));
        r.add_resource("<Font ID=\"1\" FamilyName=\"SimSun\"/>");
        let xml = r.to_xml();
        let node = parse_xml_to_nodes(&xml).unwrap();
        let restored = Res::from_xml(&node).unwrap();
        assert_eq!(restored.base_loc.as_ref().map(|l| l.loc()), Some("./Res"));
        assert_eq!(restored.resource_count(), 1);
    }

    // ── Outlines / CT_OutlineElem ──

    #[test]
    fn outline_elem_roundtrip() {
        let elem = CT_OutlineElem::new("Chapter 1").page(3);
        let r = roundtrip_parse(&elem);
        assert_eq!(r.title, "Chapter 1");
        assert_eq!(r.page, Some(3));
    }

    #[test]
    fn outlines_roundtrip() {
        let mut ol = Outlines::new();
        ol.add(CT_OutlineElem::new("Ch1").page(1));
        ol.add(CT_OutlineElem::new("Ch2").page(5));
        let r = roundtrip_parse(&ol);
        assert_eq!(r.elements.len(), 2);
        assert_eq!(r.elements[0].title, "Ch1");
        assert_eq!(r.elements[1].page, Some(5));
    }

    #[test]
    fn outline_elem_nested() {
        let mut root = CT_OutlineElem::new("Root").page(1);
        root.add_child(CT_OutlineElem::new("Child").page(2));
        let xml = root.to_xml();
        assert!(xml.contains("Title=\"Root\""));
        assert!(xml.contains("Title=\"Child\""));
        let node = parse_xml_to_nodes(&xml).unwrap();
        let restored = CT_OutlineElem::from_xml(&node).unwrap();
        assert_eq!(restored.title, "Root");
        assert_eq!(restored.children.len(), 1);
        assert_eq!(restored.children[0].title, "Child");
    }

    // ── Keywords ──

    #[test]
    fn keywords_roundtrip() {
        let mut kw = Keywords::new();
        kw.add("OFD");
        kw.add("PDF");
        let xml = kw.to_xml();
        let node = parse_xml_to_nodes(&xml).unwrap();
        let restored = Keywords::from_xml(&node).unwrap();
        assert_eq!(restored.keywords, vec!["OFD", "PDF"]);
    }

    // ── CT_TemplatePage ──

    #[test]
    fn template_page_roundtrip() {
        let tpl = CT_TemplatePage::new(10)
            .name("bg")
            .z_order(TemplateZOrder::Front)
            .base_loc("Tpl_10.xml");
        let r = roundtrip_parse(&tpl);
        assert_eq!(r.id, 10);
        assert_eq!(r.name.as_deref(), Some("bg"));
        assert_eq!(r.z_order, Some(TemplateZOrder::Front));
        assert_eq!(r.base_loc.as_deref(), Some("Tpl_10.xml"));
    }

    // ── CT_PageArea ──

    #[test]
    fn page_area_roundtrip() {
        let area = CT_PageArea::new()
            .physical_box(0.0, 0.0, 210.0, 297.0)
            .bleed_box(-5.0, -5.0, 220.0, 307.0);
        let r = roundtrip_parse(&area);
        assert_eq!(r.physical_box.as_deref(), Some("0 0 210 297"));
        assert_eq!(r.bleed_box.as_deref(), Some("-5 -5 220 307"));
    }

    // ── CT_CommonData ──

    #[test]
    fn common_data_roundtrip() {
        let mut cd = CT_CommonData::new()
            .max_unit_id(50)
            .page_area(CT_PageArea::with_physical(0.0, 0.0, 210.0, 297.0))
            .default_cs(2);
        cd.add_public_res("PublicRes.xml");
        cd.add_template_page(CT_TemplatePage::new(1).name("bg"));
        let xml = cd.to_xml();
        let node = parse_xml_to_nodes(&xml).unwrap();
        let restored = CT_CommonData::from_xml(&node).unwrap();
        assert_eq!(restored.max_unit_id, Some(50));
        assert!(restored.page_area.is_some());
        assert_eq!(restored.public_res, vec!["PublicRes.xml"]);
        assert_eq!(restored.default_cs, Some(2));
        assert_eq!(restored.template_pages.len(), 1);
    }

    // ── CT_PageBlock ──

    #[test]
    fn page_block_roundtrip() {
        let mut block = CT_PageBlock::new();
        block.add_text_object(PageBlockTextObject::new(1, "0 0 100 20", "hello"));
        block.add_path_object(PageBlockPathObject::new(2, "0 0 50 50", "M0 0L10 10"));
        block.add_image_object(PageBlockImageObject::new(3, "0 0 100 100", 10));
        let xml = block.to_xml();
        let node = parse_xml_to_nodes(&xml).unwrap();
        let restored = CT_PageBlock::from_xml(&node).unwrap();
        assert_eq!(restored.text_objects.len(), 1);
        assert_eq!(restored.text_objects[0].content, "hello");
        assert_eq!(restored.path_objects.len(), 1);
        assert_eq!(restored.image_objects.len(), 1);
        assert_eq!(restored.image_objects[0].resource_id, 10);
    }

    #[test]
    fn page_block_nested_roundtrip() {
        let mut inner = CT_PageBlock::new();
        inner.add_text_object(PageBlockTextObject::new(1, "0 0 10 10", "x"));
        let mut outer = CT_PageBlock::new();
        outer.add_text_object(PageBlockTextObject::new(2, "0 0 10 10", "y"));
        outer.add_page_block(inner);
        let xml = outer.to_xml();
        let node = parse_xml_to_nodes(&xml).unwrap();
        let restored = CT_PageBlock::from_xml(&node).unwrap();
        assert_eq!(restored.text_objects.len(), 1);
        assert_eq!(restored.page_blocks.len(), 1);
        assert_eq!(restored.page_blocks[0].text_objects.len(), 1);
    }

    // ── CT_Layer ──

    #[test]
    fn layer_roundtrip() {
        let mut layer = CT_Layer::foreground().draw_param(7);
        layer
            .block
            .add_text_object(PageBlockTextObject::new(1, "0 0 50 20", "hi"));
        let xml = layer.to_xml();
        let node = parse_xml_to_nodes(&xml).unwrap();
        let restored = CT_Layer::from_xml(&node).unwrap();
        assert_eq!(restored.layer_type, LayerType::Foreground);
        assert_eq!(restored.draw_param, Some(7));
        assert_eq!(restored.block.text_objects.len(), 1);
        assert_eq!(restored.block.text_objects[0].content, "hi");
    }

    #[test]
    fn layer_to_xml_attrs() {
        let layer = CT_Layer::body();
        let xml = layer.to_xml();
        assert!(xml.contains("Type=\"Body\""));
    }

    // ── TextCode ──

    #[test]
    fn text_code_roundtrip() {
        let tc = TextCode::with_content("Hello")
            .coordinate(10.0, 20.0)
            .delta_x(vec![6.0, 6.0, 6.0]);
        let xml = tc.to_xml();
        let node = parse_xml_to_nodes(&xml).unwrap();
        let restored = TextCode::from_xml(&node).unwrap();
        assert_eq!(restored.content, "Hello");
        assert!((restored.x.unwrap() - 10.0).abs() < f64::EPSILON);
        assert!((restored.y.unwrap() - 20.0).abs() < f64::EPSILON);
        assert_eq!(restored.delta_x.len(), 3);
    }

    // ── CT_CGTransform ──

    #[test]
    fn cg_transform_roundtrip() {
        let cg = CT_CGTransform::new()
            .code_position(0)
            .code_count(2)
            .glyph_count(2)
            .glyphs(vec![100, 200]);
        let r = roundtrip_parse(&cg);
        assert_eq!(r.code_position, Some(0));
        assert_eq!(r.code_count, Some(2));
        assert_eq!(r.glyph_count, Some(2));
        assert_eq!(r.glyphs, vec![100, 200]);
    }

    // ── CT_Text ──

    #[test]
    fn ct_text_roundtrip() {
        let mut t = CT_Text::new(5, "10 20 200 30")
            .font(3)
            .size(12.0)
            .weight(700)
            .italic(true)
            .stroke(true)
            .fill(false);
        t.add_text_code(TextCode::with_content("test").coordinate(10.0, 30.0));
        let xml = t.to_xml();
        let node = parse_xml_to_nodes(&xml).unwrap();
        let restored = CT_Text::from_xml(&node).unwrap();
        assert_eq!(restored.id, 5);
        assert_eq!(restored.boundary, "10 20 200 30");
        assert_eq!(restored.font_ref, Some(3));
        assert!((restored.size.unwrap() - 12.0).abs() < f64::EPSILON);
        assert_eq!(restored.weight, Some(700));
        assert!(restored.italic);
        assert!(restored.stroke);
        assert!(!restored.fill);
        assert_eq!(restored.text_codes.len(), 1);
        assert_eq!(restored.text_codes[0].content, "test");
    }

    #[test]
    fn ct_text_to_xml_attrs() {
        let mut t = CT_Text::new(1, "0 0 100 20").font(2).size(14.0);
        t.add_text_code(TextCode::with_content("x"));
        let xml = t.to_xml();
        assert!(xml.contains("ID=\"1\""));
        assert!(xml.contains("Boundary=\"0 0 100 20\""));
        assert!(xml.contains("Font=\"2\""));
        assert!(xml.contains("Size=\"14\""));
        assert!(xml.contains("<TextObject"));
        assert!(xml.contains("</TextObject>"));
    }

    // ── CT_Image ──

    #[test]
    fn ct_image_roundtrip() {
        let img = CT_Image::new(1, "0 0 100 100", 5).interpolate(true);
        let r = roundtrip_parse(&img);
        assert_eq!(r.id, 1);
        assert_eq!(r.boundary, "0 0 100 100");
        assert_eq!(r.resource_id, 5);
        assert!(r.interpolate);
    }

    #[test]
    fn ct_image_to_xml_attrs() {
        let img = CT_Image::new(2, "10 20 50 50", 3);
        let xml = img.to_xml();
        assert!(xml.contains("ID=\"2\""));
        assert!(xml.contains("ResourceID=\"3\""));
        assert!(!xml.contains("Interpolate"));
    }

    // ── CTDest ──

    #[test]
    fn ct_dest_roundtrip() {
        let dest = CTDest::new(2)
            .dest_type(DestType::XYZ)
            .left(10.5)
            .top(20.5)
            .zoom(2.0);
        let r = roundtrip_parse(&dest);
        assert_eq!(r.page, 2);
        assert_eq!(r.dest_type, DestType::XYZ);
        assert!((r.left.unwrap() - 10.5).abs() < f64::EPSILON);
        assert!((r.top.unwrap() - 20.5).abs() < f64::EPSILON);
        assert!((r.zoom.unwrap() - 2.0).abs() < f64::EPSILON);
    }

    // ── Goto ──

    #[test]
    fn goto_roundtrip() {
        let dest = CTDest::new(3).dest_type(DestType::FitH).left(10.0);
        let goto = Goto::new(dest);
        let xml = goto.to_xml();
        let node = parse_xml_to_nodes(&xml).unwrap();
        let restored = Goto::from_xml(&node).unwrap();
        assert_eq!(restored.dest.page, 3);
        assert_eq!(restored.dest.dest_type, DestType::FitH);
        assert!((restored.dest.left.unwrap() - 10.0).abs() < f64::EPSILON);
    }

    // ── Appearance ──

    #[test]
    fn appearance_roundtrip() {
        let app = Appearance::new("app1", "Normal").resource_path("/res/a.xml");
        let r = roundtrip_parse(&app);
        assert_eq!(r.id, "app1");
        assert_eq!(r.appearance_type, "Normal");
        assert_eq!(r.resource_path.as_deref(), Some("/res/a.xml"));
    }

    // ── Annot ──

    #[test]
    fn annot_roundtrip() {
        let a = Annot::new("ann1", AnnotType::Highlight)
            .creator("user1")
            .flags(4)
            .last_mod_date("2025-01-01T00:00:00")
            .location(10.0, 20.0, 30.0, 40.0)
            .add_appearance(Appearance::new("app1", "Normal"));
        let xml = a.to_xml();
        let node = parse_xml_to_nodes(&xml).unwrap();
        let restored = Annot::from_xml(&node).unwrap();
        assert_eq!(restored.id, "ann1");
        assert_eq!(restored.annot_type, AnnotType::Highlight);
        assert_eq!(restored.creator.as_deref(), Some("user1"));
        assert_eq!(restored.flags, 4);
        assert_eq!(restored.appearances.len(), 1);
    }

    // ── CTAttachment ──

    #[test]
    fn ct_attachment_roundtrip() {
        let a = CTAttachment::new("a1", "test.pdf")
            .format("application/pdf")
            .size(1024)
            .visible(false)
            .file("Attachments/test.pdf");
        let r = roundtrip_parse(&a);
        assert_eq!(r.id, "a1");
        assert_eq!(r.name, "test.pdf");
        assert_eq!(r.format.as_deref(), Some("application/pdf"));
        assert_eq!(r.size, Some(1024));
        assert!(!r.visible);
        assert_eq!(r.file.as_deref(), Some("Attachments/test.pdf"));
    }

    // ── Attachments ──

    #[test]
    fn attachments_roundtrip() {
        let a = Attachments::new()
            .add_attachment(CTAttachment::new("a1", "readme.txt"))
            .add_attachment(CTAttachment::new("a2", "data.xlsx"));
        let xml = a.to_xml();
        let node = parse_xml_to_nodes(&xml).unwrap();
        let restored = Attachments::from_xml(&node).unwrap();
        assert_eq!(restored.len(), 2);
        assert_eq!(restored.items[0].id, "a1");
        assert_eq!(restored.items[1].id, "a2");
    }

    // ── OfdMetadata ──

    #[test]
    fn ofd_metadata_roundtrip() {
        let meta = OfdMetadata {
            doc_id: Some("doc-001".to_string()),
            title: Some("Test Document".to_string()),
            author: Some("Author Name".to_string()),
            creator: Some("easyofd".to_string()),
            creator_version: Some("1.0".to_string()),
            max_unit_id: 42,
            doc_usage: Some("Normal".to_string()),
            keywords: Some("OFD,test".to_string()),
            ..Default::default()
        };
        let xml = meta.to_xml();
        let node = parse_xml_to_nodes(&xml).unwrap();
        let restored = OfdMetadata::from_xml(&node).unwrap();
        assert_eq!(restored.doc_id.as_deref(), Some("doc-001"));
        assert_eq!(restored.title.as_deref(), Some("Test Document"));
        assert_eq!(restored.author.as_deref(), Some("Author Name"));
        assert_eq!(restored.creator.as_deref(), Some("easyofd"));
        assert_eq!(restored.max_unit_id, 42);
        assert_eq!(restored.doc_usage.as_deref(), Some("Normal"));
    }

    #[test]
    fn ofd_metadata_to_xml_children() {
        let meta = OfdMetadata {
            doc_id: Some("id1".to_string()),
            author: Some("auth".to_string()),
            ..Default::default()
        };
        let xml = meta.to_xml();
        assert!(xml.contains("<DocID>id1</DocID>"));
        assert!(xml.contains("<Author>auth</Author>"));
        assert!(xml.contains("<DocInfo"));
        assert!(xml.contains("</DocInfo>"));
    }
}
