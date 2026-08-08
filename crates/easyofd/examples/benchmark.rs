//! 可重复的 Reader 与 OFD→Markdown 基准入口。
//!
//! 运行：`cargo run --release -p easyofd --example benchmark -- 10000`

use std::hint::black_box;
use std::time::Instant;

use easyofd::{EasyOfd, OfdPage, OfdResult, TextObject};

fn main() -> OfdResult<()> {
    let page_count = std::env::args()
        .nth(1)
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(1_000);
    let root = std::env::temp_dir().join("easyofd-benchmark");
    std::fs::create_dir_all(&root)?;
    let source = root.join(format!("pages-{page_count}.ofd"));

    let file = std::fs::File::create(&source)?;
    let mut writer = EasyOfd::stream_writer(file);
    for number in 1..=page_count {
        let mut page = OfdPage::new(210.0, 297.0);
        page.add_text(TextObject::new(
            10.0,
            10.0,
            format!("benchmark page {number}"),
        ));
        writer.write_page(page)?;
    }
    writer.finish()?;
    let input_bytes = std::fs::metadata(&source)?.len();

    let read_started = Instant::now();
    let mut text_bytes = 0_usize;
    let visited = EasyOfd::read_pages(&source).do_read(|_, page| {
        for object in page.content {
            if let easyofd::ContentObject::Text(text) = object {
                text_bytes += text.text.len();
            }
        }
        Ok(())
    })?;
    let read_millis = read_started.elapsed().as_millis();

    let markdown_started = Instant::now();
    let report = EasyOfd::to_markdown(&source).convert_to(std::io::sink())?;
    let markdown_millis = markdown_started.elapsed().as_millis();

    black_box(text_bytes);
    println!(
        "{{\"scenario\":\"ofd_text_pages\",\"pages\":{page_count},\"input_bytes\":{input_bytes},\"visited_pages\":{visited},\"text_bytes\":{text_bytes},\"read_millis\":{read_millis},\"markdown_pages\":{},\"markdown_millis\":{markdown_millis}}}",
        report.pages_converted
    );
    Ok(())
}
