//! 字体解析类型模块。
//!
//! 对应 Java: org.ofdrw.converter.font
//!
//! 包含 OFD 转换过程中需要的字体表解析类型（NameRecord、NamingTable、
//! CmapSubtable、GlyphData 等），以及字形描述符（GlyfDescript 等）。
//! 这些类型用于字体子集化、字形查找和坐标变换。

pub mod cmap_subtable;
pub mod font_wrapper;
pub mod glyf_composite_comp;
pub mod glyf_composite_descript;
pub mod glyf_descript;
pub mod glyf_simple_descript;
pub mod glyph_data;
pub mod glyph_data_provider;
pub mod horizontal_header_table;
pub mod horizontal_metrics_table;
pub mod name_record;
pub mod naming_table;

pub use cmap_subtable::CmapSubtable;
pub use font_wrapper::FontWrapper;
pub use glyf_composite_comp::GlyfCompositeComp;
pub use glyf_composite_descript::GlyfCompositeDescript;
pub use glyf_descript::GlyfDescript;
pub use glyf_simple_descript::GlyfSimpleDescript;
pub use glyph_data::{BoundingBox, GlyphData};
pub use glyph_data_provider::GlyphDataProvider;
pub use horizontal_header_table::HorizontalHeaderTable;
pub use horizontal_metrics_table::HorizontalMetricsTable;
pub use name_record::NameRecord;
pub use naming_table::NamingTable;
