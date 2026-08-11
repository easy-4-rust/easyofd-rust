//! Stream-read an OFD file page-by-page using `OfdReader::visit_path`.
//!
//! This pattern is ideal for large files: pages are visited in order and
//! never all held in memory at once.
//!
//! Usage:
//!   cargo run --release --example read_with_visitor

use easyofd::{ContentObject, EasyOfd, OfdPage, TextObject};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a demo OFD with several pages.
    let dir = std::env::temp_dir().join("easyofd_example_visitor");
    std::fs::create_dir_all(&dir)?;
    let path = dir.join("multi_page.ofd");

    let pages: Vec<OfdPage> = (1..=5)
        .map(|i| {
            let mut page = OfdPage::new(210.0, 297.0);
            page.add_text(
                TextObject::new(10.0, 20.0, format!("Page {i} heading"))
                    .size(20.0)
                    .bold(),
            );
            page.add_text(TextObject::new(
                10.0,
                50.0,
                format!("This is the body text of page {i}."),
            ));
            page
        })
        .collect();

    EasyOfd::write_pages_to(&path, pages)?;
    println!("Created demo OFD with 5 pages at: {}\n", path.display());

    // --- Visit all pages ---
    println!("=== Visiting all pages ===");
    let total = easyofd::OfdReader::visit_path(
        &path,
        easyofd::ReadOptions::default(),
        |page_number, page| {
            let text_count = page
                .content
                .iter()
                .filter(|o| matches!(o, ContentObject::Text(_)))
                .count();
            println!("  Page {page_number}: {text_count} text object(s)");
            Ok(())
        },
    )?;
    println!("Visited {total} pages.\n");

    // --- Visit only pages 2..=4 ---
    println!("=== Visiting pages 2..=4 only ===");
    let range_total = easyofd::OfdReader::visit_path(
        &path,
        easyofd::ReadOptions {
            first_page: Some(2),
            last_page: Some(4),
            ..easyofd::ReadOptions::default()
        },
        |page_number, page| {
            let texts: Vec<&str> = page
                .content
                .iter()
                .filter_map(|o| {
                    if let ContentObject::Text(t) = o {
                        Some(t.text.as_str())
                    } else {
                        None
                    }
                })
                .collect();
            println!("  Page {page_number}: {texts:?}");
            Ok(())
        },
    )?;
    println!("Visited {range_total} pages in range.\n");

    // --- Same via the fluent builder API ---
    println!("=== Fluent builder (EasyOfd::read_pages) ===");
    let mut word_count = 0usize;
    let count = EasyOfd::read_pages(&path).do_read(|_page_number, page| {
        for obj in &page.content {
            if let ContentObject::Text(t) = obj {
                word_count += t.text.split_whitespace().count();
            }
        }
        Ok(())
    })?;
    println!("Scanned {count} pages, total words: {word_count}");

    // Clean up.
    let _ = std::fs::remove_dir_all(dir);
    Ok(())
}
