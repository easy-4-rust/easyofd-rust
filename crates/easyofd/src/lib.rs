//! # `EasyOFD`
//!
//! A Rust library for easy OFD (Open Fixed-layout Document) operations,
//! inspired by [EasyExcel](https://github.com/alibaba/easyexcel).
//!
//! ## ofdrw 子模块对照表
//!
//! 本项目在架构上对标 Java 版 [ofdrw](https://github.com/ofdrw/ofdrw) v2.4.0 的 14 个子模块。
//! 下表列出每个 ofdrw 模块与对应 easyofd crate 的映射关系：
//!
//! | ofdrw 模块 | easyofd crate | 说明 |
//! |---|---|---|
//! | `ofdrw-core` | `easyofd-core` | 核心数据模型 |
//! | `ofdrw-pkg` | `easyofd-package` | ZIP 安全与包结构 |
//! | `ofdrw-layout` | `easyofd-layout` | 版面分析 |
//! | `ofdrw-font` | `easyofd-font` | 字体管理与嵌入（预留） |
//! | `ofdrw-full` | `easyofd`（本 crate） | 聚合门面 |
//! | `ofdrw-reader` | `easyofd-reader` | OFD 解析 |
//! | `ofdrw-sign` | `easyofd-signature` | 数字签名（GB/T 38540） |
//! | `ofdrw-gv` | 不单独建 crate | 全局变量，仅 1 个文件 |
//! | `ofdrw-gm` | `easyofd-gm` | 国密算法 SM2/SM3/SM4（预留） |
//! | `ofdrw-converter` | `easyofd-convert` | OFD/PDF 互转 |
//! | `ofdrw-crypto` | `easyofd-crypto` | 加密基础设施（预留） |
//! | `ofdrw-tool` | 不单独建 crate | CLI 工具，用户可后续决策 |
//! | `ofdrw-graphics2d` | `easyofd-graphics2d` | 2D 图形渲染抽象（预留） |
//! | `ofdrw-archive` | `easyofd-archive` | 归档合规规则引擎（预留） |
//!
//! 额外 crate（ofdrw 无直接对应）：
//! - `easyofd-derive` / `easyofd-derive-impl`：过程宏（`#[derive(OfdModel)]`）
//! - `easyofd-markdown`：OFD 转 Markdown
//! - `easyofd-template`：OFD 模板填充
//! - `easyofd-writer`：OFD 写入器
//!
//! ## Quick Start
//!
//! ### One-liner write with derive macro
//!
//! ```rust,ignore
//! use easyofd::{EasyOfd, OfdModel};
//!
//! #[derive(OfdModel)]
//! #[ofd(page_width = 210.0, page_height = 297.0)]
//! struct Invoice {
//!     #[ofd(x = 20.0, y = 30.0, size = 18.0, bold)]
//!     title: String,
//!     #[ofd(x = 20.0, y = 50.0)]
//!     amount: String,
//! }
//!
//! let data = vec![
//!     Invoice { title: "Invoice #001".into(), amount: "$100.00".into() },
//!     Invoice { title: "Invoice #002".into(), amount: "$200.00".into() },
//! ];
//!
//! EasyOfd::write::<Invoice>("output.ofd").do_write(&data)?;
//! ```
//!
//! ### Manual page construction
//!
//! ```rust,ignore
//! use easyofd::{EasyOfd, TextObject, OfdPage};
//!
//! let mut page = OfdPage::new(210.0, 297.0);
//! page.add_text(TextObject::new(20.0, 30.0, "Hello OFD!"));
//!
//! EasyOfd::write_pages("output.ofd")
//!     .metadata_title("My Document")
//!     .do_write(vec![page])?;
//! ```

mod easy_ofd;
mod ofd_read_builder;
mod ofd_writer_builder;
mod page_writer_builder;

// Re-export core types for convenience.
pub use easyofd_core::{
    AnnPage, Annot, AnnotType, Annotations, Appearance, CTAction, ContentObject, EventType,
    ImageFormat, ImageObject, OfdAction, OfdError, OfdField, OfdFieldKind, OfdMetadata, OfdModel,
    OfdPage, OfdResult, PageAnnot, PathObject, TextObject, URI, Watermark, page_size,
};

// Re-export derive macro.
pub use easyofd_derive::OfdModel;

// Re-export writer internals for advanced usage.
pub use easyofd_writer::{
    EmbeddedFont, FontFormat, OfdEditor, OfdStreamWriter, OfdWriter, WriteOptions,
};

// Re-export reader for advanced usage.
pub use easyofd_reader::OfdReader;
pub use easyofd_reader::ReadOptions;
pub use ofd_read_builder::OfdReadBuilder;

// Re-export package, layout and Markdown conversion APIs.
pub use easyofd_layout::{LayoutAnalyzer, LayoutBlock, LayoutOptions, LayoutResult};
pub use easyofd_markdown::{
    ConversionLoss, ConversionReport, ConversionWarning, ConvertedAsset, ImagePolicy,
    MarkdownConversionBuilder, MarkdownConversionResult, MarkdownConverter, MarkdownOptions,
    OcrPolicy, OcrProvider, PageBreakStyle,
};
pub use easyofd_package::PackageLimits;

// Re-export template for advanced usage.
pub use easyofd_template::OfdTemplateFiller;

// Re-export signature types for advanced usage.
pub use easyofd_signature::{
    ElectronicSeal, OfdSignatureBuilder, SignatureAlgorithm, SignedOfd, read_signature,
    verify_signature,
};

// Re-export convert functions for advanced usage.
pub use easyofd_convert::{
    ConvertOptions, ImageConvertFormat, convert_image, ofd_to_pdf, pdf_to_ofd,
};

// Re-export facade types.
pub use easy_ofd::EasyOfd;
pub use ofd_writer_builder::OfdWriterBuilder;
pub use page_writer_builder::PageWriterBuilder;

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── EasyOfd static methods ────────────────────────────────────────────────

    #[test]
    fn test_write_pages_builder() {
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_text(TextObject::new(20.0, 30.0, "Hello from EasyOFD!"));

        let bytes = EasyOfd::write_pages("test.ofd")
            .metadata_title("Test Document")
            .do_write_to_bytes(vec![page])
            .unwrap();

        assert!(!bytes.is_empty());
        assert_eq!(&bytes[0..2], b"PK");
    }

    #[test]
    fn test_write_pages_to_bytes_static() {
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_text(TextObject::new(0.0, 0.0, "Direct bytes"));

        let bytes = EasyOfd::write_pages_to_bytes(vec![page]).unwrap();
        assert!(!bytes.is_empty());
        assert_eq!(&bytes[0..2], b"PK");
    }

    #[test]
    fn test_write_pages_to_file() {
        let dir = std::env::temp_dir().join("easyofd_unit_test");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("unit_test.ofd");

        let page = OfdPage::new(210.0, 297.0);
        EasyOfd::write_pages_to(&path, vec![page]).unwrap();

        let bytes = std::fs::read(&path).unwrap();
        assert_eq!(&bytes[0..2], b"PK");
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_streaming_read_and_markdown_facades() {
        let dir = std::env::temp_dir().join("easyofd_facade_markdown");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("facade.ofd");
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_text(TextObject::new(10.0, 10.0, "Facade title").size(24.0));
        EasyOfd::write_pages_to(&path, vec![page]).unwrap();

        let mut visited = 0;
        EasyOfd::read_pages(&path)
            .page_range(1, 1)
            .do_read(|page_number, _| {
                assert_eq!(page_number, 1);
                visited += 1;
                Ok(())
            })
            .unwrap();
        assert_eq!(visited, 1);

        let result = EasyOfd::to_markdown(&path).do_convert().unwrap();
        assert!(result.markdown.contains("Facade title"));
        assert_eq!(result.report.pages_converted, 1);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn test_with_images() {
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_text(TextObject::new(20.0, 30.0, "Invoice"));
        page.add_image(ImageObject::jpeg(
            150.0,
            30.0,
            30.0,
            30.0,
            vec![0xFF, 0xD8, 0xFF, 0xE0],
        ));

        let bytes = EasyOfd::write_pages_to_bytes(vec![page]).unwrap();
        let names = zip_entry_names(&bytes);
        assert!(names.contains(&"Doc_0/Res/Image_0.jpeg".to_string()));
    }

    #[test]
    fn test_with_paths() {
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_text(TextObject::new(20.0, 30.0, "With lines"));
        page.add_path(PathObject::hline(20.0, 40.0, 190.0));
        page.add_path(PathObject::rect(20.0, 50.0, 170.0, 100.0));

        let bytes = EasyOfd::write_pages_to_bytes(vec![page]).unwrap();
        assert!(!bytes.is_empty());
    }

    // ── PageWriterBuilder all methods ─────────────────────────────────────────

    #[test]
    fn test_page_writer_builder_all_metadata() {
        let page = OfdPage::new(210.0, 297.0);
        let bytes = EasyOfd::write_pages("x.ofd")
            .metadata_title("T")
            .metadata_author("A")
            .metadata_creator("C")
            .do_write_to_bytes(vec![page])
            .unwrap();
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_page_writer_builder_do_write() {
        let dir = std::env::temp_dir().join("easyofd_unit_test2");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("page_builder.ofd");

        let page = OfdPage::new(210.0, 297.0);
        EasyOfd::write_pages(path.to_string_lossy().into_owned())
            .do_write(vec![page])
            .unwrap();

        let bytes = std::fs::read(&path).unwrap();
        assert_eq!(&bytes[0..2], b"PK");
        let _ = std::fs::remove_file(&path);
    }

    // ── Edge cases ────────────────────────────────────────────────────────────

    #[test]
    fn test_empty_pages() {
        let bytes = EasyOfd::write_pages_to_bytes(vec![]).unwrap();
        assert!(!bytes.is_empty());
        assert_eq!(&bytes[0..2], b"PK");
    }

    #[test]
    fn test_empty_content_page() {
        let page = OfdPage::new(210.0, 297.0);
        let bytes = EasyOfd::write_pages_to_bytes(vec![page]).unwrap();
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_special_xml_chars() {
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_text(TextObject::new(0.0, 0.0, "<>&\"'"));
        let bytes = EasyOfd::write_pages_to_bytes(vec![page]).unwrap();
        assert!(!bytes.is_empty());
    }

    // ── Re-export verification ────────────────────────────────────────────────

    #[test]
    fn test_re_exports() {
        // Verify all re-exported types are accessible
        let _: OfdPage = OfdPage::new(1.0, 1.0);
        let _: TextObject = TextObject::new(0.0, 0.0, "x");
        let _: ImageObject = ImageObject::jpeg(0.0, 0.0, 1.0, 1.0, vec![0]);
        let _: PathObject = PathObject::new(0.0, 0.0, "M0 0");
        let _: ImageFormat = ImageFormat::Jpeg;
        let _: OfdFieldKind = OfdFieldKind::Text;
        let _: OfdMetadata = OfdMetadata::default();
        let _: WriteOptions = WriteOptions::default();
    }

    fn zip_entry_names(bytes: &[u8]) -> Vec<String> {
        let cursor = std::io::Cursor::new(bytes);
        let mut archive = zip::ZipArchive::new(cursor).unwrap();
        (0..archive.len())
            .map(|i| archive.by_index(i).unwrap().name().to_string())
            .collect()
    }
}
