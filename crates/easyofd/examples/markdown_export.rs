//! Export an OFD document to Markdown using `EasyOfd::to_markdown()`.
//!
//! Demonstrates both in-memory and streaming-to-file conversion, with
//! loss reporting.
//!
//! Usage:
//!   cargo run --release --example markdown_export

use easyofd::{EasyOfd, OfdPage, TextObject};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a demo OFD with text and a path (line).
    let dir = std::env::temp_dir().join("easyofd_example_markdown");
    std::fs::create_dir_all(&dir)?;
    let ofd_path = dir.join("sample.ofd");

    let mut page1 = OfdPage::new(210.0, 297.0);
    page1.add_text(
        TextObject::new(20.0, 30.0, "Invoice Summary")
            .size(24.0)
            .bold(),
    );
    page1.add_text(TextObject::new(20.0, 60.0, "Date: 2026-08-10"));
    page1.add_text(TextObject::new(20.0, 80.0, "Total: $1,234.56"));
    page1.add_path(easyofd::PathObject::hline(20.0, 100.0, 170.0));

    let mut page2 = OfdPage::new(210.0, 297.0);
    page2.add_text(
        TextObject::new(20.0, 30.0, "Item Details")
            .size(18.0)
            .bold(),
    );
    page2.add_text(TextObject::new(20.0, 55.0, "1. Widget A -- $100.00"));
    page2.add_text(TextObject::new(20.0, 75.0, "2. Widget B -- $1,134.56"));

    EasyOfd::write_pages_to(&ofd_path, vec![page1, page2])?;
    println!("Created demo OFD at: {}\n", ofd_path.display());

    // --- In-memory conversion ---
    println!("=== In-memory Markdown ===");
    let result = EasyOfd::to_markdown(&ofd_path).do_convert()?;
    println!("Pages converted: {}", result.report.pages_converted);
    println!("Losses: {}", result.report.losses.len());
    println!("Warnings: {}", result.report.warnings.len());
    println!("\n--- Markdown output ---\n{}", result.markdown);

    // --- Stream to file ---
    let md_path = dir.join("output.md");
    println!("=== Streaming to file ===");
    let report = EasyOfd::to_markdown(&ofd_path).convert_to(std::fs::File::create(&md_path)?)?;
    println!("Wrote Markdown to: {}", md_path.display());
    println!("Pages converted: {}", report.pages_converted);
    if !report.losses.is_empty() {
        println!("Conversion losses:");
        for loss in &report.losses {
            println!(
                "  - [{}] page={}, policy={}",
                loss.feature, loss.page, loss.policy
            );
        }
    }

    // Clean up.
    let _ = std::fs::remove_dir_all(dir);
    Ok(())
}
