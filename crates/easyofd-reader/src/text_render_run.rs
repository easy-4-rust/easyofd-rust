use easyofd_core::CT_Text;

/// 转换专用的文字绘制信息，不改变简化模型的公共字段。
#[derive(Debug, Clone)]
pub struct TextRenderRun {
    /// 在页面内容序列中的索引。
    pub content_index: usize,
    /// 原始文字样式、定位与字形映射。
    pub text: CT_Text,
    /// 对象到页面的仿射变换。
    pub ctm: Option<[f64; 6]>,
    /// 有效描边线宽，毫米。
    pub line_width_mm: f64,
    /// 字体名称，保留资源引用解析结果。
    pub font_name: Option<String>,
    /// 嵌入字体数据；无嵌入字体时由后端选择字体。
    pub font_data: Option<Vec<u8>>,
}

impl TextRenderRun {
    /// 判断是否来自既有简化写入模型；其Font为名称而非OFD资源ID，字号单位为pt。
    /// 返回true时后端保留既有模型的坐标和字号契约。
    pub fn uses_legacy_model(&self) -> bool {
        self.text.font_ref.is_none()
            && self.font_data.is_none()
            && self.ctm.is_none()
            && !self.text.stroke
            && self.text.fill
            && self.text.cg_transforms.is_empty()
            && self.text.text_codes.len() <= 1
            && self
                .text
                .text_codes
                .iter()
                .all(|c| c.delta_x.is_empty() && c.delta_y.is_empty())
    }
}
