//! 图片处理工具。
//!
//! 对应 Java: org.ofdrw.reader.tools.ImageUtils
//!
//! Java 版依赖 Apache PDFBox JBIG2 解码器和 AWT `BufferedImage`。
//! Rust 版提供纯数据层面的图片处理辅助函数，不依赖图形库。

/// 图片格式。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ImageFormat {
    /// PNG 格式。
    Png,
    /// JPEG 格式。
    Jpeg,
    /// BMP 格式。
    Bmp,
    /// TIFF 格式。
    Tiff,
    /// JB2 (JBIG2) 格式。
    Jb2,
    /// 未知格式。
    Unknown,
}

impl ImageFormat {
    /// 从文件扩展名推断图片格式。
    #[must_use]
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "png" => Self::Png,
            "jpg" | "jpeg" => Self::Jpeg,
            "bmp" => Self::Bmp,
            "tiff" | "tif" => Self::Tiff,
            "jb2" | "gbig2" => Self::Jb2,
            _ => Self::Unknown,
        }
    }

    /// 从文件路径推断图片格式。
    #[must_use]
    pub fn from_path(path: &str) -> Self {
        let ext = path.rsplit('.').next().unwrap_or("");
        Self::from_extension(ext)
    }

    /// 返回格式的 MIME 类型字符串。
    #[must_use]
    pub fn mime_type(&self) -> &'static str {
        match self {
            Self::Png => "image/png",
            Self::Jpeg => "image/jpeg",
            Self::Bmp => "image/bmp",
            Self::Tiff => "image/tiff",
            Self::Jb2 => "image/x-jbig2",
            Self::Unknown => "application/octet-stream",
        }
    }

    /// 返回格式的常用文件扩展名。
    #[must_use]
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Jpeg => "jpeg",
            Self::Bmp => "bmp",
            Self::Tiff => "tiff",
            Self::Jb2 => "jb2",
            Self::Unknown => "bin",
        }
    }
}

/// 检查图片数据是否具有有效的文件头签名。
///
/// 对应 Java: `ImageUtils` 中的格式判断逻辑
#[must_use]
pub fn detect_format(data: &[u8]) -> ImageFormat {
    if data.len() < 4 {
        return ImageFormat::Unknown;
    }
    // PNG: 89 50 4E 47
    if data.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
        return ImageFormat::Png;
    }
    // JPEG: FF D8 FF
    if data.starts_with(&[0xFF, 0xD8, 0xFF]) {
        return ImageFormat::Jpeg;
    }
    // BMP: 42 4D
    if data.starts_with(&[0x42, 0x4D]) {
        return ImageFormat::Bmp;
    }
    // TIFF: 49 49 2A 00 (little-endian) or 4D 4D 00 2A (big-endian)
    if data.starts_with(&[0x49, 0x49, 0x2A, 0x00]) || data.starts_with(&[0x4D, 0x4D, 0x00, 0x2A]) {
        return ImageFormat::Tiff;
    }
    // JBIG2: 97 4A 42 32
    if data.starts_with(&[0x97, 0x4A, 0x42, 0x32]) {
        return ImageFormat::Jb2;
    }
    ImageFormat::Unknown
}

/// 计算灰度值。
///
/// 对应 Java: `ImageUtils.gray(int r, int g, int b)`
#[must_use]
pub fn gray(r: u32, g: u32, b: u32) -> u32 {
    (r * 19_595 + g * 38_469 + b * 7_472) >> 16
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_format_from_extension() {
        assert_eq!(ImageFormat::from_extension("png"), ImageFormat::Png);
        assert_eq!(ImageFormat::from_extension("JPEG"), ImageFormat::Jpeg);
        assert_eq!(ImageFormat::from_extension("jpg"), ImageFormat::Jpeg);
        assert_eq!(ImageFormat::from_extension("bmp"), ImageFormat::Bmp);
        assert_eq!(ImageFormat::from_extension("tiff"), ImageFormat::Tiff);
        assert_eq!(ImageFormat::from_extension("tif"), ImageFormat::Tiff);
        assert_eq!(ImageFormat::from_extension("jb2"), ImageFormat::Jb2);
        assert_eq!(ImageFormat::from_extension("xyz"), ImageFormat::Unknown);
    }

    #[test]
    fn test_image_format_from_path() {
        assert_eq!(
            ImageFormat::from_path("/doc/res/photo.jpg"),
            ImageFormat::Jpeg
        );
        assert_eq!(ImageFormat::from_path("image.PNG"), ImageFormat::Png);
    }

    #[test]
    fn test_image_format_mime_type() {
        assert_eq!(ImageFormat::Png.mime_type(), "image/png");
        assert_eq!(ImageFormat::Jpeg.mime_type(), "image/jpeg");
    }

    #[test]
    fn test_image_format_extension() {
        assert_eq!(ImageFormat::Png.extension(), "png");
        assert_eq!(ImageFormat::Jpeg.extension(), "jpeg");
    }

    #[test]
    fn test_detect_format_png() {
        let data = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        assert_eq!(detect_format(&data), ImageFormat::Png);
    }

    #[test]
    fn test_detect_format_jpeg() {
        let data = [0xFF, 0xD8, 0xFF, 0xE0];
        assert_eq!(detect_format(&data), ImageFormat::Jpeg);
    }

    #[test]
    fn test_detect_format_bmp() {
        let data = [0x42, 0x4D, 0x00, 0x00];
        assert_eq!(detect_format(&data), ImageFormat::Bmp);
    }

    #[test]
    fn test_detect_format_tiff_le() {
        let data = [0x49, 0x49, 0x2A, 0x00];
        assert_eq!(detect_format(&data), ImageFormat::Tiff);
    }

    #[test]
    fn test_detect_format_tiff_be() {
        let data = [0x4D, 0x4D, 0x00, 0x2A];
        assert_eq!(detect_format(&data), ImageFormat::Tiff);
    }

    #[test]
    fn test_detect_format_jb2() {
        let data = [0x97, 0x4A, 0x42, 0x32];
        assert_eq!(detect_format(&data), ImageFormat::Jb2);
    }

    #[test]
    fn test_detect_format_unknown() {
        let data = [0x00, 0x01, 0x02, 0x03];
        assert_eq!(detect_format(&data), ImageFormat::Unknown);
    }

    #[test]
    fn test_detect_format_short_data() {
        assert_eq!(detect_format(&[0x89]), ImageFormat::Unknown);
        assert_eq!(detect_format(&[]), ImageFormat::Unknown);
    }

    #[test]
    fn test_gray() {
        // 纯白应返回 255
        let white = gray(255, 255, 255);
        assert_eq!(white, 255);
        // 纯黑应返回 0
        let black = gray(0, 0, 0);
        assert_eq!(black, 0);
    }
}
