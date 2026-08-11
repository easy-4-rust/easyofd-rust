//! 文档转换器 trait。
//!
//! 对应 Java: org.ofdrw.converter.ofdconverter.DocConverter

use std::path::Path;

/// 文档转换器。
///
/// 对应 Java `DocConverter` 接口。定义了文档格式转换的通用契约。
///
/// 实现此 trait 的类型负责将特定格式的文档（PDF、图片等）转换为 OFD 格式。
pub trait DocConverter {
    /// 转换错误类型。
    type Error: std::error::Error;

    /// 执行转换。
    ///
    /// # 参数
    /// - `output`：输出 OFD 文件路径
    ///
    /// # 错误
    /// 转换失败时返回错误。
    fn convert(&self, output: &Path) -> Result<(), Self::Error>;

    /// 设置转换选项（可选实现）。
    fn set_option(&mut self, _key: &str, _value: &str) {}

    /// 返回源文件路径。
    fn source(&self) -> &Path;
}

/// 文本转换器配置。
///
/// 对应 Java `TextConverter` 的配置参数。
#[derive(Debug, Clone)]
pub struct TextConverterConfig {
    /// 字体名称。
    pub font_name: String,
    /// 字体大小。
    pub font_size: f64,
    /// 页面宽度（mm）。
    pub page_width: f64,
    /// 页面高度（mm）。
    pub page_height: f64,
    /// 行间距。
    pub line_spacing: f64,
}

impl TextConverterConfig {
    /// 创建默认配置。
    pub fn new() -> Self {
        Self {
            font_name: "SimSun".to_string(),
            font_size: 12.0,
            page_width: 210.0,
            page_height: 297.0,
            line_spacing: 1.5,
        }
    }
}

impl Default for TextConverterConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// 图片转换器配置。
///
/// 对应 Java `ImageConverter` 的配置参数。
#[derive(Debug, Clone)]
pub struct ImageConverterConfig {
    /// 页面宽度（mm）。
    pub page_width: f64,
    /// 页面高度（mm）。
    pub page_height: f64,
    /// 图片质量（0-100）。
    pub quality: u8,
    /// 是否自适应页面大小。
    pub fit_to_page: bool,
}

impl ImageConverterConfig {
    /// 创建默认配置。
    pub fn new() -> Self {
        Self {
            page_width: 210.0,
            page_height: 297.0,
            quality: 80,
            fit_to_page: true,
        }
    }
}

impl Default for ImageConverterConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// PDF 转换器配置。
///
/// 对应 Java `PDFConverter` 的配置参数。
#[derive(Debug, Clone)]
pub struct PdfConverterConfig {
    /// 页面宽度（mm）。
    pub page_width: f64,
    /// 页面高度（mm）。
    pub page_height: f64,
    /// DPI。
    pub dpi: u32,
}

impl PdfConverterConfig {
    /// 创建默认配置。
    pub fn new() -> Self {
        Self {
            page_width: 210.0,
            page_height: 297.0,
            dpi: 72,
        }
    }
}

impl Default for PdfConverterConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_converter_config_default() {
        let config = TextConverterConfig::new();
        assert_eq!(config.font_name, "SimSun");
        assert!((config.font_size - 12.0).abs() < f64::EPSILON);
        assert!((config.page_width - 210.0).abs() < f64::EPSILON);
        assert!((config.page_height - 297.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_image_converter_config_default() {
        let config = ImageConverterConfig::default();
        assert_eq!(config.quality, 80);
        assert!(config.fit_to_page);
    }

    #[test]
    fn test_pdf_converter_config_default() {
        let config = PdfConverterConfig::new();
        assert_eq!(config.dpi, 72);
        assert!((config.page_width - 210.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_text_converter_config_clone() {
        let config = TextConverterConfig::new();
        let config2 = config.clone();
        assert_eq!(config.font_name, config2.font_name);
    }
}
