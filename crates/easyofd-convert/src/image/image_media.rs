//! 图片媒体数据。
//!
//! 对应 Java: org.ofdrw.converter.image.ImageMedia
//!
//! Java 版 `ImageMedia` 封装图片的元信息（尺寸、格式、DPI）和原始数据。
//! Rust 版提供等价的数据结构。

/// 图片格式。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum MediaImageFormat {
    /// PNG 格式。
    Png,
    /// JPEG 格式。
    Jpeg,
    /// BMP 格式。
    Bmp,
    /// TIFF 格式。
    Tiff,
    /// GIF 格式。
    Gif,
    /// 未知格式。
    Unknown,
}

impl MediaImageFormat {
    /// 从文件扩展名推断格式。
    #[must_use]
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "png" => Self::Png,
            "jpg" | "jpeg" => Self::Jpeg,
            "bmp" => Self::Bmp,
            "tiff" | "tif" => Self::Tiff,
            "gif" => Self::Gif,
            _ => Self::Unknown,
        }
    }

    /// 返回格式的 MIME 类型。
    #[must_use]
    pub fn mime_type(&self) -> &'static str {
        match self {
            Self::Png => "image/png",
            Self::Jpeg => "image/jpeg",
            Self::Bmp => "image/bmp",
            Self::Tiff => "image/tiff",
            Self::Gif => "image/gif",
            Self::Unknown => "application/octet-stream",
        }
    }

    /// 返回格式的文件扩展名。
    #[must_use]
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Jpeg => "jpg",
            Self::Bmp => "bmp",
            Self::Tiff => "tiff",
            Self::Gif => "gif",
            Self::Unknown => "bin",
        }
    }
}

/// 图片媒体数据。
///
/// 对应 Java: `org.ofdrw.converter.image.ImageMedia`
///
/// 封装图片的原始数据和元信息。
#[derive(Debug, Clone)]
pub struct ImageMedia {
    /// 图片原始数据。
    pub data: Vec<u8>,
    /// 图片格式。
    pub format: MediaImageFormat,
    /// 图片宽度（像素）。
    pub width: u32,
    /// 图片高度（像素）。
    pub height: u32,
    /// 水平 DPI（每英寸点数）。
    pub dpi_x: f64,
    /// 垂直 DPI。
    pub dpi_y: f64,
}

impl ImageMedia {
    /// 创建图片媒体。
    #[must_use]
    pub fn new(data: Vec<u8>, format: MediaImageFormat, width: u32, height: u32) -> Self {
        Self {
            data,
            format,
            width,
            height,
            dpi_x: 96.0,
            dpi_y: 96.0,
        }
    }

    /// 设置 DPI。
    #[must_use]
    pub fn with_dpi(mut self, dpi_x: f64, dpi_y: f64) -> Self {
        self.dpi_x = dpi_x;
        self.dpi_y = dpi_y;
        self
    }

    /// 获取图片数据大小（字节）。
    #[must_use]
    pub fn data_size(&self) -> usize {
        self.data.len()
    }

    /// 判断图片数据是否为空。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// 从魔术字节推断格式并创建。
    #[must_use]
    pub fn from_data(data: Vec<u8>, width: u32, height: u32) -> Self {
        let format = if data.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
            MediaImageFormat::Png
        } else if data.starts_with(&[0xFF, 0xD8, 0xFF]) {
            MediaImageFormat::Jpeg
        } else if data.starts_with(&[0x42, 0x4D]) {
            MediaImageFormat::Bmp
        } else {
            MediaImageFormat::Unknown
        };
        Self::new(data, format, width, height)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_format_from_extension() {
        assert_eq!(
            MediaImageFormat::from_extension("png"),
            MediaImageFormat::Png
        );
        assert_eq!(
            MediaImageFormat::from_extension("JPEG"),
            MediaImageFormat::Jpeg
        );
        assert_eq!(
            MediaImageFormat::from_extension("bmp"),
            MediaImageFormat::Bmp
        );
        assert_eq!(
            MediaImageFormat::from_extension("tif"),
            MediaImageFormat::Tiff
        );
        assert_eq!(
            MediaImageFormat::from_extension("gif"),
            MediaImageFormat::Gif
        );
        assert_eq!(
            MediaImageFormat::from_extension("xyz"),
            MediaImageFormat::Unknown
        );
    }

    #[test]
    fn test_image_format_mime_type() {
        assert_eq!(MediaImageFormat::Png.mime_type(), "image/png");
        assert_eq!(MediaImageFormat::Jpeg.mime_type(), "image/jpeg");
    }

    #[test]
    fn test_image_format_extension() {
        assert_eq!(MediaImageFormat::Png.extension(), "png");
        assert_eq!(MediaImageFormat::Jpeg.extension(), "jpg");
    }

    #[test]
    fn test_image_media_new() {
        let media = ImageMedia::new(vec![0xFF], MediaImageFormat::Jpeg, 100, 200);
        assert_eq!(media.width, 100);
        assert_eq!(media.height, 200);
        assert_eq!(media.format, MediaImageFormat::Jpeg);
        assert!((media.dpi_x - 96.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_image_media_with_dpi() {
        let media =
            ImageMedia::new(vec![0xFF], MediaImageFormat::Png, 100, 200).with_dpi(300.0, 300.0);
        assert!((media.dpi_x - 300.0).abs() < f64::EPSILON);
        assert!((media.dpi_y - 300.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_image_media_data_size() {
        let media = ImageMedia::new(vec![0; 1024], MediaImageFormat::Png, 32, 32);
        assert_eq!(media.data_size(), 1024);
        assert!(!media.is_empty());
    }

    #[test]
    fn test_image_media_is_empty() {
        let media = ImageMedia::new(vec![], MediaImageFormat::Unknown, 0, 0);
        assert!(media.is_empty());
    }

    #[test]
    fn test_from_data_png() {
        let mut data = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        data.extend_from_slice(&[0; 100]);
        let media = ImageMedia::from_data(data, 10, 10);
        assert_eq!(media.format, MediaImageFormat::Png);
    }

    #[test]
    fn test_from_data_jpeg() {
        let data = vec![0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x00];
        let media = ImageMedia::from_data(data, 10, 10);
        assert_eq!(media.format, MediaImageFormat::Jpeg);
    }

    #[test]
    fn test_from_data_unknown() {
        let data = vec![0x00, 0x01, 0x02, 0x03];
        let media = ImageMedia::from_data(data, 10, 10);
        assert_eq!(media.format, MediaImageFormat::Unknown);
    }
}
