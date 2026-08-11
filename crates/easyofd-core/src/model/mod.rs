//! 核心 OFD 数据模型类型。
//!
//! 这些类型直接映射到 GB/T 33190-2016 XML 元素。

pub mod bookmark;
pub mod bookmarks;
mod content_object;
pub mod creation_date;
pub mod creator;
pub mod custom_data;
pub mod custom_datas;
mod image_format;
mod image_object;
pub mod ofd_id;
mod ofd_metadata;
mod ofd_page;
pub mod page_size;
mod path_object;
pub mod permissions;
pub mod template_page;
mod text_object;

pub use bookmark::Bookmark;
pub use bookmarks::Bookmarks;
pub use content_object::ContentObject;
pub use creation_date::CreationDate;
pub use creator::Creator;
pub use custom_data::CustomData;
pub use custom_datas::CustomDatas;
pub use image_format::ImageFormat;
pub use image_object::ImageObject;
pub use ofd_id::OfdId;
pub use ofd_metadata::OfdMetadata;
pub use ofd_page::OfdPage;
pub use path_object::PathObject;
pub use permissions::Permissions;
pub use template_page::TemplatePage;
pub use text_object::TextObject;

#[cfg(test)]
mod tests {
    use super::*;

    // ─── OfdMetadata ──────────────────────────────────────────────────────────

    #[test]
    fn test_ofd_metadata_default() {
        let meta = OfdMetadata::default();
        assert_eq!(meta.version, "1.0");
        assert!(meta.title.is_none());
        assert!(meta.author.is_none());
        assert!(meta.creator.is_none());
        assert!(meta.creation_date.is_none());
    }

    #[test]
    fn test_ofd_metadata_clone_debug() {
        let meta = OfdMetadata {
            title: Some("t".into()),
            ..Default::default()
        };
        let meta2 = meta.clone();
        assert_eq!(meta2.title.unwrap(), "t");
        let dbg = format!("{meta:?}");
        assert!(dbg.contains("OfdMetadata"));
    }

    // ─── OfdPage ──────────────────────────────────────────────────────────────

    #[test]
    fn test_ofd_page_new() {
        let page = OfdPage::new(210.0, 297.0);
        assert!((page.width - 210.0).abs() < f64::EPSILON);
        assert!((page.height - 297.0).abs() < f64::EPSILON);
        assert!(page.content.is_empty());
    }

    #[test]
    fn test_ofd_page_add_text() {
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_text(TextObject::new(10.0, 20.0, "hello"));
        assert_eq!(page.content.len(), 1);
        assert!(matches!(&page.content[0], ContentObject::Text(_)));
    }

    #[test]
    fn test_ofd_page_add_image() {
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_image(ImageObject::jpeg(0.0, 0.0, 10.0, 10.0, vec![0xFF]));
        assert_eq!(page.content.len(), 1);
        assert!(matches!(&page.content[0], ContentObject::Image(_)));
    }

    #[test]
    fn test_ofd_page_add_path() {
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_path(PathObject::hline(0.0, 10.0, 100.0));
        assert_eq!(page.content.len(), 1);
        assert!(matches!(&page.content[0], ContentObject::Path(_)));
    }

    #[test]
    fn test_ofd_page_mixed_content() {
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_text(TextObject::new(0.0, 0.0, "t"));
        page.add_image(ImageObject::png(0.0, 0.0, 1.0, 1.0, vec![0x89]));
        page.add_path(PathObject::vline(5.0, 0.0, 10.0));
        assert_eq!(page.content.len(), 3);
    }

    #[test]
    fn test_ofd_page_clone_debug() {
        let page = OfdPage::new(100.0, 200.0);
        let page2 = page.clone();
        assert!((page2.width - 100.0).abs() < f64::EPSILON);
        let dbg = format!("{page:?}");
        assert!(dbg.contains("OfdPage"));
    }

    // ─── page_size constants ──────────────────────────────────────────────────

    #[test]
    fn test_page_size_a4() {
        assert_eq!(page_size::A4, (210.0, 297.0));
    }

    #[test]
    fn test_page_size_a4_landscape() {
        assert_eq!(page_size::A4_LANDSCAPE, (297.0, 210.0));
    }

    #[test]
    fn test_page_size_a3() {
        assert_eq!(page_size::A3, (297.0, 420.0));
    }

    #[test]
    fn test_page_size_letter() {
        assert_eq!(page_size::LETTER, (215.9, 279.4));
    }

    // ─── TextObject ───────────────────────────────────────────────────────────

    #[test]
    fn test_text_object_new() {
        let t = TextObject::new(10.0, 20.0, "hello");
        assert!((t.x - 10.0).abs() < f64::EPSILON);
        assert!((t.y - 20.0).abs() < f64::EPSILON);
        assert_eq!(t.text, "hello");
        assert_eq!(t.font, "SimSun");
        assert!((t.size - 12.0).abs() < f64::EPSILON);
        assert_eq!(t.weight, 400);
        assert!(!t.italic);
        assert_eq!(t.color, 0);
        assert!(t.width.is_none());
        assert!(t.height.is_none());
    }

    #[test]
    fn test_text_object_from_string() {
        let s = String::from("owned");
        let t = TextObject::new(0.0, 0.0, s);
        assert_eq!(t.text, "owned");
    }

    #[test]
    fn test_text_object_builder_font() {
        let t = TextObject::new(0.0, 0.0, "x").font("SimHei");
        assert_eq!(t.font, "SimHei");
    }

    #[test]
    fn test_text_object_builder_size() {
        let t = TextObject::new(0.0, 0.0, "x").size(24.0);
        assert!((t.size - 24.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_text_object_builder_bold() {
        let t = TextObject::new(0.0, 0.0, "x").bold();
        assert_eq!(t.weight, 700);
    }

    #[test]
    fn test_text_object_builder_italic() {
        let t = TextObject::new(0.0, 0.0, "x").italic();
        assert!(t.italic);
    }

    #[test]
    fn test_text_object_builder_color() {
        let t = TextObject::new(0.0, 0.0, "x").color(0xFF_0000);
        assert_eq!(t.color, 0xFF_0000);
    }

    #[test]
    fn test_text_object_builder_chaining() {
        let t = TextObject::new(1.0, 2.0, "x")
            .font("Arial")
            .size(16.0)
            .bold()
            .italic()
            .color(0x00_FF00);
        assert_eq!(t.font, "Arial");
        assert!((t.size - 16.0).abs() < f64::EPSILON);
        assert_eq!(t.weight, 700);
        assert!(t.italic);
        assert_eq!(t.color, 0x00_FF00);
    }

    #[test]
    fn test_text_object_clone_debug() {
        let t = TextObject::new(0.0, 0.0, "x");
        let t2 = t.clone();
        assert_eq!(t2.text, "x");
        let dbg = format!("{t:?}");
        assert!(dbg.contains("TextObject"));
    }

    // ─── ImageFormat ──────────────────────────────────────────────────────────

    #[test]
    fn test_image_format_variants() {
        assert_ne!(ImageFormat::Jpeg, ImageFormat::Png);
        assert_ne!(ImageFormat::Bmp, ImageFormat::Tiff);
        assert_eq!(ImageFormat::Jpeg, ImageFormat::Jpeg);
    }

    #[test]
    fn test_image_format_clone_copy_debug() {
        let f = ImageFormat::Png;
        let f2 = f;
        assert_eq!(f2, ImageFormat::Png);
        let dbg = format!("{f:?}");
        assert!(dbg.contains("Png"));
    }

    // ─── ImageObject ──────────────────────────────────────────────────────────

    #[test]
    fn test_image_object_new() {
        let img = ImageObject::new(10.0, 20.0, 30.0, 40.0, vec![1, 2], ImageFormat::Png);
        assert!((img.x - 10.0).abs() < f64::EPSILON);
        assert!((img.y - 20.0).abs() < f64::EPSILON);
        assert!((img.width - 30.0).abs() < f64::EPSILON);
        assert!((img.height - 40.0).abs() < f64::EPSILON);
        assert_eq!(img.data, vec![1, 2]);
        assert_eq!(img.format, ImageFormat::Png);
    }

    #[test]
    fn test_image_object_jpeg() {
        let img = ImageObject::jpeg(0.0, 0.0, 10.0, 10.0, vec![0xFF]);
        assert_eq!(img.format, ImageFormat::Jpeg);
    }

    #[test]
    fn test_image_object_png() {
        let img = ImageObject::png(0.0, 0.0, 10.0, 10.0, vec![0x89]);
        assert_eq!(img.format, ImageFormat::Png);
    }

    #[test]
    fn test_image_object_clone_debug() {
        let img = ImageObject::jpeg(0.0, 0.0, 1.0, 1.0, vec![0]);
        let img2 = img.clone();
        assert_eq!(img2.data, vec![0]);
        let dbg = format!("{img:?}");
        assert!(dbg.contains("ImageObject"));
    }

    // ─── PathObject ───────────────────────────────────────────────────────────

    #[test]
    fn test_path_object_new() {
        let p = PathObject::new(5.0, 10.0, "M0 0L10 10");
        assert!((p.x - 5.0).abs() < f64::EPSILON);
        assert!((p.y - 10.0).abs() < f64::EPSILON);
        assert_eq!(p.path_data, "M0 0L10 10");
        assert_eq!(p.stroke_color, 0);
        assert!((p.stroke_width - 0.35).abs() < f64::EPSILON);
        assert!(p.fill_color.is_none());
    }

    #[test]
    fn test_path_object_hline() {
        let p = PathObject::hline(10.0, 20.0, 100.0);
        assert!((p.x - 10.0).abs() < f64::EPSILON);
        assert!((p.y - 20.0).abs() < f64::EPSILON);
        assert!(p.path_data.contains("M10"));
        assert!(p.path_data.contains("L100"));
    }

    #[test]
    fn test_path_object_vline() {
        let p = PathObject::vline(5.0, 0.0, 50.0);
        assert!((p.x - 5.0).abs() < f64::EPSILON);
        assert!(p.path_data.contains("M5"));
    }

    #[test]
    fn test_path_object_rect() {
        let p = PathObject::rect(0.0, 0.0, 100.0, 50.0);
        assert!(p.path_data.starts_with('M'));
        assert!(p.path_data.ends_with('Z'));
        assert!(p.path_data.contains("L100"));
    }

    #[test]
    fn test_path_object_builder_stroke_color() {
        let p = PathObject::new(0.0, 0.0, "M0 0").stroke_color(0xFF_0000);
        assert_eq!(p.stroke_color, 0xFF_0000);
    }

    #[test]
    fn test_path_object_builder_stroke_width() {
        let p = PathObject::new(0.0, 0.0, "M0 0").stroke_width(1.5);
        assert!((p.stroke_width - 1.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_path_object_builder_fill_color() {
        let p = PathObject::new(0.0, 0.0, "M0 0").fill_color(0x00_FF00);
        assert_eq!(p.fill_color, Some(0x00_FF00));
    }

    #[test]
    fn test_path_object_builder_chaining() {
        let p = PathObject::new(0.0, 0.0, "M0 0")
            .stroke_color(0xFF)
            .stroke_width(2.0)
            .fill_color(0xFF_FF00);
        assert_eq!(p.stroke_color, 0xFF);
        assert!((p.stroke_width - 2.0).abs() < f64::EPSILON);
        assert_eq!(p.fill_color, Some(0xFF_FF00));
    }

    #[test]
    fn test_path_object_clone_debug() {
        let p = PathObject::new(0.0, 0.0, "M0 0");
        let p2 = p.clone();
        assert_eq!(p2.path_data, "M0 0");
        let dbg = format!("{p:?}");
        assert!(dbg.contains("PathObject"));
    }

    // ─── ContentObject ────────────────────────────────────────────────────────

    #[test]
    fn test_content_object_variants_debug() {
        let text = ContentObject::Text(TextObject::new(0.0, 0.0, "x"));
        let img = ContentObject::Image(ImageObject::jpeg(0.0, 0.0, 1.0, 1.0, vec![0]));
        let path = ContentObject::Path(PathObject::new(0.0, 0.0, "M0 0"));
        assert!(format!("{text:?}").contains("Text"));
        assert!(format!("{img:?}").contains("Image"));
        assert!(format!("{path:?}").contains("Path"));
    }

    // ─── ImageObject::from_file ─────────────────────────────────────────────

    #[test]
    fn test_image_from_file_png() {
        let dir = std::env::temp_dir().join("easyofd_img_test");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.png");
        let png: Vec<u8> = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
        std::fs::write(&path, &png).unwrap();

        let img = ImageObject::from_file(0.0, 0.0, 10.0, 10.0, &path).unwrap();
        assert_eq!(img.format, ImageFormat::Png);
        assert!(!img.data.is_empty());
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_image_from_file_jpg_by_ext() {
        let dir = std::env::temp_dir().join("easyofd_img_test2");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.jpg");
        std::fs::write(&path, b"not-real-jpg").unwrap();

        let img = ImageObject::from_file(0.0, 0.0, 10.0, 10.0, &path).unwrap();
        assert_eq!(img.format, ImageFormat::Jpeg);
        let _ = std::fs::remove_file(&path);
    }
}
