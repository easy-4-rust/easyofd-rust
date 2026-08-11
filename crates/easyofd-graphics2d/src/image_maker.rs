//! 图片到 OFD `ImageObject` 转换器。
//!
//! 对应 Java 版 `ofdrw-graphics2d` 中的图片处理工具，
//! 将原始图片字节转为 [`ImageObject`]，支持格式检测与尺寸推断。

use easyofd_core::model::{ImageFormat, ImageObject};

/// 图片到 OFD `ImageObject` 转换器。
///
/// 提供从原始字节构建 [`ImageObject`] 的便捷方法，
/// 支持 JPEG / PNG / BMP / TIFF 格式自动检测。
#[derive(Debug, Clone, Copy)]
pub struct ImageMaker;

impl ImageMaker {
    /// 从原始字节创建 `ImageObject`，自动检测图片格式。
    ///
    /// # 参数
    /// - `x`, `y`：图片左上角坐标（mm）
    /// - `width`, `height`：图片显示尺寸（mm）
    /// - `data`：图片原始字节
    #[must_use]
    pub fn from_bytes(x: f64, y: f64, width: f64, height: f64, data: &[u8]) -> ImageObject {
        let format = Self::detect_format(data);
        ImageObject::new(x, y, width, height, data.to_vec(), format)
    }

    /// 从原始字节创建 JPEG `ImageObject`。
    #[must_use]
    pub fn jpeg(x: f64, y: f64, width: f64, height: f64, data: &[u8]) -> ImageObject {
        ImageObject::jpeg(x, y, width, height, data.to_vec())
    }

    /// 从原始字节创建 PNG `ImageObject`。
    #[must_use]
    pub fn png(x: f64, y: f64, width: f64, height: f64, data: &[u8]) -> ImageObject {
        ImageObject::png(x, y, width, height, data.to_vec())
    }

    /// 从原始字节创建 BMP `ImageObject`。
    #[must_use]
    pub fn bmp(x: f64, y: f64, width: f64, height: f64, data: &[u8]) -> ImageObject {
        ImageObject::new(x, y, width, height, data.to_vec(), ImageFormat::Bmp)
    }

    /// 从原始字节创建 TIFF `ImageObject`。
    #[must_use]
    pub fn tiff(x: f64, y: f64, width: f64, height: f64, data: &[u8]) -> ImageObject {
        ImageObject::new(x, y, width, height, data.to_vec(), ImageFormat::Tiff)
    }

    /// 根据魔术字节检测图片格式。
    ///
    /// 检测顺序：JPEG → PNG → BMP → TIFF → 默认 JPEG。
    #[must_use]
    pub fn detect_format(data: &[u8]) -> ImageFormat {
        if data.len() >= 2 && data[0] == 0xFF && data[1] == 0xD8 {
            return ImageFormat::Jpeg;
        }
        if data.len() >= 4
            && data[0] == 0x89
            && data[1] == b'P'
            && data[2] == b'N'
            && data[3] == b'G'
        {
            return ImageFormat::Png;
        }
        if data.len() >= 2 && data[0] == b'B' && data[1] == b'M' {
            return ImageFormat::Bmp;
        }
        if data.len() >= 4
            && ((data[0] == b'I' && data[1] == b'I') || (data[0] == b'M' && data[1] == b'M'))
            && data[2] == 0x00
            && data[3] == 0x2A
        {
            return ImageFormat::Tiff;
        }
        // 默认 JPEG
        ImageFormat::Jpeg
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_jpeg() {
        let data = [0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10];
        assert_eq!(ImageMaker::detect_format(&data), ImageFormat::Jpeg);
    }

    #[test]
    fn test_detect_png() {
        let data = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
        assert_eq!(ImageMaker::detect_format(&data), ImageFormat::Png);
    }

    #[test]
    fn test_detect_bmp() {
        let data = [b'B', b'M', 0x00, 0x00];
        assert_eq!(ImageMaker::detect_format(&data), ImageFormat::Bmp);
    }

    #[test]
    fn test_detect_tiff_le() {
        let data = [b'I', b'I', 0x00, 0x2A, 0x00, 0x00];
        assert_eq!(ImageMaker::detect_format(&data), ImageFormat::Tiff);
    }

    #[test]
    fn test_detect_tiff_be() {
        let data = [b'M', b'M', 0x00, 0x2A, 0x00, 0x00];
        assert_eq!(ImageMaker::detect_format(&data), ImageFormat::Tiff);
    }

    #[test]
    fn test_detect_unknown_defaults_to_jpeg() {
        let data = [0x00, 0x01, 0x02, 0x03];
        assert_eq!(ImageMaker::detect_format(&data), ImageFormat::Jpeg);
    }

    #[test]
    fn test_from_bytes_auto_detect() {
        let png_data = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
        let img = ImageMaker::from_bytes(10.0, 20.0, 100.0, 200.0, &png_data);
        assert_eq!(img.format, ImageFormat::Png);
        assert!((img.x - 10.0).abs() < f64::EPSILON);
        assert!((img.y - 20.0).abs() < f64::EPSILON);
        assert!((img.width - 100.0).abs() < f64::EPSILON);
        assert!((img.height - 200.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_jpeg_constructor() {
        let data = [0xFF, 0xD8, 0xFF];
        let img = ImageMaker::jpeg(0.0, 0.0, 10.0, 10.0, &data);
        assert_eq!(img.format, ImageFormat::Jpeg);
        assert_eq!(img.data, data);
    }

    #[test]
    fn test_png_constructor() {
        let data = [0x89, b'P', b'N', b'G'];
        let img = ImageMaker::png(0.0, 0.0, 10.0, 10.0, &data);
        assert_eq!(img.format, ImageFormat::Png);
    }

    #[test]
    fn test_bmp_constructor() {
        let data = *b"BM";
        let img = ImageMaker::bmp(0.0, 0.0, 10.0, 10.0, &data);
        assert_eq!(img.format, ImageFormat::Bmp);
    }

    #[test]
    fn test_tiff_constructor() {
        let data = [b'I', b'I', 0x00, 0x2A];
        let img = ImageMaker::tiff(0.0, 0.0, 10.0, 10.0, &data);
        assert_eq!(img.format, ImageFormat::Tiff);
    }
}
