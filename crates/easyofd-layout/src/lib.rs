//! OFD 页面几何布局分析。
//!
//! 默认分析完全确定，不依赖 OCR 或大模型，用于恢复基本阅读顺序、文本行和标题。

mod layout_analyzer;
mod layout_block;
mod layout_options;
mod layout_result;

pub use layout_analyzer::LayoutAnalyzer;
pub use layout_block::LayoutBlock;
pub use layout_options::LayoutOptions;
pub use layout_result::LayoutResult;
