//! OFD → PNG 导出器。
//!
//! 对应 Java: org.ofdrw.converter.ofdconverter.ImageConverter
//!
//! 将 OFD 页面内容真实渲染为 PNG 图片。内部使用
//! [`crate::converter::RasterRenderer`] 完成像素级渲染，
//! 再通过 `image` crate 编码为 PNG 格式写出。

use std::path::Path;

use easyofd_core::{OfdError, OfdResult};
use easyofd_reader::OfdReader;

use crate::converter::RasterRenderer;

use super::Exporter;
use crate::ConvertOptions;

/// OFD → PNG 图片导出器。
///
/// 对应 Java: org.ofdrw.converter.ofdconverter.ImageConverter
///
/// 将 OFD 页面内容渲染为 PNG 图片。使用 [`RasterRenderer`] 执行
/// 真实的像素级渲染（文本、图片、路径三路），产出非占位的真实页面图片。
pub struct ImageExporter {
    /// 转换选项。
    options: ConvertOptions,
    /// 栅格渲染器。
    renderer: RasterRenderer,
}

impl ImageExporter {
    /// 创建新的图片导出器。
    pub fn new(options: ConvertOptions) -> Self {
        Self {
            options,
            renderer: RasterRenderer::new(),
        }
    }

    /// 使用默认选项创建图片导出器。
    pub fn with_defaults() -> Self {
        Self {
            options: ConvertOptions::default(),
            renderer: RasterRenderer::new(),
        }
    }

    /// 获取转换选项的引用。
    pub fn options(&self) -> &ConvertOptions {
        &self.options
    }

    /// 设置转换选项。
    pub fn set_options(&mut self, options: ConvertOptions) {
        self.options = options;
    }

    /// 获取渲染器的引用。
    pub fn renderer(&self) -> &RasterRenderer {
        &self.renderer
    }

    /// 获取渲染器的可变引用（用于调整 DPI 等参数）。
    pub fn renderer_mut(&mut self) -> &mut RasterRenderer {
        &mut self.renderer
    }
}

impl Exporter for ImageExporter {
    fn convert(&self, source: &Path, target: &Path) -> OfdResult<()> {
        let ofd_bytes = std::fs::read(source).map_err(OfdError::Io)?;
        let reader = OfdReader::from_bytes(&ofd_bytes)?;

        let pages = reader.pages();
        let range = if self.options.pages.is_empty() {
            0..pages.len()
        } else {
            self.options.pages.start.min(pages.len())..self.options.pages.end.min(pages.len())
        };

        if range.is_empty() {
            return Err(OfdError::Conversion("没有可转换的页面".into()));
        }

        // 对于单页导出，直接输出到 target
        // 对于多页导出，使用 target 作为基础文件名，添加页码后缀
        let is_single_page = range.len() == 1;

        for page_idx in range {
            let page = &pages[page_idx];
            let rgba_image = self.renderer.render_page(page);

            // 编码为 PNG
            let png_data = encode_to_png(&rgba_image)?;

            let output_path = if is_single_page {
                target.to_path_buf()
            } else {
                let stem = target
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("page");
                let ext = target.extension().and_then(|s| s.to_str()).unwrap_or("png");
                target.with_file_name(format!("{stem}_{page_idx}.{ext}"))
            };

            std::fs::write(&output_path, &png_data).map_err(OfdError::Io)?;
        }

        Ok(())
    }
}

/// 将 `RgbaImage` 编码为 PNG 字节数据。
fn encode_to_png(img: &::image::RgbaImage) -> OfdResult<Vec<u8>> {
    use ::image::ImageEncoder;
    let mut buf = Vec::new();
    let encoder = ::image::codecs::png::PngEncoder::new(&mut buf);
    encoder
        .write_image(
            img.as_raw(),
            img.width(),
            img.height(),
            ::image::ExtendedColorType::Rgba8,
        )
        .map_err(|e| OfdError::Conversion(format!("PNG 编码失败: {e}")))?;
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;
    use easyofd_core::{OfdPage, PathObject, TextObject};
    use easyofd_writer::OfdWriter;

    #[test]
    fn test_image_exporter_new() {
        let options = ConvertOptions {
            pages: 0..5,
            page_size: Some((210.0, 297.0)),
        };
        let exporter = ImageExporter::new(options);
        assert_eq!(exporter.options().pages, 0..5);
        assert_eq!(exporter.options().page_size, Some((210.0, 297.0)));
    }

    #[test]
    fn test_image_exporter_with_defaults() {
        let exporter = ImageExporter::with_defaults();
        assert!(exporter.options().pages.is_empty());
        assert!(exporter.options().page_size.is_none());
    }

    #[test]
    fn test_image_exporter_convert_single_page() {
        let mut writer = OfdWriter::new();
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_text(TextObject::new(10.0, 20.0, "图片导出测试"));
        writer.add_page(page);
        let ofd_bytes = writer.build().unwrap();

        let ofd_path = "/tmp/test_image_exporter.ofd";
        let png_path = "/tmp/test_image_exporter.png";
        std::fs::write(ofd_path, &ofd_bytes).unwrap();

        let exporter = ImageExporter::with_defaults();
        let result = exporter.convert(Path::new(ofd_path), Path::new(png_path));
        assert!(result.is_ok(), "图片导出应该成功: {:?}", result.err());
        assert!(Path::new(png_path).exists());

        // 验证输出是合法 PNG
        let output = std::fs::read(png_path).unwrap();
        assert!(output.len() > 8);
        assert_eq!(
            &output[0..8],
            &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]
        );

        // 验证图片尺寸（210mm x 297mm at 96 DPI）
        let expected_w = (210.0_f64 / 25.4 * 96.0).round() as u32;
        let expected_h = (297.0_f64 / 25.4 * 96.0).round() as u32;
        let img = ::image::open(png_path).unwrap();
        assert_eq!(img.width(), expected_w, "宽度应为 {expected_w}px");
        assert_eq!(img.height(), expected_h, "高度应为 {expected_h}px");

        // 验证图片非纯白（有文本内容）
        let rgba = img.to_rgba8();
        let mut non_white_count = 0;
        for pixel in rgba.pixels() {
            if pixel[0] < 255 || pixel[1] < 255 || pixel[2] < 255 {
                non_white_count += 1;
            }
        }
        assert!(
            non_white_count > 0,
            "渲染后的图片应包含非白色像素（文本/路径内容）"
        );

        let _ = std::fs::remove_file(ofd_path);
        let _ = std::fs::remove_file(png_path);
    }

    #[test]
    fn test_image_exporter_convert_multi_page() {
        let mut writer = OfdWriter::new();
        writer.add_page(OfdPage::new(210.0, 297.0));
        writer.add_page(OfdPage::new(210.0, 297.0));
        writer.add_page(OfdPage::new(210.0, 297.0));
        let ofd_bytes = writer.build().unwrap();

        let ofd_path = "/tmp/test_multi_page.ofd";
        let png_path = "/tmp/test_multi_page.png";
        std::fs::write(ofd_path, &ofd_bytes).unwrap();

        let exporter = ImageExporter::with_defaults();
        let result = exporter.convert(Path::new(ofd_path), Path::new(png_path));
        assert!(result.is_ok(), "多页导出应成功: {:?}", result.err());

        // 多页应生成 _0, _1, _2 后缀
        assert!(Path::new("/tmp/test_multi_page_0.png").exists());
        assert!(Path::new("/tmp/test_multi_page_1.png").exists());
        assert!(Path::new("/tmp/test_multi_page_2.png").exists());

        let _ = std::fs::remove_file(ofd_path);
        let _ = std::fs::remove_file("/tmp/test_multi_page_0.png");
        let _ = std::fs::remove_file("/tmp/test_multi_page_1.png");
        let _ = std::fs::remove_file("/tmp/test_multi_page_2.png");
    }

    #[test]
    fn test_image_exporter_convert_mixed_content() {
        let mut writer = OfdWriter::new();
        let mut page = OfdPage::new(100.0, 100.0);
        page.add_text(TextObject::new(10.0, 20.0, "Hello"));
        page.add_path(PathObject::rect(5.0, 5.0, 80.0, 80.0));
        writer.add_page(page);
        let ofd_bytes = writer.build().unwrap();

        let ofd_path = "/tmp/test_mixed_content.ofd";
        let png_path = "/tmp/test_mixed_content.png";
        std::fs::write(ofd_path, &ofd_bytes).unwrap();

        let exporter = ImageExporter::with_defaults();
        let result = exporter.convert(Path::new(ofd_path), Path::new(png_path));
        assert!(result.is_ok(), "混合内容导出应成功: {:?}", result.err());

        let output = std::fs::read(png_path).unwrap();
        assert_eq!(
            &output[0..8],
            &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]
        );

        let _ = std::fs::remove_file(ofd_path);
        let _ = std::fs::remove_file(png_path);
    }

    #[test]
    fn test_image_exporter_render_empty_page() {
        let mut writer = OfdWriter::new();
        writer.add_page(OfdPage::new(210.0, 297.0));
        let ofd_bytes = writer.build().unwrap();

        let ofd_path = "/tmp/test_empty_page.ofd";
        let png_path = "/tmp/test_empty_page.png";
        std::fs::write(ofd_path, &ofd_bytes).unwrap();

        let exporter = ImageExporter::with_defaults();
        let result = exporter.convert(Path::new(ofd_path), Path::new(png_path));
        assert!(result.is_ok(), "空页面不应 panic: {:?}", result.err());

        // 空页面应为白色
        let img = ::image::open(png_path).unwrap().to_rgba8();
        for pixel in img.pixels() {
            assert_eq!(pixel[0], 255);
            assert_eq!(pixel[1], 255);
            assert_eq!(pixel[2], 255);
        }

        let _ = std::fs::remove_file(ofd_path);
        let _ = std::fs::remove_file(png_path);
    }

    #[test]
    fn test_image_exporter_set_options() {
        let mut exporter = ImageExporter::with_defaults();
        assert!(exporter.options().pages.is_empty());

        let options = ConvertOptions {
            pages: 2..8,
            page_size: Some((100.0, 150.0)),
        };
        exporter.set_options(options);
        assert_eq!(exporter.options().pages, 2..8);
        assert_eq!(exporter.options().page_size, Some((100.0, 150.0)));
    }

    #[test]
    fn test_image_exporter_missing_file() {
        let exporter = ImageExporter::with_defaults();
        let result = exporter.convert(Path::new("/nonexistent.ofd"), Path::new("/tmp/out.png"));
        assert!(result.is_err());
    }

    #[test]
    fn test_image_exporter_renderer_access() {
        let mut exporter = ImageExporter::with_defaults();
        assert!((exporter.renderer().dpi() - 96.0).abs() < f64::EPSILON);
        *exporter.renderer_mut() = RasterRenderer::new().with_dpi(300.0);
        assert!((exporter.renderer().dpi() - 300.0).abs() < f64::EPSILON);
    }
}
