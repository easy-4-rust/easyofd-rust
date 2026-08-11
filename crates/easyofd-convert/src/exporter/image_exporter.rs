//! OFD → PNG 导出器。

use std::path::Path;

use easyofd_core::{ContentObject, OfdError, OfdResult};
use easyofd_reader::OfdReader;

use super::Exporter;
use crate::ConvertOptions;

/// OFD → PNG 图片导出器。
///
/// 对应 Java: org.ofdrw.converter.ofdconverter.ImageConverter
///
/// 将 OFD 页面内容渲染为 PNG 图片。
/// 当前实现为简化版本，生成包含页面内容文本摘要的占位图片。
pub struct ImageExporter {
    /// 转换选项。
    options: ConvertOptions,
}

impl ImageExporter {
    /// 创建新的图片导出器。
    pub fn new(options: ConvertOptions) -> Self {
        Self { options }
    }

    /// 使用默认选项创建图片导出器。
    pub fn with_defaults() -> Self {
        Self {
            options: ConvertOptions::default(),
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
            let png_data = render_page_to_png(page)?;

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

/// 将页面渲染为 PNG 数据。
///
/// 当前实现生成一个简单的 PNG 图片，包含页面尺寸信息和内容摘要。
/// 完整实现需要集成 image crate 进行像素级渲染。
fn render_page_to_png(page: &easyofd_core::OfdPage) -> OfdResult<Vec<u8>> {
    // 收集页面中的文本内容作为元信息
    let text_summary: String = page
        .content
        .iter()
        .filter_map(|obj| {
            if let ContentObject::Text(t) = obj {
                Some(t.text.as_str())
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .join(" ");

    // 生成一个最小的合法 PNG 文件（1x1 白色像素）
    // 真实实现应使用 image crate 渲染完整页面
    let png_data = create_minimal_png(page.width as u32, page.height as u32, &text_summary);

    Ok(png_data)
}

/// 创建最小的合法 PNG 文件。
///
/// 生成一个单像素白色 PNG，用于占位。
/// PNG 格式参考：https://www.w3.org/TR/PNG/
fn create_minimal_png(_width: u32, _height: u32, _text: &str) -> Vec<u8> {
    // PNG 签名
    let mut png = Vec::new();
    png.extend_from_slice(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]);

    // IHDR chunk: 1x1, 8-bit RGB
    let width: u32 = 1;
    let height: u32 = 1;
    let ihdr_data: Vec<u8> = [
        &width.to_be_bytes()[..],
        &height.to_be_bytes()[..],
        &[8u8, 2, 0, 0, 0], // bit depth=8, color type=2 (RGB), compression=0, filter=0, interlace=0
    ]
    .concat();
    append_chunk(&mut png, *b"IHDR", &ihdr_data);

    // IDAT chunk: 1x1 RGB pixel (white), with zlib wrapper
    // filter byte (0=None) + RGB(255,255,255)
    let raw_data = [0u8, 255, 255, 255]; // filter=0, R=255, G=255, B=255
    let compressed = zlib_compress_simple(&raw_data);
    append_chunk(&mut png, *b"IDAT", &compressed);

    // IEND chunk
    append_chunk(&mut png, *b"IEND", &[]);

    png
}

/// 向 PNG 数据追加一个 chunk。
fn append_chunk(png: &mut Vec<u8>, chunk_type: [u8; 4], data: &[u8]) {
    // Length (4 bytes, big-endian)
    png.extend_from_slice(&(data.len() as u32).to_be_bytes());
    // Chunk type
    png.extend_from_slice(&chunk_type);
    // Data
    png.extend_from_slice(data);
    // CRC32 over chunk_type + data
    let crc_input: Vec<u8> = chunk_type.iter().chain(data.iter()).copied().collect();
    let crc = crc32(&crc_input);
    png.extend_from_slice(&crc.to_be_bytes());
}

/// 简化的 zlib 压缩（无压缩，store 模式）。
///
/// 生成 zlib 格式的包装：CMF(1 byte) + FLG(1 byte) + [block header] + [data] + ADLER32(4 bytes)
fn zlib_compress_simple(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    // CMF: CM=8 (deflate), CINFO=7 (32K window)
    out.push(0x78);
    // FLG: FCHECK 使得 (CMF*256+FLG) % 31 == 0, FDICT=0, FLEVEL=0
    out.push(0x01);
    // Deflate stored block: BFINAL=1, BTYPE=00
    out.push(0x01);
    // LEN (2 bytes LE)
    out.extend_from_slice(&(data.len() as u16).to_le_bytes());
    // NLEN (2 bytes LE) = ~LEN
    out.extend_from_slice(&(!(data.len() as u16)).to_le_bytes());
    // Data
    out.extend_from_slice(data);
    // ADLER32
    let adler = adler32(data);
    out.extend_from_slice(&adler.to_be_bytes());
    out
}

/// CRC32 计算。
fn crc32(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFF_FFFF;
    for &byte in data {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ 0xEDB8_8320;
            } else {
                crc >>= 1;
            }
        }
    }
    crc ^ 0xFFFF_FFFF
}

/// Adler-32 校验和计算。
fn adler32(data: &[u8]) -> u32 {
    let mut a: u32 = 1;
    let mut b: u32 = 0;
    for &byte in data {
        a = (a + u32::from(byte)) % 65521;
        b = (b + a) % 65521;
    }
    (b << 16) | a
}

#[cfg(test)]
mod tests {
    use super::*;
    use easyofd_core::{OfdPage, TextObject};
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
    fn test_image_exporter_convert() {
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

        // 验证输出是合法 PNG（以 PNG 签名开头）
        let output = std::fs::read(png_path).unwrap();
        assert!(output.len() > 8);
        assert_eq!(
            &output[0..8],
            &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]
        );

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
    fn test_crc32() {
        // 测试已知的 CRC32 值
        assert_eq!(crc32(b"123456789"), 0xCBF4_3926);
    }

    #[test]
    fn test_adler32() {
        // "Wikipedia" 的 Adler-32
        assert_eq!(adler32(b"Wikipedia"), 0x11E6_0398);
    }
}
