//! 图片对象。

use crate::model::image_format::ImageFormat;

/// 带位置和尺寸的图片对象。
#[derive(Debug, Clone)]
pub struct ImageObject {
    /// 距左边缘的 X 位置（mm）。
    pub x: f64,
    /// 距顶部的 Y 位置（mm）。
    pub y: f64,
    /// 宽度（mm）。
    pub width: f64,
    /// 高度（mm）。
    pub height: f64,
    /// 图片数据（原始字节）。
    pub data: Vec<u8>,
    /// 图片格式。
    pub format: ImageFormat,
}

impl ImageObject {
    /// 创建新的图片对象。
    #[must_use]
    pub fn new(
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        data: Vec<u8>,
        format: ImageFormat,
    ) -> Self {
        Self {
            x,
            y,
            width,
            height,
            data,
            format,
        }
    }

    /// 创建 JPEG 图片对象。
    #[must_use]
    pub fn jpeg(x: f64, y: f64, width: f64, height: f64, data: Vec<u8>) -> Self {
        Self::new(x, y, width, height, data, ImageFormat::Jpeg)
    }

    /// 创建 PNG 图片对象。
    #[must_use]
    pub fn png(x: f64, y: f64, width: f64, height: f64, data: Vec<u8>) -> Self {
        Self::new(x, y, width, height, data, ImageFormat::Png)
    }

    /// 从文件路径创建图片对象，自动检测格式。
    ///
    /// 从以下信息检测格式：
    /// - 文件扩展名（.jpg/.jpeg → Jpeg, .png → Png, .bmp → Bmp, .tiff/.tif → Tiff）
    /// - 魔术字节（扩展名不明确时的回退方案）
    ///
    /// # 错误
    ///
    /// 文件无法读取或格式不支持时返回错误。
    pub fn from_file(
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        path: impl AsRef<std::path::Path>,
    ) -> crate::OfdResult<Self> {
        let data = std::fs::read(path.as_ref()).map_err(crate::OfdError::Io)?;
        let format = detect_image_format(path.as_ref(), &data);
        Ok(Self::new(x, y, width, height, data, format))
    }
}

/// 从文件扩展名和/或魔术字节检测图片格式。
fn detect_image_format(path: &std::path::Path, data: &[u8]) -> ImageFormat {
    // 首先检查扩展名
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        match ext.to_lowercase().as_str() {
            "jpg" | "jpeg" => return ImageFormat::Jpeg,
            "png" => return ImageFormat::Png,
            "bmp" => return ImageFormat::Bmp,
            "tiff" | "tif" => return ImageFormat::Tiff,
            _ => {}
        }
    }
    // 回退: 魔术字节
    if data.len() >= 2 && data[0] == 0xFF && data[1] == 0xD8 {
        return ImageFormat::Jpeg;
    }
    if data.len() >= 4 && data[0] == 0x89 && data[1] == b'P' && data[2] == b'N' && data[3] == b'G' {
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
