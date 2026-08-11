//! 页面对象子包 (pageObj)。
//!
//! 对应 GB/T 33190-2016 第 7.7 节，包含页面块、图层、模板页、
//! 图形单元、页面区域和文档公共数据等类型。

mod ct_common_data;
mod ct_graphic_unit;
mod ct_layer;
mod ct_page_area;
mod ct_page_block;
mod ct_template_page;

pub use ct_common_data::CT_CommonData;
pub use ct_graphic_unit::{CT_GraphicUnit, LineCapType, LineJoinType};
pub use ct_layer::{CT_Layer, LayerType};
pub use ct_page_area::{Box as PageAreaBox, CT_PageArea};
pub use ct_page_block::{
    CT_PageBlock, PageBlockImageObject, PageBlockPathObject, PageBlockTextObject,
};
pub use ct_template_page::{CT_TemplatePage, TemplateZOrder};
