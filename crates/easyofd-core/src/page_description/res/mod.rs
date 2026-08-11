//! 资源类型模块。
//!
//! 对应 Java: org.ofdrw.core.basicStructure.res

mod color_spaces;
mod composite_graphic_units;
mod ct_multi_media;
mod draw_params;
mod fonts;
mod media_type;
mod multi_medias;

pub use color_spaces::ColorSpaces;
pub use composite_graphic_units::CompositeGraphicUnits;
pub use ct_multi_media::CT_MultiMedia;
pub use draw_params::DrawParams;
pub use fonts::Fonts;
pub use media_type::MediaType;
pub use multi_medias::MultiMedias;
