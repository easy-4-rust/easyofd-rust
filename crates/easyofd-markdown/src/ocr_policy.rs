/// 图片 OCR 的触发策略。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OcrPolicy {
    /// 完全禁用 OCR。
    #[default]
    Disabled,
    /// 仅当页面没有嵌入文本时识别页面图片。
    WhenPageHasNoText,
    /// 识别每个有图片字节的对象。
    AllImages,
}
