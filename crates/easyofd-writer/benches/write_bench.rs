//! OFD 写入性能基准测试。
//!
//! 测试不同规模页面的写入性能。

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use easyofd_core::{OfdPage, TextObject};
use easyofd_writer::OfdWriter;

fn bench_single_page(c: &mut Criterion) {
    c.bench_function("write_single_page", |b| {
        b.iter(|| {
            let mut writer = OfdWriter::new();
            let mut page = OfdPage::new(210.0, 297.0);
            page.add_text(TextObject::new(10.0, 20.0, "Hello OFD"));
            writer.add_page(page);
            writer.build().unwrap()
        });
    });
}

fn bench_multi_page(c: &mut Criterion) {
    let mut group = c.benchmark_group("write_multi_page");
    for page_count in [1, 5, 10, 20] {
        group.bench_with_input(
            BenchmarkId::from_parameter(page_count),
            &page_count,
            |b, &count| {
                b.iter(|| {
                    let mut writer = OfdWriter::new();
                    for i in 0..count {
                        let mut page = OfdPage::new(210.0, 297.0);
                        page.add_text(TextObject::new(10.0, 20.0, &format!("Page {i}")));
                        writer.add_page(page);
                    }
                    writer.build().unwrap()
                });
            },
        );
    }
    group.finish();
}

fn bench_text_density(c: &mut Criterion) {
    let mut group = c.benchmark_group("write_text_density");
    for text_count in [10, 50, 100, 500] {
        group.bench_with_input(
            BenchmarkId::from_parameter(text_count),
            &text_count,
            |b, &count| {
                b.iter(|| {
                    let mut writer = OfdWriter::new();
                    let mut page = OfdPage::new(210.0, 297.0);
                    for i in 0..count {
                        let y = 20.0 + f64::from(i) * 5.0;
                        page.add_text(TextObject::new(
                            10.0,
                            y,
                            &format!("Text line {i}"),
                        ));
                    }
                    writer.add_page(page);
                    writer.build().unwrap()
                });
            },
        );
    }
    group.finish();
}

criterion_group!(benches, bench_single_page, bench_multi_page, bench_text_density);
criterion_main!(benches);
