//! OFD 页面几何布局分析。
//!
//! 默认分析完全确定，不依赖 OCR 或大模型，用于恢复基本阅读顺序、文本行和标题。
//!
//! ## 模块结构
//!
//! - [`layout_analyzer`] — 基于几何坐标的阅读顺序恢复。
//! - [`div`] — Div 盒式模型（CSS box model 思想）。
//! - [`xycut`] — XY-cut 页面分割算法（多栏 / 表格识别）。
//! - [`segment_engine`] — 流式版面分段引擎（SegmentationEngine）。
//! - [`streaming_layout`] — 流式布局分析器（StreamingLayoutAnalyzer）。
//! - [`vpage_parser`] — 虚拟页面 → OFD XML 转换引擎（VPageParseEngine）。

mod border;
mod div;
mod layout_analyzer;
mod layout_block;
mod layout_options;
mod layout_result;
mod position;
mod rectangle;
mod segment_engine;
mod span;
mod streaming_layout;
mod vpage_parser;
mod xycut;

pub use border::Border;
pub use div::{Div, DivContent, TextStyle};
pub use layout_analyzer::LayoutAnalyzer;
pub use layout_block::LayoutBlock;
pub use layout_options::LayoutOptions;
pub use layout_result::LayoutResult;
pub use position::Position;
pub use rectangle::Rectangle;
pub use segment_engine::{Segment, SegmentationEngine};
pub use span::Span;
pub use streaming_layout::{StreamingLayoutAnalyzer, VirtualPage};
pub use vpage_parser::{VPageParseEngine, div_to_content_objects};
pub use xycut::{Region, XyCutOptions, xycut};
