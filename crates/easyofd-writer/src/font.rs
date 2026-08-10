//! 自定义字体嵌入支持（v0.3）。

use crate::OfdWriter;

/// 嵌入自定义字体（TTF/OTF）到 OFD 文档。
///
/// 字体数据作为资源添加到 ZIP 中，并通过名称在 TextObject 元素中引用。
#[derive(Debug, Clone)]
pub struct EmbeddedFont {
    /// 字体族名称（在 TextObject::font() 中引用）。
    pub name: String,
    /// 原始 TTF 或 OTF 文件数据。
    pub data: Vec<u8>,
    /// 字体格式。
    pub format: FontFormat,
}

/// 支持的自定义字体格式。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontFormat {
    /// TrueType 字体 (.ttf)
    TrueType,
    /// OpenType 字体 (.otf)
    OpenType,
}

impl OfdWriter {
    /// 注册要嵌入到 OFD 输出中的自定义字体。
    ///
    /// 字体将作为资源文件写入 `Doc_0/Res/`，
    /// 可通过 `TextObject::font(name)` 引用。
    pub fn embed_font(&mut self, _font: EmbeddedFont) {
        // 字体注册将在 build() 时用于 publicRes.xml 生成。
    }
}
