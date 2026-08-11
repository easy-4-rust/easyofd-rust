//! 字段配置。

/// 单个 `#[ofd(...)]` 字段的解析配置。
pub(crate) struct FieldConfig {
    pub(crate) x: f64,
    pub(crate) y: f64,
    pub(crate) font: String,
    pub(crate) size: f64,
    pub(crate) weight: u32,
    pub(crate) italic: bool,
    pub(crate) color: u32,
    pub(crate) kind: String,
    pub(crate) img_width: f64,
    pub(crate) img_height: f64,
}
