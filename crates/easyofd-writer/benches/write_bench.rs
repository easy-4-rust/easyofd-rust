//! OFD 写入性能基准测试。
//!
//! 覆盖场景：纯文本（30 行/页）、文本+图片（1 图/页）、roundtrip（写→读→写）。
//! 样本矩阵：1 页 / 10 页 / 100 页。

// criterion_group! 宏展开会产生无文档函数；本地抑制以满足 workspace `-D warnings`。
#![allow(
    missing_docs,
    clippy::needless_borrows_for_generic_args,
    clippy::cast_precision_loss
)]

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use easyofd_core::{ImageObject, OfdPage, TextObject};
use easyofd_reader::OfdReader;
use easyofd_writer::OfdWriter;

/// 合成最小合法 JPEG 数据（SOI + APP0 + EOI）。
fn minimal_jpeg() -> Vec<u8> {
    vec![
        0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46, 0x49, 0x46, 0x00, 0x01, 0x01, 0x00, 0x00,
        0x01, 0x00, 0x01, 0x00, 0x00, 0xFF, 0xD9,
    ]
}

// ═════════════════════════════════════════════════════════════════════════════
// 写入基准：纯文本（30 行/页）
// ═════════════════════════════════════════════════════════════════════════════

fn bench_write_text(c: &mut Criterion) {
    let mut group = c.benchmark_group("write_text_30_lines");
    for &pages in &[1usize, 10, 100] {
        group.bench_with_input(BenchmarkId::from_parameter(pages), &pages, |b, &p| {
            b.iter(|| {
                let mut writer = OfdWriter::new();
                for i in 0..p {
                    let mut page = OfdPage::new(210.0, 297.0);
                    for j in 0..30 {
                        page.add_text(TextObject::new(
                            10.0,
                            20.0 + f64::from(u32::try_from(j).unwrap_or(u32::MAX)) * 9.0,
                            &format!("Page {i} line {j} — easyofd benchmark text payload"),
                        ));
                    }
                    writer.add_page(page);
                }
                let bytes = writer.build().unwrap();
                eprintln!(
                    "[write_text] {p} pages: {:.2} KB",
                    bytes.len() as f64 / 1024.0
                );
                bytes
            });
        });
    }
    group.finish();
}

// ═════════════════════════════════════════════════════════════════════════════
// 写入基准：文本+图片（每页 30 行 + 1 图）
// ═════════════════════════════════════════════════════════════════════════════

fn bench_write_image(c: &mut Criterion) {
    let jpeg = minimal_jpeg();
    let mut group = c.benchmark_group("write_text_plus_image");
    for &pages in &[1usize, 10, 100] {
        group.bench_with_input(BenchmarkId::from_parameter(pages), &pages, |b, &p| {
            b.iter(|| {
                let mut writer = OfdWriter::new();
                for i in 0..p {
                    let mut page = OfdPage::new(210.0, 297.0);
                    for j in 0..30 {
                        page.add_text(TextObject::new(
                            10.0,
                            20.0 + f64::from(u32::try_from(j).unwrap_or(u32::MAX)) * 9.0,
                            &format!("Page {i} line {j}"),
                        ));
                    }
                    page.add_image(ImageObject::jpeg(150.0, 200.0, 30.0, 30.0, jpeg.clone()));
                    writer.add_page(page);
                }
                let bytes = writer.build().unwrap();
                eprintln!(
                    "[write_image] {p} pages: {:.2} KB",
                    bytes.len() as f64 / 1024.0
                );
                bytes
            });
        });
    }
    group.finish();
}

// ═════════════════════════════════════════════════════════════════════════════
// Roundtrip 基准：写 → 读 → 写（30 行/页纯文本）
// ═════════════════════════════════════════════════════════════════════════════

fn bench_roundtrip_text(c: &mut Criterion) {
    let mut group = c.benchmark_group("roundtrip_write_read_write_text");
    for &pages in &[1usize, 10, 100] {
        group.bench_with_input(BenchmarkId::from_parameter(pages), &pages, |b, &p| {
            b.iter(|| {
                // 写
                let mut writer = OfdWriter::new();
                for i in 0..p {
                    let mut page = OfdPage::new(210.0, 297.0);
                    for j in 0..30 {
                        page.add_text(TextObject::new(
                            10.0,
                            20.0 + f64::from(u32::try_from(j).unwrap_or(u32::MAX)) * 9.0,
                            &format!("Page {i} line {j}"),
                        ));
                    }
                    writer.add_page(page);
                }
                let bytes = writer.build().unwrap();
                // 读
                let reader = OfdReader::from_bytes(&bytes).unwrap();
                let parsed_pages = reader.pages().to_vec();
                // 再写
                let mut writer2 = OfdWriter::new();
                for page in &parsed_pages {
                    writer2.add_page(page.clone());
                }
                let _out = writer2.build().unwrap();
            });
        });
    }
    group.finish();
}

// ═════════════════════════════════════════════════════════════════════════════
// Roundtrip 基准：写 → 读 → 写（文本+图片）
// ═════════════════════════════════════════════════════════════════════════════

fn bench_roundtrip_image(c: &mut Criterion) {
    let jpeg = minimal_jpeg();
    let mut group = c.benchmark_group("roundtrip_write_read_write_image");
    for &pages in &[1usize, 10, 100] {
        group.bench_with_input(BenchmarkId::from_parameter(pages), &pages, |b, &p| {
            b.iter(|| {
                let mut writer = OfdWriter::new();
                for i in 0..p {
                    let mut page = OfdPage::new(210.0, 297.0);
                    for j in 0..30 {
                        page.add_text(TextObject::new(
                            10.0,
                            20.0 + f64::from(u32::try_from(j).unwrap_or(u32::MAX)) * 9.0,
                            &format!("Page {i} line {j}"),
                        ));
                    }
                    page.add_image(ImageObject::jpeg(150.0, 200.0, 30.0, 30.0, jpeg.clone()));
                    writer.add_page(page);
                }
                let bytes = writer.build().unwrap();
                let reader = OfdReader::from_bytes(&bytes).unwrap();
                let parsed_pages = reader.pages().to_vec();
                let mut writer2 = OfdWriter::new();
                for page in &parsed_pages {
                    writer2.add_page(page.clone());
                }
                let _out = writer2.build().unwrap();
            });
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_write_text,
    bench_write_image,
    bench_roundtrip_text,
    bench_roundtrip_image
);
criterion_main!(benches);
