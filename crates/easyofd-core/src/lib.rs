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
pub mod compat;
pub mod composite_obj;
pub mod consts;
pub mod crypto;
pub mod custom_tags;
pub mod doc;
pub mod doc_vpreferences;
pub mod error;
pub mod extensions;
pub mod graph;
pub mod graphics2d;
pub mod holder;
pub mod image;
pub mod integrity;
pub mod model;
pub mod ofd_common_qname;
pub mod ofd_element;
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

// ── 新增类型导出 ──────────────────────────────────────────────────────────

pub use crypto::{
    CryptoParameter, DecyptSeed, Encryptions, ExtendParams, SigParameter, SigParameters, UserInfo,
};
pub use graphics2d::{
    GraphicsDeviceType, OfdGraphics2DDrawParam, OfdGraphicsDocument, OfdPageGraphics2D,
    OfdPageGraphicsConfiguration, OfdPageGraphicsDevice, OfdShape, OfdShapes,
};
pub use ofd_common_qname::OfdCommonQName;
pub use ofd_element::{DefaultElementProxy, OfdElement, OfdSimpleTypeElement};

// ── 兼容别名（Java 类名 → Rust 类型名）────────────────────────────────────
// 不用 wildcard，已在 compat 模块中逐个 pub use。详见 compat 模块文档。

/// 对应 Java: OFDCommonQName（Rust 命名别名）。
pub type OFDCommonQName = OfdCommonQName;

/// 对应 Java: OFDGraphicsDocument（Rust 命名别名）。
pub type OFDGraphicsDocument = OfdGraphicsDocument;

/// 对应 Java: OFDGraphics2DDrawParam（Rust 命名别名）。
pub type OFDGraphics2DDrawParam = OfdGraphics2DDrawParam;

/// 对应 Java: OFDPageGraphics2D（Rust 命名别名）。
pub type OFDPageGraphics2D = OfdPageGraphics2D;

/// 对应 Java: OFDPageGraphicsConfiguration（Rust 命名别名）。
pub type OFDPageGraphicsConfiguration = OfdPageGraphicsConfiguration;

/// 对应 Java: OFDPageGraphicsDevice（Rust 命名别名）。
pub type OFDPageGraphicsDevice = OfdPageGraphicsDevice;

/// 对应 Java: OFDShapes（Rust 命名别名）。
pub type OFDShapes = OfdShapes;
