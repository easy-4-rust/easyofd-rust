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

#[test]
fn rgba_image_keeps_dimensions_pixels_and_alpha_mask() {
    use easyofd_core::{ImageFormat, ImageObject, OfdPage};
    use easyofd_writer::OfdWriter;
    let pixels = image::RgbaImage::from_raw(2, 1, vec![255, 0, 0, 255, 0, 0, 255, 0]).unwrap();
    let mut png = std::io::Cursor::new(Vec::new());
    pixels.write_to(&mut png, image::ImageFormat::Png).unwrap();
    let mut page = OfdPage::new(100., 100.);
    page.add_image(ImageObject::new(
        10.,
        20.,
        30.,
        40.,
        png.into_inner(),
        ImageFormat::Png,
    ));
    let mut writer = OfdWriter::new();
    writer.add_page(page);
    let ofd = std::env::temp_dir().join(format!("easyofd-rgba-{}.ofd", std::process::id()));
    let target = ofd.with_extension("pdf");
    std::fs::write(&ofd, writer.build().unwrap()).unwrap();
    ofd_to_pdf(&ofd, &target, &ConvertOptions::default()).unwrap();
    let pdf = lopdf::Document::load(&target).unwrap();
    let image = pdf
        .objects
        .values()
        .filter_map(|object| object.as_stream().ok())
        .find(|stream| {
            stream
                .dict
                .get(b"Subtype")
                .is_ok_and(|value| value.as_name().ok() == Some(b"Image"))
                && stream
                    .dict
                    .get(b"ColorSpace")
                    .is_ok_and(|value| value.as_name().ok() == Some(b"DeviceRGB"))
        })
        .unwrap();
    assert_eq!(image.dict.get(b"Width").unwrap().as_i64().unwrap(), 2);
    assert_eq!(image.dict.get(b"Height").unwrap().as_i64().unwrap(), 1);
    assert_eq!(
        image.decompressed_content().unwrap(),
        vec![255, 0, 0, 0, 0, 255]
    );
    let mask_id = image
        .dict
        .get(b"SMask")
        .expect("RGBA transparency must be retained")
        .as_reference()
        .unwrap();
    let mask = pdf.get_object(mask_id).unwrap().as_stream().unwrap();
    assert_eq!(mask.decompressed_content().unwrap(), vec![255, 0]);
    let id = pdf.get_pages()[&1];
    let content = lopdf::content::Content::decode(&pdf.get_page_content(id)).unwrap();
    let matrix = content
        .operations
        .iter()
        .find(|op| op.operator == "cm")
        .unwrap();
    for (index, expected) in [
        (0, 30. * 72. / 25.4),
        (3, 40. * 72. / 25.4),
        (4, 10. * 72. / 25.4),
        (5, 40. * 72. / 25.4),
    ] {
        assert!((f64::from(matrix.operands[index].as_float().unwrap()) - expected).abs() < 0.001);
    }
    std::fs::remove_file(ofd).unwrap();
    std::fs::remove_file(target).unwrap();
}

#[test]
fn cgtransform_glyph_is_embedded_and_preserves_unicode_text() {
    use std::io::{Cursor, Read, Write};
    let source = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/upstream_20261014/stroke_true.ofd");
    let mut archive = zip::ZipArchive::new(Cursor::new(std::fs::read(source).unwrap())).unwrap();
    let mut font_bytes = Vec::new();
    archive
        .by_name("Doc_0/Res/font.ttf")
        .unwrap()
        .read_to_end(&mut font_bytes)
        .unwrap();
    let gid = ttf_parser::Face::parse(&font_bytes, 0)
        .unwrap()
        .glyph_index('/')
        .unwrap()
        .0;
    let face = ttf_parser::Face::parse(&font_bytes, 0).unwrap();
    let advance = f64::from(face.glyph_hor_advance(ttf_parser::GlyphId(gid)).unwrap())
        / f64::from(face.units_per_em())
        * 20.;
    let mut out = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        let name = file.name().to_owned();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        if name.ends_with("P.xml") {
            bytes = String::from_utf8(bytes).unwrap().replace("Y=\"20\"", &format!("Y=\"20\" DeltaX=\"{advance}\"")).replace(">/</TextCode>", &format!(">中/</TextCode><CGTransform CodePosition=\"0\" CodeCount=\"2\" GlyphCount=\"2\" Glyphs=\"{gid} {gid}\"/>" )).into_bytes();
        }
        out.start_file(name, zip::write::SimpleFileOptions::default())
            .unwrap();
        out.write_all(&bytes).unwrap();
    }
    let ofd = std::env::temp_dir().join(format!("easyofd-cgtransform-{}.ofd", std::process::id()));
    let pdf_path = ofd.with_extension("pdf");
    std::fs::write(&ofd, out.finish().unwrap().into_inner()).unwrap();
    ofd_to_pdf(&ofd, &pdf_path, &ConvertOptions::default()).unwrap();
    if let Some(dir) = std::env::var_os("EASYOFD_PDF_REVIEW_DIR") {
        std::fs::copy(
            &pdf_path,
            std::path::Path::new(&dir).join("glyph-collision.pdf"),
        )
        .unwrap();
    }
    let pdf = lopdf::Document::load(&pdf_path).unwrap();
    assert_eq!(
        pdf.extract_text_chunks(&[1])
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
            .concat(),
        "中/\n"
    );
    let content =
        lopdf::content::Content::decode(&pdf.get_page_content(pdf.get_pages()[&1])).unwrap();
    assert!(content.operations.iter().any(|op| op.operator == "Tj"));
    let fonts = pdf.get_page_fonts(pdf.get_pages()[&1]).unwrap();
    for operation in content.operations.iter().filter(|op| op.operator == "Tf") {
        assert!(
            fonts.contains_key(operation.operands[0].as_name().unwrap()),
            "each glyph font must be registered"
        );
    }
    let matrix = content
        .operations
        .iter()
        .find(|op| op.operator == "Tm")
        .unwrap();
    assert!((f64::from(matrix.operands[4].as_float().unwrap()) - 10. * 72. / 25.4).abs() < 0.01);
    assert!((f64::from(matrix.operands[5].as_float().unwrap()) - 110. * 72. / 25.4).abs() < 0.01);
    std::fs::remove_file(ofd).unwrap();
    std::fs::remove_file(pdf_path).unwrap();
}
