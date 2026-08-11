//! AWT 渲染器。
//!
//! 对应 Java: org.ofdrw.converter.AWTMaker
//!
//! Java 版 `AWTMaker` 依赖 `java.awt.Graphics2D` 进行像素级渲染，
//! 包括字体渲染（`FontRenderContext`）、坐标变换（`AffineTransform`）、
//! 图片绘制（`BufferedImage`）等 AWT/Swing 专有 API。
//!
//! Rust 版使用 [`RasterRenderer`] 替代，基于 `image` crate 的 `RgbaImage`
//! 作为画布，`fontdue` 渲染文本，`tiny-skia` 渲染矢量路径，`image` 解码/合成图片。
//! 渲染管线与 Java AWTMaker 功能等价。

use std::path::Path;

use easyofd_core::{OfdError, OfdResult};
use easyofd_reader::OfdReader;

use super::raster::RasterRenderer;
use crate::ConvertOptions;

/// OFD → 图片渲染器。
///
/// 对应 Java: `org.ofdrw.converter.AWTMaker`
///
/// 持有 OFD 文件数据与转换选项，内部委托 [`RasterRenderer`] 完成
/// 真实的像素级渲染，输出 PNG 图片。
#[derive(Debug)]
pub struct AWTMaker {
    /// OFD 文件原始字节。
    ofd_bytes: Vec<u8>,
    /// 转换选项。
    options: ConvertOptions,
    /// 栅格渲染器。
    renderer: RasterRenderer,
}

impl AWTMaker {
    /// 从 OFD 文件路径创建 AWTMaker。
    ///
    /// # 错误
    ///
    /// 文件无法读取时返回错误。
    pub fn from_file(path: impl AsRef<Path>) -> OfdResult<Self> {
        let ofd_bytes = std::fs::read(path.as_ref()).map_err(OfdError::Io)?;
        Ok(Self {
            ofd_bytes,
            options: ConvertOptions::default(),
            renderer: RasterRenderer::new(),
        })
    }

    /// 从 OFD 字节数据创建 AWTMaker。
    #[must_use]
    pub fn from_bytes(ofd_bytes: Vec<u8>) -> Self {
        Self {
            ofd_bytes,
            options: ConvertOptions::default(),
            renderer: RasterRenderer::new(),
        }
    }

    /// 设置转换选项。
    #[must_use]
    pub fn with_options(mut self, options: ConvertOptions) -> Self {
        self.options = options;
        self
    }

    /// 设置渲染 DPI。
    #[must_use]
    pub fn with_dpi(mut self, dpi: f64) -> Self {
        self.renderer = self.renderer.with_dpi(dpi);
        self
    }

    /// 设置背景颜色（RGB 0xRRGGBB）。
    #[must_use]
    pub fn with_background_color(mut self, color: u32) -> Self {
        self.renderer = self.renderer.with_background_color(color);
        self
    }

    /// 获取渲染器的引用。
    #[must_use]
    pub fn renderer(&self) -> &RasterRenderer {
        &self.renderer
    }

    /// 将 OFD 渲染为 PNG 并写入目标路径。
    ///
    /// 对应 Java: `AWTMaker.convert(Path target)`
    ///
    /// 对于多页 OFD，每页生成独立的 PNG 文件，文件名后缀 `_0`, `_1`, ...
    /// 对于单页 OFD，直接输出到 target。
    ///
    /// # 错误
    ///
    /// OFD 解析失败、页面范围无效或 IO 错误时返回错误。
    pub fn convert(&self, target: &Path) -> OfdResult<()> {
        let reader = OfdReader::from_bytes(&self.ofd_bytes)?;
        let pages = reader.pages();

        let range = if self.options.pages.is_empty() {
            0..pages.len()
        } else {
            self.options.pages.start.min(pages.len())..self.options.pages.end.min(pages.len())
        };

        if range.is_empty() {
            return Err(OfdError::Conversion("没有可转换的页面".into()));
        }

        let is_single_page = range.len() == 1;

        for page_idx in range {
            let page = &pages[page_idx];
            let rgba_image = self.renderer.render_page(page);

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

    /// 返回替代实现的名称。
    ///
    /// 对应 Java: `AWTMaker` 的替代说明。
    ///
    /// 现在指向真实的渲染实现。
    #[must_use]
    pub fn replacement() -> &'static str {
        "easyofd_convert::converter::AWTMaker (基于 RasterRenderer 真实渲染)"
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
    fn test_awt_maker_from_bytes() {
        let mut writer = OfdWriter::new();
        writer.add_page(OfdPage::new(210.0, 297.0));
        let ofd_bytes = writer.build().unwrap();

        let maker = AWTMaker::from_bytes(ofd_bytes);
        assert!((maker.renderer().dpi() - 96.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_awt_maker_convert_single_page() {
        let mut writer = OfdWriter::new();
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_text(TextObject::new(10.0, 20.0, "AWT 测试"));
        writer.add_page(page);
        let ofd_bytes = writer.build().unwrap();

        let png_path = "/tmp/test_awt_maker.png";
        let maker = AWTMaker::from_bytes(ofd_bytes);
        let result = maker.convert(Path::new(png_path));
        assert!(result.is_ok(), "AWTMaker 转换应成功: {:?}", result.err());
        assert!(Path::new(png_path).exists());

        // 验证 PNG 签名
        let output = std::fs::read(png_path).unwrap();
        assert_eq!(
            &output[0..8],
            &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]
        );

        let _ = std::fs::remove_file(png_path);
    }

    #[test]
    fn test_awt_maker_convert_multi_page() {
        let mut writer = OfdWriter::new();
        writer.add_page(OfdPage::new(100.0, 100.0));
        writer.add_page(OfdPage::new(100.0, 100.0));
        let ofd_bytes = writer.build().unwrap();

        let png_path = "/tmp/test_awt_multi.png";
        let maker = AWTMaker::from_bytes(ofd_bytes);
        let result = maker.convert(Path::new(png_path));
        assert!(result.is_ok());

        assert!(Path::new("/tmp/test_awt_multi_0.png").exists());
        assert!(Path::new("/tmp/test_awt_multi_1.png").exists());

        let _ = std::fs::remove_file("/tmp/test_awt_multi_0.png");
        let _ = std::fs::remove_file("/tmp/test_awt_multi_1.png");
    }

    #[test]
    fn test_awt_maker_with_dpi() {
        let mut writer = OfdWriter::new();
        writer.add_page(OfdPage::new(210.0, 297.0));
        let ofd_bytes = writer.build().unwrap();

        let png_path = "/tmp/test_awt_dpi.png";
        let maker = AWTMaker::from_bytes(ofd_bytes).with_dpi(150.0);
        assert!((maker.renderer().dpi() - 150.0).abs() < f64::EPSILON);

        let result = maker.convert(Path::new(png_path));
        assert!(result.is_ok());

        // 验证高 DPI 下图片尺寸更大
        let img = ::image::open(png_path).unwrap();
        let expected_w = (210.0_f64 / 25.4 * 150.0).round() as u32;
        assert_eq!(img.width(), expected_w);

        let _ = std::fs::remove_file(png_path);
    }

    #[test]
    fn test_awt_maker_from_file() {
        let mut writer = OfdWriter::new();
        writer.add_page(OfdPage::new(100.0, 100.0));
        let ofd_bytes = writer.build().unwrap();

        let ofd_path = "/tmp/test_awt_file.ofd";
        std::fs::write(ofd_path, &ofd_bytes).unwrap();

        let maker = AWTMaker::from_file(ofd_path);
        assert!(maker.is_ok());

        let _ = std::fs::remove_file(ofd_path);
    }

    #[test]
    fn test_awt_maker_from_file_not_found() {
        let result = AWTMaker::from_file("/nonexistent.ofd");
        assert!(result.is_err());
    }

    #[test]
    fn test_awt_maker_with_options() {
        let mut writer = OfdWriter::new();
        writer.add_page(OfdPage::new(100.0, 100.0));
        writer.add_page(OfdPage::new(100.0, 100.0));
        writer.add_page(OfdPage::new(100.0, 100.0));
        let ofd_bytes = writer.build().unwrap();

        // 只导出第 1-2 页
        let options = ConvertOptions {
            pages: 0..2,
            page_size: None,
        };
        let png_path = "/tmp/test_awt_options.png";
        let maker = AWTMaker::from_bytes(ofd_bytes).with_options(options);
        let result = maker.convert(Path::new(png_path));
        assert!(result.is_ok());

        // 只应生成 2 个文件
        assert!(Path::new("/tmp/test_awt_options_0.png").exists());
        assert!(Path::new("/tmp/test_awt_options_1.png").exists());
        assert!(!Path::new("/tmp/test_awt_options_2.png").exists());

        let _ = std::fs::remove_file("/tmp/test_awt_options_0.png");
        let _ = std::fs::remove_file("/tmp/test_awt_options_1.png");
    }

    #[test]
    fn test_awt_maker_empty_page_no_panic() {
        let mut writer = OfdWriter::new();
        writer.add_page(OfdPage::new(210.0, 297.0));
        let ofd_bytes = writer.build().unwrap();

        let png_path = "/tmp/test_awt_empty.png";
        let maker = AWTMaker::from_bytes(ofd_bytes);
        let result = maker.convert(Path::new(png_path));
        assert!(result.is_ok(), "空页面不应 panic");

        let _ = std::fs::remove_file(png_path);
    }

    #[test]
    fn test_awt_maker_mixed_content() {
        let mut writer = OfdWriter::new();
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_text(TextObject::new(10.0, 20.0, "混合内容"));
        page.add_path(PathObject::rect(50.0, 50.0, 100.0, 50.0).fill_color(0xFF_0000));
        writer.add_page(page);
        let ofd_bytes = writer.build().unwrap();

        let png_path = "/tmp/test_awt_mixed.png";
        let maker = AWTMaker::from_bytes(ofd_bytes);
        let result = maker.convert(Path::new(png_path));
        assert!(result.is_ok());

        // 验证非纯白输出
        let img = ::image::open(png_path).unwrap().to_rgba8();
        let mut non_white = 0;
        for pixel in img.pixels() {
            if pixel[0] < 255 || pixel[1] < 255 || pixel[2] < 255 {
                non_white += 1;
            }
        }
        assert!(non_white > 0, "混合内容页面应有非白色像素");

        let _ = std::fs::remove_file(png_path);
    }

    #[test]
    fn test_awt_maker_replacement() {
        // 更新后的 replacement 指向真实实现
        assert!(AWTMaker::replacement().contains("AWTMaker"));
        assert!(AWTMaker::replacement().contains("RasterRenderer"));
    }

    #[test]
    fn test_awt_maker_with_background_color() {
        let mut writer = OfdWriter::new();
        writer.add_page(OfdPage::new(100.0, 100.0));
        let ofd_bytes = writer.build().unwrap();

        let png_path = "/tmp/test_awt_bg.png";
        let maker = AWTMaker::from_bytes(ofd_bytes).with_background_color(0x00_0000);
        let result = maker.convert(Path::new(png_path));
        assert!(result.is_ok());

        // 验证背景色为黑色
        let img = ::image::open(png_path).unwrap().to_rgba8();
        let pixel = img.get_pixel(0, 0);
        assert_eq!(pixel[0], 0);
        assert_eq!(pixel[1], 0);
        assert_eq!(pixel[2], 0);

        let _ = std::fs::remove_file(png_path);
    }
}
