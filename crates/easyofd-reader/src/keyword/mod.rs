//! 关键字搜索相关类型。
//!
//! 对应 Java: org.ofdrw.reader.keyword

mod keyword_extractor;
mod keyword_position;
mod keyword_resource;

pub use keyword_extractor::KeywordExtractor;
pub use keyword_position::KeywordPosition;
pub use keyword_resource::KeywordResource;
