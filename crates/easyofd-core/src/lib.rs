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
pub mod composite_obj;
pub mod custom_tags;
pub mod doc;
pub mod error;
pub mod extensions;
pub mod model;
pub mod ofd_model;
pub mod versions;
pub mod watermark;

// Re-export core types at crate root for convenience.
pub use action::{
    Actions, Bookmark as ActionBookmark, CTAction, CTDest, DestType, EventType, Goto, GotoA, Movie,
    OfdAction, OfdGotoTarget, PlayType, Sound, URI,
};
pub use annotation::{AnnPage, Annot, AnnotType, Annotations, Appearance, PageAnnot};
pub use attachment::{Attachments, CTAttachment};
pub use composite_obj::{CT_Composite, CT_VectorG, Content};
pub use custom_tags::{CustomTag, CustomTags};
pub use doc::bookmark::{Bookmark, Bookmarks};
pub use doc::permission::{CtPermission, Print, ValidPeriod};
pub use error::{OfdError, OfdResult};
pub use extensions::{CtExtension, Extensions, Property};
pub use model::{
    ContentObject, ImageFormat, ImageObject, OfdMetadata, OfdPage, PathObject, TextObject,
    page_size,
};
pub use ofd_model::{OfdField, OfdFieldKind, OfdModel};
pub use versions::{DocVersion, File, FileList, Version, Versions};
pub use watermark::Watermark;
