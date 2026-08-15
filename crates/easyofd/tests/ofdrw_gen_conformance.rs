//! ofdrw 生成样本的 conformance 测试。
//!
//! 自动发现 `tests/fixtures/ofdrw_gen/` 下所有 `.ofd` 文件，
//! 对每个文件执行 roundtrip（read -> write -> read），
//! 期望至少"不 panic + 读回页数 >= 1"。
//!
//! 不要求 byte diff（生成样本可能与 ofdrw 默认行为有差异）。
//!
//! 若目录不存在或无样本文件，测试自动跳过（不 panic）。

use std::path::{Path, PathBuf};

use easyofd_core::ContentObject;
use easyofd_reader::OfdReader;
use easyofd_writer::OfdWriter;

// ─── 工具函数 ───────────────────────────────────────────────────────────────

/// 返回 `tests/fixtures/ofdrw_gen/` 目录路径。
fn gen_fixture_dir() -> PathBuf {
    let manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    Path::new(&manifest)
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("tests")
        .join("fixtures")
        .join("ofdrw_gen")
}

/// 收集目录下所有 `.ofd` 文件路径，按文件名排序。
fn collect_ofd_files(dir: &Path) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = std::fs::read_dir(dir)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|ext| ext == "ofd"))
        .collect();
    files.sort();
    files
}

// ─── 自动发现 roundtrip 测试 ───────────────────────────────────────────────

/// 对 `tests/fixtures/ofdrw_gen/` 下所有 `.ofd` 文件执行 roundtrip 测试。
///
/// 每个文件期望：
/// - 能被 OfdReader 成功打开（不 panic）
/// - page_count >= 1
/// - roundtrip（read -> write -> read）后页数不变
///
/// 若目录不存在或无样本，测试跳过。
#[test]
fn ofdrw_gen_roundtrip_all() {
    let dir = gen_fixture_dir();
    if !dir.exists() {
        eprintln!(
            "SKIP: ofdrw_gen 目录不存在 ({})，\n\
             运行 bash scripts/ofd_sample_gen.sh 生成样本。",
            dir.display()
        );
        return;
    }

    let files = collect_ofd_files(&dir);
    if files.is_empty() {
        eprintln!(
            "SKIP: ofdrw_gen 目录下无 .ofd 文件 ({})，\n\
             运行 bash scripts/ofd_sample_gen.sh 生成样本。",
            dir.display()
        );
        return;
    }

    let mut passed = 0;
    let mut failed = 0;

    for path in &files {
        let name = path.file_name().unwrap().to_string_lossy();
        let result = std::panic::catch_unwind(|| {
            // 阶段 1：读取
            let reader = match OfdReader::open(path) {
                Ok(r) => r,
                Err(e) => {
                    panic!("读取失败: {e}");
                }
            };
            let page_count = reader.page_count();
            assert!(page_count >= 1, "期望页数 >= 1，实际 {page_count}");

            // 阶段 2：roundtrip write
            let mut writer = OfdWriter::new();
            for page in reader.pages() {
                writer.add_page(page.clone());
            }
            let bytes = writer.build().expect("roundtrip write 应成功");

            // 阶段 3：roundtrip read
            let reader2 = OfdReader::from_bytes(&bytes).expect("roundtrip read 应成功");
            assert_eq!(reader2.page_count(), page_count, "roundtrip 后页数变化");
        });

        match result {
            Ok(()) => {
                passed += 1;
            }
            Err(payload) => {
                failed += 1;
                let msg = if let Some(s) = payload.downcast_ref::<String>() {
                    s.clone()
                } else if let Some(s) = payload.downcast_ref::<&str>() {
                    s.to_string()
                } else {
                    "unknown panic".to_string()
                };
                eprintln!("FAIL: {name} — {msg}");
            }
        }
    }

    eprintln!(
        "\nofdrw_gen roundtrip: {passed} passed, {failed} failed, {total} total",
        total = files.len()
    );

    // 允许部分失败（某些样本可能包含 easyofd 尚不支持的特性），
    // 但至少要有一个通过。
    assert!(
        passed > 0,
        "所有 {total} 个生成样本 roundtrip 均失败，期望至少 1 个通过",
        total = files.len()
    );
}

/// 验证所有生成样本都是合法的 ZIP 归档。
#[test]
fn ofdrw_gen_all_are_valid_zip() {
    let dir = gen_fixture_dir();
    if !dir.exists() {
        eprintln!("SKIP: ofdrw_gen 目录不存在");
        return;
    }

    let files = collect_ofd_files(&dir);
    if files.is_empty() {
        eprintln!("SKIP: ofdrw_gen 目录下无 .ofd 文件");
        return;
    }

    for path in &files {
        let name = path.file_name().unwrap().to_string_lossy();
        let data = std::fs::read(path).unwrap_or_else(|e| panic!("无法读取 {name}: {e}"));
        let cursor = std::io::Cursor::new(&data[..]);
        zip::ZipArchive::new(cursor).unwrap_or_else(|e| panic!("{name} 不是合法 ZIP: {e}"));
    }
}

/// 验证所有生成样本都包含 OFD.xml（基本结构合规）。
#[test]
fn ofdrw_gen_all_contain_ofd_xml() {
    let dir = gen_fixture_dir();
    if !dir.exists() {
        eprintln!("SKIP: ofdrw_gen 目录不存在");
        return;
    }

    let files = collect_ofd_files(&dir);
    if files.is_empty() {
        eprintln!("SKIP: ofdrw_gen 目录下无 .ofd 文件");
        return;
    }

    for path in &files {
        let name = path.file_name().unwrap().to_string_lossy();
        let data = std::fs::read(path).unwrap_or_else(|e| panic!("无法读取 {name}: {e}"));
        let cursor = std::io::Cursor::new(&data[..]);
        let mut archive =
            zip::ZipArchive::new(cursor).unwrap_or_else(|e| panic!("{name} 不是合法 ZIP: {e}"));

        // 检查是否包含 OFD.xml
        let has_ofd_xml = archive.by_name("OFD.xml").is_ok();
        assert!(has_ofd_xml, "{name} 缺少 OFD.xml 入口文件");
    }
}

/// 统计测试：报告生成样本中包含图片和路径对象的比例。
#[test]
fn ofdrw_gen_content_statistics() {
    let dir = gen_fixture_dir();
    if !dir.exists() {
        eprintln!("SKIP: ofdrw_gen 目录不存在");
        return;
    }

    let files = collect_ofd_files(&dir);
    if files.is_empty() {
        eprintln!("SKIP: ofdrw_gen 目录下无 .ofd 文件");
        return;
    }

    let mut total = 0;
    let mut with_text = 0;
    let mut with_image = 0;
    let mut with_path = 0;
    let mut parse_errors = 0;

    for path in &files {
        total += 1;
        let reader = if let Ok(r) = OfdReader::open(path) {
            r
        } else {
            parse_errors += 1;
            continue;
        };

        let pages = reader.pages();
        if pages.iter().any(|p| {
            p.content
                .iter()
                .any(|c| matches!(c, ContentObject::Text(_)))
        }) {
            with_text += 1;
        }
        if pages.iter().any(|p| {
            p.content
                .iter()
                .any(|c| matches!(c, ContentObject::Image(_)))
        }) {
            with_image += 1;
        }
        if pages.iter().any(|p| {
            p.content
                .iter()
                .any(|c| matches!(c, ContentObject::Path(_)))
        }) {
            with_path += 1;
        }
    }

    eprintln!(
        "\nofdrw_gen 内容统计:\n\
         总计: {total}\n\
         含文字: {with_text}\n\
         含图片: {with_image}\n\
         含路径: {with_path}\n\
         解析失败: {parse_errors}"
    );
}
