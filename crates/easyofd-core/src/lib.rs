//! # easyofd-core
//!
//! Core types, traits, and models for the easyofd-rust OFD document library.
//!
//! This crate provides:
//! - [`model`] — OFD data model types (`OfdPage`, `TextObject`, `ImageObject`, etc.)
//! - [`ofd_model`] — The [`OfdModel`] trait for mapping Rust types to OFD pages
//! - [`error`] — Error types ([`OfdError`])
//! - [`page_size`] — Common page size constants

pub mod action;
pub mod annotation;
pub mod attachment;
pub mod basic_type;
pub mod composite_obj;
pub mod crypto;
pub mod custom_tags;
pub mod doc;
pub mod doc_vpreferences;
pub mod error;
pub mod extensions;
pub mod graph;
pub mod holder;
pub mod image;
pub mod integrity;
pub mod model;
pub mod ofd_model;
pub mod page_description;
pub mod page_obj;
pub mod signatures;
pub mod text;
pub mod versions;
pub mod watermark;
pub mod xml_element;
mod xml_impls;
pub mod xml_parse;

// Re-export core types at crate root for convenience.
pub use xml_element::{XmlElement, XmlElementError, XmlNode, xml_escape};
pub use xml_parse::parse_xml_to_nodes;

// Re-export core types at crate root for convenience.
pub use action::{
    Actions, Bookmark as ActionBookmark, CTAction, CTDest, DestType, EventType, Goto, GotoA, Movie,
    OfdAction, OfdGotoTarget, PlayType, Sound, URI,
};
pub use annotation::{AnnPage, Annot, AnnotType, Annotations, Appearance, PageAnnot};
pub use attachment::{Attachments, CTAttachment};
pub use basic_type::{ST_Array, ST_Box, ST_ID, ST_Loc, ST_Pos, ST_RefID};
pub use composite_obj::{CT_Composite, CT_VectorG, Content};
pub use custom_tags::{CustomTag, CustomTags};
pub use doc::bookmark::{Bookmark, Bookmarks};
pub use doc::ct_doc_info::{CtDocInfo, DocUsage};
pub use doc::ct_v_preferences::{
    CtVPreferences, PageLayout, PageMode, TabDisplay, ZoomMode, ZoomScale,
};
pub use doc::doc_dir::DocDir;
pub use doc::ofd_dir::OfdDir;
pub use doc::permission::{CtPermission, Print, ValidPeriod};
pub use error::{OfdError, OfdResult};
pub use extensions::{CtExtension, Extensions, Property};
pub use graph::{AbbreviatedData, CT_Path, FillRule, PathCommand};
pub use model::{
    Bookmark as ModelBookmark, Bookmarks as ModelBookmarks, ContentObject, CreationDate, Creator,
    CustomData, CustomDatas, ImageFormat, ImageObject, OfdId, OfdMetadata, OfdPage, PathObject,
    Point, TextObject, Weight, page_size,
};
pub use ofd_model::{OfdField, OfdFieldKind, OfdModel};
pub use page_obj::{
    CT_CommonData, CT_GraphicUnit, CT_Layer, CT_PageArea, CT_PageBlock, CT_TemplatePage, LayerType,
    LineCapType, LineJoinType, TemplateZOrder,
};
pub use signatures::{
    Provider, Reference, References, Seal, SealImageType, SigType, Signature, Signatures,
    SignedInfo, StampAnnot, StampAnnotEntity,
};
pub use text::{CT_CGTransform, CT_Font, CT_Text, Direction, TextCode};
pub use versions::{DocVersion, File, FileList, Version, Versions};
pub use watermark::Watermark;
