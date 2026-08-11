//! Read an OFD file and print page count + all text content.
//!
//! Usage:
//!   cargo run --release --example read_simple -- path/to/file.ofd
//!
//! If no path is given, a small demo OFD is created in a temp directory
//! and read back.

use easyofd::EasyOfd;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args().nth(1);

    // If a file path was provided, read it; otherwise create a demo OFD first.
    let ofd_path = if let Some(p) = path {
        std::path::PathBuf::from(p)
    } else {
        let dir = std::env::temp_dir().join("easyofd_example_read_simple");
        std::fs::create_dir_all(&dir)?;
        let demo_path = dir.join("demo.ofd");

        let mut page1 = easyofd::OfdPage::new(210.0, 297.0);
        page1.add_text(easyofd::TextObject::new(20.0, 30.0, "Hello, OFD World!").size(24.0));
        page1.add_text(easyofd::TextObject::new(20.0, 60.0, "This is page one."));

        let mut page2 = easyofd::OfdPage::new(210.0, 297.0);
        page2.add_text(
            easyofd::TextObject::new(20.0, 30.0, "Page Two")
                .size(18.0)
                .bold(),
        );
        page2.add_text(easyofd::TextObject::new(
            20.0,
            60.0,
            "easyofd-rust reads OFD documents with zero unsafe code.",
        ));

        EasyOfd::write_pages_to(&demo_path, vec![page1, page2])?;
        println!("Created demo OFD at: {}", demo_path.display());
        demo_path
    };

    // Use the visitor pattern -- pages are not retained in memory.
    println!("\n--- Visitor-based reading ---");
    let visited = EasyOfd::read_pages(&ofd_path).do_read(|page_number, page| {
        let mut texts = Vec::new();
        for obj in &page.content {
            if let easyofd::ContentObject::Text(t) = obj {
                texts.push(t.text.as_str());
            }
        }
        println!("Page {page_number}: {} text object(s)", texts.len());
        for t in &texts {
            println!("  > {t}");
        }
        Ok(())
    })?;
    println!("Total pages visited: {visited}");

    // Full-load reading via OfdReader.
    println!("\n--- Full-load reading ---");
    let reader = EasyOfd::read(&ofd_path)?;
    println!("Page count : {}", reader.page_count());
    println!("All text   :\n{}", reader.extract_all_text());

    Ok(())
}
