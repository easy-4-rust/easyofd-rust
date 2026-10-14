//! 转换绘制元数据回归。
use easyofd_reader::OfdReader;
use std::io::{Cursor, Write};
use zip::write::SimpleFileOptions;

fn fixture(attrs: &str, resources: &str) -> Vec<u8> {
    let mut z = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let entries=[("OFD.xml", "<OFD><DocBody><DocInfo><DocID>test</DocID></DocInfo><DocRoot>Doc_0/Document.xml</DocRoot></DocBody></OFD>".to_string()),
    ("Doc_0/Document.xml", "<Document><CommonData><PublicRes>PublicRes.xml</PublicRes></CommonData><Pages><Page ID=\"1\" BaseLoc=\"Pages/P.xml\"/></Pages></Document>".to_string()),
    ("Doc_0/PublicRes.xml",format!("<Res>{resources}</Res>")),
    ("Doc_0/Pages/P.xml",format!("<Page><Content><Layer><TextObject ID=\"2\" Boundary=\"10 20 30 40\" {attrs}><TextCode X=\"1\" Y=\"2\" DeltaX=\"g 2 4\">ABC</TextCode><TextCode Y=\"5\">D</TextCode><CGTransform CodePosition=\"0\" CodeCount=\"1\" GlyphCount=\"1\" Glyphs=\"3\"/></TextObject></Layer></Content></Page>"))];
    for (name, data) in entries {
        z.start_file(name, SimpleFileOptions::default()).unwrap();
        z.write_all(data.as_bytes()).unwrap();
    }
    z.finish().unwrap().into_inner()
}
#[test]
fn preserves_defaults_coordinates_and_glyphs() {
    let r = OfdReader::from_bytes(&fixture("CTM=\"2 0 0 3 100 200\"", "")).unwrap();
    let runs = r.text_render_runs(0).unwrap();
    assert_eq!(runs.len(), 1);
    assert!(!runs[0].text.stroke);
    assert!(runs[0].text.fill);
    assert_eq!(runs[0].ctm, Some([2., 0., 0., 3., 100., 200.]));
    assert_eq!(runs[0].text.text_codes.len(), 2);
    assert_eq!(runs[0].text.text_codes[0].delta_x, vec![4., 4.]);
    assert_eq!(runs[0].text.cg_transforms[0].glyphs, vec![3]);
    assert!(r.text_render_runs(1).is_err());
}
#[test]
fn inherits_color_but_not_stroke_enable() {
    for (attr, enabled) in [
        ("", false),
        ("Stroke=\"false\"", false),
        ("Stroke=\"true\"", true),
    ] {
        let r=OfdReader::from_bytes(&fixture(&format!("DrawParam=\"11\" {attr}"),"<DrawParams><DrawParam ID=\"11\"><StrokeColor Value=\"255 0 0\"/></DrawParam></DrawParams>")).unwrap();
        let run = &r.text_render_runs(0).unwrap()[0];
        assert_eq!(run.text.stroke, enabled);
        assert_eq!(run.text.stroke_color, Some(0x00ff_0000));
    }
}
#[test]
fn rejects_bad_drawing_references_and_nonfinite_ctm() {
    assert!(
        OfdReader::from_bytes(&fixture(
            "DrawParam=\"11\"",
            "<DrawParams><DrawParam ID=\"11\" Relative=\"11\"/></DrawParams>"
        ))
        .unwrap()
        .text_render_runs(0)
        .is_err()
    );
    assert!(
        OfdReader::from_bytes(&fixture("DrawParam=\"404\"", ""))
            .unwrap()
            .text_render_runs(0)
            .is_err()
    );
    assert!(
        OfdReader::from_bytes(&fixture("CTM=\"1 0 0 1 NaN 0\"", ""))
            .unwrap()
            .text_render_runs(0)
            .is_err()
    );
}

#[test]
fn missing_image_does_not_shift_following_text_metadata() {
    let data = fixture("Font=\"10\"", "");
    let mut archive = zip::ZipArchive::new(Cursor::new(data)).unwrap();
    let mut out = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        let name = file.name().to_owned();
        let mut bytes = Vec::new();
        std::io::Read::read_to_end(&mut file, &mut bytes).unwrap();
        if name == "Doc_0/Pages/P.xml" {
            bytes = String::from_utf8(bytes)
                .unwrap()
                .replace(
                    "<TextObject",
                    "<ImageObject ResourceID=\"404\" Boundary=\"0 0 1 1\"/><TextObject",
                )
                .into_bytes();
        }
        out.start_file(name, SimpleFileOptions::default()).unwrap();
        out.write_all(&bytes).unwrap();
    }
    let reader = OfdReader::from_bytes(&out.finish().unwrap().into_inner()).unwrap();
    assert_eq!(reader.pages()[0].content.len(), 1);
    assert_eq!(reader.text_render_runs(0).unwrap()[0].content_index, 0);
}

#[test]
fn multilevel_draw_parameter_preserves_parent_stroke_and_child_fill() {
    let resources = "<DrawParams><DrawParam ID=\"1\" LineWidth=\"0.5\"><StrokeColor Value=\"255 0 0\"/></DrawParam><DrawParam ID=\"2\" Relative=\"1\"><FillColor Value=\"0 255 0\"/></DrawParam></DrawParams>";
    let r = OfdReader::from_bytes(&fixture("DrawParam=\"2\" Stroke=\"true\"", resources)).unwrap();
    let run = &r.text_render_runs(0).unwrap()[0];
    assert!((run.line_width_mm - 0.5).abs() < f64::EPSILON);
    assert_eq!(run.text.stroke_color, Some(0x00ff_0000));
    assert_eq!(run.text.fill_color, Some(0x0000_ff00));
}
