//! 图片格式枚举。

/// 支持的图片格式。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    /// JPEG 图片。
    Jpeg,
    /// PNG 图片。
    Png,
    /// BMP 图片。
    Bmp,
    /// TIFF 图片。
    Tiff,
}
