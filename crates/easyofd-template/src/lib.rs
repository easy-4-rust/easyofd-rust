//! # easyofd-template
//!
//! OFD template engine — replaces `{placeholder}` patterns in OFD XML content.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use std::collections::HashMap;
//! use easyofd::EasyOfd;
//!
//! let mut data = HashMap::new();
//! data.insert("name".to_string(), "Alice".to_string());
//! data.insert("amount".to_string(), "$1,234.00".to_string());
//!
//! EasyOfd::fill_template("template.ofd", &data)?
//!     .save("output.ofd")?;
//! ```

mod ofd_template_filler;

pub use ofd_template_filler::OfdTemplateFiller;

#[cfg(test)]
mod tests {
    use super::*;
    use easyofd_core::{OfdPage, TextObject};
    use easyofd_writer::OfdWriter;
    use std::collections::HashMap;
    use std::io::Cursor;

    fn make_template(placeholders: &[&str]) -> Vec<u8> {
        let mut page = OfdPage::new(210.0, 297.0);
        for p in placeholders {
            page.add_text(TextObject::new(20.0, 30.0, format!("{{{p}}}")));
        }
        let mut writer = OfdWriter::new();
        writer.add_page(page);
        writer.build().unwrap()
    }

    #[test]
    fn test_fill_single_placeholder() {
        let template = make_template(&["name"]);
        let mut data = HashMap::new();
        data.insert("name".into(), "Alice".into());

        let filler = OfdTemplateFiller::fill_bytes(&template, &data).unwrap();
        let output = filler.into_bytes();

        // Verify the output is a valid ZIP
        let cursor = Cursor::new(&output);
        let archive = zip::ZipArchive::new(cursor).unwrap();
        assert!(!archive.is_empty());

        // Read the page XML and check replacement
        let reader = easyofd_reader::OfdReader::from_bytes(&output).unwrap();
        let text = reader.extract_all_text();
        assert!(text.contains("Alice"));
        assert!(!text.contains("{name}"));
    }

    #[test]
    fn test_fill_escapes_xml_values() {
        let template = make_template(&["value"]);
        let mut data = HashMap::new();
        data.insert("value".into(), "<A&B>".into());
        let output = OfdTemplateFiller::fill_bytes(&template, &data)
            .unwrap()
            .into_bytes();
        let reader = easyofd_reader::OfdReader::from_bytes(&output).unwrap();
        assert_eq!(reader.extract_all_text(), "<A&B>");
    }

    #[test]
    fn test_fill_multiple_placeholders() {
        let template = make_template(&["title", "amount", "date"]);
        let mut data = HashMap::new();
        data.insert("title".into(), "Invoice #001".into());
        data.insert("amount".into(), "$1,234.00".into());
        data.insert("date".into(), "2026-01-15".into());

        let filler = OfdTemplateFiller::fill_bytes(&template, &data).unwrap();
        let output = filler.into_bytes();

        let reader = easyofd_reader::OfdReader::from_bytes(&output).unwrap();
        let text = reader.extract_all_text();
        assert!(text.contains("Invoice #001"));
        assert!(text.contains("$1,234.00"));
        assert!(text.contains("2026-01-15"));
        assert!(!text.contains("{title}"));
    }

    #[test]
    fn test_fill_template_file() {
        let template = make_template(&["greeting"]);
        let dir = std::env::temp_dir().join("easyofd_template");
        std::fs::create_dir_all(&dir).unwrap();
        let tpl_path = dir.join("template.ofd");
        std::fs::write(&tpl_path, &template).unwrap();

        let mut data = HashMap::new();
        data.insert("greeting".into(), "Hello World".into());

        let filler = OfdTemplateFiller::fill(&tpl_path, &data).unwrap();
        let output_path = dir.join("filled.ofd");
        filler.save(&output_path).unwrap();

        let reader = easyofd_reader::OfdReader::open(&output_path).unwrap();
        assert!(reader.extract_all_text().contains("Hello World"));

        let _ = std::fs::remove_file(&tpl_path);
        let _ = std::fs::remove_file(&output_path);
    }

    #[test]
    fn test_fill_missing_key_preserves_placeholder() {
        // Keys not in the data map should remain as-is
        let template = make_template(&["present", "missing"]);
        let mut data = HashMap::new();
        data.insert("present".into(), "value".into());
        // "missing" is not in the map

        let filler = OfdTemplateFiller::fill_bytes(&template, &data).unwrap();
        let output = filler.into_bytes();

        let reader = easyofd_reader::OfdReader::from_bytes(&output).unwrap();
        let text = reader.extract_all_text();
        assert!(text.contains("value"));
        assert!(text.contains("{missing}")); // preserved
    }
}
