//! 实际PDF文字矩阵回归。
use easyofd_convert::{ConvertOptions, ofd_to_pdf};
#[test]
fn actual_pdf_contains_per_glyph_transformed_origins() {
    let source = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/upstream_20261014/ab.ofd");
    let target = std::env::temp_dir().join(format!("easyofd-ab-{}.pdf", std::process::id()));
    ofd_to_pdf(&source, &target, &ConvertOptions::default()).unwrap();
    let doc = lopdf::Document::load(&target).unwrap();
    let id = *doc.get_pages().values().next().unwrap();
    let content = lopdf::content::Content::decode(&doc.get_page_content(id)).unwrap();
    let matrices: Vec<_> = content
        .operations
        .iter()
        .filter(|op| op.operator == "Tm")
        .collect();
    assert_eq!(matrices.len(), 2);
    let x = f64::from(matrices[0].operands[4].as_float().unwrap());
    let y = f64::from(matrices[0].operands[5].as_float().unwrap());
    assert!((x - 112. * 72. / 25.4).abs() < 0.01);
    assert!((y - (1000. - 226.) * 72. / 25.4).abs() < 0.01);
    let _ = std::fs::remove_file(target);
}

#[test]
fn embedded_font_missing_character_returns_error() {
    use std::io::{Cursor, Read, Write};
    let source = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/upstream_20261014/stroke_true.ofd");
    let mut archive = zip::ZipArchive::new(Cursor::new(std::fs::read(source).unwrap())).unwrap();
    let mut out = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for i in 0..archive.len() {
        let mut f = archive.by_index(i).unwrap();
        let name = f.name().to_owned();
        let mut bytes = Vec::new();
        f.read_to_end(&mut bytes).unwrap();
        if name.ends_with("P.xml") {
            bytes = String::from_utf8(bytes)
                .unwrap()
                .replace(">/</TextCode>", ">🦀</TextCode>")
                .into_bytes();
        }
        out.start_file(name, zip::write::SimpleFileOptions::default())
            .unwrap();
        out.write_all(&bytes).unwrap();
    }
    let ofd = std::env::temp_dir().join(format!("missing-glyph-{}.ofd", std::process::id()));
    let pdf = ofd.with_extension("pdf");
    std::fs::write(&ofd, out.finish().unwrap().into_inner()).unwrap();
    let result = ofd_to_pdf(&ofd, &pdf, &ConvertOptions::default());
    let _ = std::fs::remove_file(ofd);
    let _ = std::fs::remove_file(pdf);
    assert!(
        result.is_err(),
        "embedded missing glyph must not silently disappear"
    );
}

#[test]
fn actual_vertical_pdf_has_six_nonoverlapping_origins() {
    let source = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/upstream_20261014/vertical.ofd");
    let target = std::env::temp_dir().join(format!("easyofd-vertical-{}.pdf", std::process::id()));
    ofd_to_pdf(source, &target, &ConvertOptions::default()).unwrap();
    let doc = lopdf::Document::load(&target).unwrap();
    let id = *doc.get_pages().values().next().unwrap();
    let content = lopdf::content::Content::decode(&doc.get_page_content(id)).unwrap();
    let ms: Vec<_> = content
        .operations
        .iter()
        .filter(|op| op.operator == "Tm")
        .collect();
    assert_eq!(ms.len(), 6);
    for (i, m) in ms.iter().enumerate() {
        let x = f64::from(m.operands[4].as_float().unwrap());
        let y = f64::from(m.operands[5].as_float().unwrap());
        assert!((x - 208. * 72. / 25.4).abs() < 0.01);
        assert!(
            (y - (140. - 29.5 - 3.18 * f64::from(u32::try_from(i).unwrap())) * 72. / 25.4).abs()
                < 0.01
        );
        assert!(m.operands[0].as_float().unwrap().abs() < f32::EPSILON);
        assert!((m.operands[1].as_float().unwrap() + 1.).abs() < f32::EPSILON);
    }
    let _ = std::fs::remove_file(target);
}
