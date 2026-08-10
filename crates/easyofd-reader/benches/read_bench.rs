//! OFD 读取性能基准测试。

use criterion::{Criterion, criterion_group, criterion_main};
use easyofd_core::{OfdPage, TextObject};
use easyofd_reader::OfdReader;
use easyofd_writer::OfdWriter;

fn make_ofd_bytes(page_count: usize, texts_per_page: usize) -> Vec<u8> {
    let mut writer = OfdWriter::new();
    for i in 0..page_count {
        let mut page = OfdPage::new(210.0, 297.0);
        for j in 0..texts_per_page {
            page.add_text(TextObject::new(
                10.0,
                20.0 + f64::from(u32::try_from(j).unwrap_or(u32::MAX)) * 5.0,
                &format!("Page {i} line {j}"),
            ));
        }
        writer.add_page(page);
    }
    writer.build().unwrap()
}

fn bench_read_small(c: &mut Criterion) {
    let data = make_ofd_bytes(1, 5);
    c.bench_function("read_1_page_5_texts", |b| {
        b.iter(|| {
            let reader = OfdReader::from_bytes(&data).unwrap();
            let _text = reader.extract_all_text();
        });
    });
}

fn bench_read_medium(c: &mut Criterion) {
    let data = make_ofd_bytes(10, 20);
    c.bench_function("read_10_pages_20_texts", |b| {
        b.iter(|| {
            let reader = OfdReader::from_bytes(&data).unwrap();
            let _text = reader.extract_all_text();
        });
    });
}

fn bench_read_large(c: &mut Criterion) {
    let data = make_ofd_bytes(50, 100);
    c.bench_function("read_50_pages_100_texts", |b| {
        b.iter(|| {
            let reader = OfdReader::from_bytes(&data).unwrap();
            let _text = reader.extract_all_text();
        });
    });
}

criterion_group!(benches, bench_read_small, bench_read_medium, bench_read_large);
criterion_main!(benches);
