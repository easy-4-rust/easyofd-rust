//! 文本抽取相关类型。
//!
//! 对应 Java: org.ofdrw.reader.extractor

mod extractor_filter;
mod region_text_extractor_filter;

pub use extractor_filter::{ExtractorFilter, FilterDecision, RectFilter};
pub use region_text_extractor_filter::RegionTextExtractorFilter;
