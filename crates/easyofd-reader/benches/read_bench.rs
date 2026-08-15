//! OFD 读取性能基准测试。
//!
//! 覆盖场景：纯文本（30 行/页）、文本+图片（1 图/页）、roundtrip（读→写）。
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

/// 生成纯文本 OFD 字节：`page_count` 页，每页 `texts_per_page` 行。
fn make_text_ofd_bytes(page_count: usize, texts_per_page: usize) -> Vec<u8> {
    let mut writer = OfdWriter::new();
    for i in 0..page_count {
        let mut page = OfdPage::new(210.0, 297.0);
        for j in 0..texts_per_page {
            page.add_text(TextObject::new(
                10.0,
                20.0 + f64::from(u32::try_from(j).unwrap_or(u32::MAX)) * 9.0,
                &format!("Page {i} line {j} — easyofd benchmark text payload"),
            ));
        }
        writer.add_page(page);
    }
    writer.build().unwrap()
}

/// 生成带图片的 OFD 字节：`page_count` 页，每页 30 行文字 + 1 张合成 JPEG 图片。
fn make_image_ofd_bytes(page_count: usize) -> Vec<u8> {
    // 合成一个最小合法 JPEG（SOI + EOI），约 40 字节，benchmark 不测解码性能。
    let jpeg_data: Vec<u8> = vec![
        0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46, 0x49, 0x46, 0x00, 0x01, 0x01, 0x00, 0x00,
        0x01, 0x00, 0x01, 0x00, 0x00, 0xFF, 0xD9,
    ];
    let mut writer = OfdWriter::new();
    for i in 0..page_count {
        let mut page = OfdPage::new(210.0, 297.0);
        for j in 0..30 {
            page.add_text(TextObject::new(
                10.0,
                20.0 + f64::from(u32::try_from(j).unwrap_or(u32::MAX)) * 9.0,
                &format!("Page {i} line {j}"),
            ));
        }
        page.add_image(ImageObject::jpeg(
            150.0,
            200.0,
            30.0,
            30.0,
            jpeg_data.clone(),
        ));
        writer.add_page(page);
    }
    writer.build().unwrap()
}

// ═════════════════════════════════════════════════════════════════════════════
// 读取基准：纯文本（30 行/页）
// ═════════════════════════════════════════════════════════════════════════════

fn bench_read_text(c: &mut Criterion) {
    let mut group = c.benchmark_group("read_text_30_lines");
    for &pages in &[1usize, 10, 100] {
        let data = make_text_ofd_bytes(pages, 30);
        let size_mb = data.len() as f64 / (1024.0 * 1024.0);
        group.throughput(criterion::Throughput::Bytes(data.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(pages), &data, |b, d| {
            b.iter(|| {
                let reader = OfdReader::from_bytes(d).unwrap();
                let _text = reader.extract_all_text();
            });
        });
        // 在 group 名称中标注样本大小
        eprintln!(
            "[read_text] {pages} pages: {:.2} KB ({:.4} MB)",
            data.len() as f64 / 1024.0,
            size_mb
        );
    }
    group.finish();
}

// ═════════════════════════════════════════════════════════════════════════════
// 读取基准：文本+图片（每页 30 行 + 1 图）
// ═════════════════════════════════════════════════════════════════════════════

fn bench_read_image(c: &mut Criterion) {
    let mut group = c.benchmark_group("read_text_plus_image");
    for &pages in &[1usize, 10, 100] {
        let data = make_image_ofd_bytes(pages);
        let size_mb = data.len() as f64 / (1024.0 * 1024.0);
        group.throughput(criterion::Throughput::Bytes(data.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(pages), &data, |b, d| {
            b.iter(|| {
                let reader = OfdReader::from_bytes(d).unwrap();
                let _text = reader.extract_all_text();
            });
        });
        eprintln!(
            "[read_image] {pages} pages: {:.2} KB ({:.4} MB)",
            data.len() as f64 / 1024.0,
            size_mb
        );
    }
    group.finish();
}

// ═════════════════════════════════════════════════════════════════════════════
// Roundtrip 基准：读 → 写（30 行/页纯文本）
// ═════════════════════════════════════════════════════════════════════════════

fn bench_roundtrip_text(c: &mut Criterion) {
    let mut group = c.benchmark_group("roundtrip_text_30_lines");
    for &pages in &[1usize, 10, 100] {
        let data = make_text_ofd_bytes(pages, 30);
        group.bench_with_input(BenchmarkId::from_parameter(pages), &data, |b, d| {
            b.iter(|| {
                // 读
                let reader = OfdReader::from_bytes(d).unwrap();
                let parsed_pages = reader.pages().to_vec();
                // 写
                let mut writer = OfdWriter::new();
                for page in &parsed_pages {
                    writer.add_page(page.clone());
                }
                let _out = writer.build().unwrap();
            });
        });
    }
    group.finish();
}

// ═════════════════════════════════════════════════════════════════════════════
// Roundtrip 基准：读 → 写（文本+图片）
// ═════════════════════════════════════════════════════════════════════════════

fn bench_roundtrip_image(c: &mut Criterion) {
    let mut group = c.benchmark_group("roundtrip_text_plus_image");
    for &pages in &[1usize, 10, 100] {
        let data = make_image_ofd_bytes(pages);
        group.bench_with_input(BenchmarkId::from_parameter(pages), &data, |b, d| {
            b.iter(|| {
                let reader = OfdReader::from_bytes(d).unwrap();
                let parsed_pages = reader.pages().to_vec();
                let mut writer = OfdWriter::new();
                for page in &parsed_pages {
                    writer.add_page(page.clone());
                }
                let _out = writer.build().unwrap();
            });
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_read_text,
    bench_read_image,
    bench_roundtrip_text,
    bench_roundtrip_image
);
criterion_main!(benches);
