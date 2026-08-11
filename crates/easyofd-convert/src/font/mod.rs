//! 字体解析类型模块。
//!
//! 对应 Java: org.ofdrw.converter.font
//!
//! 包含 OFD 转换过程中需要的字体表解析类型（NameRecord、NamingTable、
//! CmapSubtable、GlyphData 等），以及字形描述符（GlyfDescript 等）。
//! 这些类型用于字体子集化、字形查找和坐标变换。

pub mod cmap_subtable;
pub mod font_draw_path_provider;
pub mod font_loader;
pub mod font_utils;
pub mod font_wrapper;
pub mod glyf_composite_comp;
pub mod glyf_composite_descript;
pub mod glyf_descript;
pub mod glyf_simple_descript;
pub mod glyph_data;
pub mod glyph_data_provider;
pub mod horizontal_header_table;
pub mod horizontal_metrics_table;
pub mod memory_ttf_data_stream;
pub mod name_record;
pub mod naming_table;
pub mod pdf_font_wrapper;
pub mod ttf_data_stream_alias;
pub mod type1_seg_split_parser;

pub use cmap_subtable::CmapSubtable;
pub use font_draw_path_provider::{FontDrawPathProvider, GlyphPath, GlyphPoint};
pub use font_loader::FontLoader;

/// 对应 Java: `org.ofdrw.converter.utils.FontUtils`
///
/// 工具类以模块级函数形式实现，见 [`font_utils`] 模块。
pub use font_utils as FontUtils;
pub use font_wrapper::FontWrapper;
pub use glyf_composite_comp::GlyfCompositeComp;
pub use glyf_composite_descript::GlyfCompositeDescript;
pub use glyf_descript::GlyfDescript;
pub use glyf_simple_descript::GlyfSimpleDescript;
pub use glyph_data::{BoundingBox, GlyphData};
pub use glyph_data_provider::GlyphDataProvider;
pub use horizontal_header_table::HorizontalHeaderTable;
pub use horizontal_metrics_table::HorizontalMetricsTable;
pub use memory_ttf_data_stream::MemoryTTFDataStream;
pub use name_record::NameRecord;
pub use naming_table::NamingTable;
pub use pdf_font_wrapper::PdfFontWrapper;
pub use type1_seg_split_parser::{Type1Seg, Type1SegSplitParser, Type1SegType};

// ── ofdrw Java 模块别名 ──

/// 对应 Java: `org.ofdrw.converter.font.TTFDataStream`
///
/// 复用 [`easyofd_font::ttf_data_stream::TtfDataStream`]，此别名保持 Java 名称兼容。
pub use ttf_data_stream_alias::TTFDataStream;
