//! 颜色相关类型。
//!
//! 对应 Java: org.ofdrw.core.pageDescription.color

mod bits_per_component;
mod ct_axial_shd;
mod ct_color;
mod ct_color_space;
mod ct_gouraud_shd;
mod ct_la_gouraud_shd;
mod ct_pattern;
mod ct_radial_shd;
mod cv;
mod edge_flag;
mod fill_color;
mod palette;
mod reflect_method;
mod relative_to;
mod stroke_color;

pub use bits_per_component::BitsPerComponent;
pub use ct_axial_shd::CT_AxialShd;
pub use ct_color::CT_Color;
pub use ct_color_space::CT_ColorSpace;
pub use ct_gouraud_shd::CT_GouraudShd;
pub use ct_la_gouraud_shd::CT_LaGouraudShd;
pub use ct_pattern::CT_Pattern;
pub use ct_radial_shd::CT_RadialShd;
pub use cv::CV;
pub use edge_flag::EdgeFlag;
pub use fill_color::FillColor;
pub use palette::Palette;
pub use reflect_method::ReflectMethod;
pub use relative_to::RelativeTo;
pub use stroke_color::StrokeColor;
