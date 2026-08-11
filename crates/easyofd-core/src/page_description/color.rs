//! 颜色相关类型。
//!
//! 对应 Java: org.ofdrw.core.pageDescription.color

mod ct_axial_shd;
mod ct_color;
mod ct_color_space;
mod ct_gouraud_shd;
mod ct_la_gouraud_shd;
mod ct_pattern;
mod ct_radial_shd;

pub use ct_axial_shd::CT_AxialShd;
pub use ct_color::CT_Color;
pub use ct_color_space::CT_ColorSpace;
pub use ct_gouraud_shd::CT_GouraudShd;
pub use ct_la_gouraud_shd::CT_LaGouraudShd;
pub use ct_pattern::CT_Pattern;
pub use ct_radial_shd::CT_RadialShd;
