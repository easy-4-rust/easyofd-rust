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
//!
//! ## 新增类型（移植自 ofdrw）
//!
//! - 枚举：[`Clear`]、[`Display`]、[`AFloat`]、[`VerticalAlign`]
//! - 画布：[`Line`]、[`CanvasBase`]、[`CanvasGradient`]、[`CanvasRadialGradient`]、[`CanvasPattern`]、[`CanvasState`]
//! - 字体：[`FontSetting`]、[`Direction`]、[`NamedColor`]
//! - 文本：[`TextMetrics`]、[`MeasureBody`]、[`TextMetricsArea`]、[`TxtGlyph`]、[`TxtLineBlock`]
//! - 元素：[`BR`]、[`PlaceholderSpan`]、[`ArtWord`]、[`AreaHolderBlock`]、[`PageAreaFiller`]
//! - 绘制器：[`Drawer`]、[`CellContentDrawer`]
//! - 回调接口：[`TextFontInfo`]、[`DivContainer`]、[`Processor`]、[`RenderFinishHandler`]、[`VPageHandler`]、[`ElementRenderFinishHandler`]、[`ElementSplit`]、[`RenderPrepare`]
//! - 错误类型：[`RenderException`]、[`DocReadException`]
//! - 工具：[`ArrayParamTool`]、[`TextMeasureTool`]、[`GraphHelper`]
//! - 其他：[`AdditionVPage`]、[`WatermarkDrawer`]、[`ExistCtFont`]、[`StreamCollect`]、[`DocContentReplace`]、[`AnnotationRender`]

// ── 已有模块 ──────────────────────────────────────────────────────────────────
mod annotation;
mod attachment;
mod border;
mod div;
mod img;
mod layout_analyzer;
mod layout_block;
mod layout_options;
mod layout_result;
mod paragraph;
mod position;
mod rectangle;
mod segment_engine;
mod span;
mod streaming_layout;
mod vpage_parser;
mod xycut;

// ── 新增：枚举 ────────────────────────────────────────────────────────────────
mod a_float;
mod clear;
mod display;
mod vertical_align;

// ── 新增：画布 ────────────────────────────────────────────────────────────────
mod canvas_base;
mod canvas_gradient;
mod canvas_pattern;
mod canvas_radial_gradient;
mod canvas_state;
mod line;

// ── 新增：字体 / 颜色 ─────────────────────────────────────────────────────────
mod font_setting;
mod named_color;

// ── 新增：文本度量 ─────────────────────────────────────────────────────────────
mod measure_body;
mod text_metrics;
mod text_metrics_area;
mod txt_glyph;
mod txt_line_block;

// ── 新增：元素 ─────────────────────────────────────────────────────────────────
mod area_holder_block;
mod art_word;
mod br;
mod page_area_filler;
mod placeholder_span;

// ── 新增：绘制器 ───────────────────────────────────────────────────────────────
mod cell_content_drawer;

// ── 新增：接口 trait ───────────────────────────────────────────────────────────
mod div_container;
mod drawer;
mod element_render_finish_handler;
mod element_split;
mod processor;
mod render_finish_handler;
mod render_prepare;
mod text_font_info;
mod v_page_handler;

// ── 新增：错误类型 ─────────────────────────────────────────────────────────────
mod doc_read_exception;
mod render_exception;

// ── 新增：工具 ─────────────────────────────────────────────────────────────────
mod addition_v_page;
mod annotation_render;
mod array_param_tool;
mod doc_content_replace;
mod exist_ct_font;
mod graph_helper;
mod stream_collect;
mod text_measure_tool;
mod watermark_drawer;

// ── 已有 pub use ───────────────────────────────────────────────────────────────
pub use annotation::{AnnotType, Annotation};
pub use attachment::Attachment;
pub use border::Border;
pub use div::{Div, DivContent, TextStyle};
pub use img::Img;
pub use layout_analyzer::LayoutAnalyzer;
pub use layout_block::LayoutBlock;
pub use layout_options::LayoutOptions;
pub use layout_result::LayoutResult;
pub use paragraph::{Paragraph, TextAlign};
pub use position::Position;
pub use rectangle::Rectangle;
pub use segment_engine::{Segment, SegmentationEngine};
pub use span::Span;
pub use streaming_layout::{StreamingLayoutAnalyzer, VirtualPage};
pub use vpage_parser::{VPageParseEngine, div_to_content_objects};
pub use xycut::{Region, XyCutOptions, xycut};

// ── 新增 pub use：枚举 ─────────────────────────────────────────────────────────
pub use a_float::AFloat;
pub use clear::Clear;
pub use display::Display;
pub use vertical_align::VerticalAlign;

// ── 新增 pub use：画布 ─────────────────────────────────────────────────────────
pub use canvas_base::CanvasBase;
pub use canvas_gradient::{CanvasGradient, ColorStop};
pub use canvas_pattern::{CanvasPattern, RepeatMode};
pub use canvas_radial_gradient::CanvasRadialGradient;
pub use canvas_state::CanvasState;
pub use line::Line;

// ── 新增 pub use：字体 / 颜色 ───────────────────────────────────────────────────
pub use font_setting::{Direction, FontSetting};
pub use named_color::NamedColor;

// ── 新增 pub use：文本度量 ───────────────────────────────────────────────────────
pub use measure_body::MeasureBody;
pub use text_metrics::TextMetrics;
pub use text_metrics_area::{CharArea, TextMetricsArea};
pub use txt_glyph::TxtGlyph;
pub use txt_line_block::TxtLineBlock;

// ── 新增 pub use：元素 ───────────────────────────────────────────────────────────
pub use area_holder_block::AreaHolderBlock;
pub use art_word::ArtWord;
pub use br::BR;
pub use page_area_filler::PageAreaFiller;
pub use placeholder_span::PlaceholderSpan;

// ── 新增 pub use：绘制器 ─────────────────────────────────────────────────────────
pub use cell_content_drawer::{CellContentDrawer, CellImage};

// ── 新增 pub use：接口 trait ─────────────────────────────────────────────────────
pub use div_container::DivContainer;
pub use drawer::{Drawer, FnDrawer};
pub use element_render_finish_handler::ElementRenderFinishHandler;
pub use element_split::ElementSplit;
pub use processor::Processor;
pub use render_finish_handler::{FnRenderFinishHandler, RenderFinishHandler};
pub use render_prepare::RenderPrepare;
pub use text_font_info::TextFontInfo;
pub use v_page_handler::VPageHandler;

// ── 新增 pub use：错误类型 ───────────────────────────────────────────────────────
pub use doc_read_exception::DocReadException;
pub use render_exception::RenderException;

// ── 新增 pub use：工具 ───────────────────────────────────────────────────────────
pub use addition_v_page::AdditionVPage;
pub use annotation_render::AnnotationRender;
pub use array_param_tool::ArrayParamTool;
pub use doc_content_replace::DocContentReplace;
pub use exist_ct_font::ExistCtFont;
pub use graph_helper::GraphHelper;
pub use stream_collect::StreamCollect;
pub use text_measure_tool::TextMeasureTool;
pub use watermark_drawer::{WatermarkDrawer, WatermarkRotation};
