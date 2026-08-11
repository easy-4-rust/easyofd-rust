//! 页面描述类型 (GB/T 33190 第8章)
//!
//! 对应 Java: org.ofdrw.core.pageDescription

#![allow(non_camel_case_types)]
#![allow(clippy::should_implement_trait)]
#![allow(clippy::useless_format)]
#![allow(clippy::many_single_char_names)]

pub mod area;
pub mod clips;
pub mod color;
pub mod ct_region;
pub mod draw_param;
pub mod graphic_transform;
pub mod res;

pub use area::Area;
pub use clips::CT_Clip;
pub use color::{
    CT_AxialShd, CT_Color, CT_ColorSpace, CT_GouraudShd, CT_LaGouraudShd, CT_Pattern, CT_RadialShd,
};
pub use ct_region::CT_Region;
pub use draw_param::CT_DrawParam;
pub use graphic_transform::CT_CGTransform;
pub use res::{
    CT_MultiMedia, ColorSpaces, CompositeGraphicUnits, DrawParams, Fonts, MediaType, MultiMedias,
};
