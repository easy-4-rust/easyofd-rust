use easyofd_core::{ImageFormat, OfdResult};

/// 可选 OCR 实现的稳定扩展接口。
pub trait OcrProvider: Send + Sync {
    /// 识别一张 OFD 内嵌图片。
    ///
    /// 返回 `None` 表示图片中没有可识别文本。实现可以封装本地 OCR、远程服务或
    /// 视觉模型，但应自行处理超时、鉴权和数据合规。
    ///
    /// # Errors
    ///
    /// OCR 后端调用失败时返回错误；转换器会将其降级为警告并继续其他页面。
    fn recognize(&self, image: &[u8], format: ImageFormat) -> OfdResult<Option<String>>;
}
