//! 图片格式转换枚举。

/// 图片格式转换辅助。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageConvertFormat {
    /// JPEG 格式。
    Jpeg,
    /// PNG 格式。
    Png,
    /// BMP 格式。
    Bmp,
}
