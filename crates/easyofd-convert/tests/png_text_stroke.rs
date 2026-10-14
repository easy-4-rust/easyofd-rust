//! PNG文字描边阳性与阴性像素矩阵。
use easyofd_convert::converter::AWTMaker;
#[test]
fn paints_stroke_only_when_enabled() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/upstream_20261014");
    for (name, stroke, fill) in [
        ("stroke_false", false, true),
        ("stroke_omitted", false, true),
        ("stroke_true", true, true),
        ("stroke_explicit", true, true),
        ("stroke_only", true, false),
    ] {
        let target =
            std::env::temp_dir().join(format!("easyofd-{name}-{}.png", std::process::id()));
        AWTMaker::from_file(root.join(format!("{name}.ofd")))
            .unwrap()
            .convert(&target)
            .unwrap();
        let image = image::open(&target).unwrap().into_rgba8();
        let red = image
            .pixels()
            .filter(|p| {
                i32::from(p[0]) > i32::from(p[1]) + 50 && i32::from(p[0]) > i32::from(p[2]) + 50
            })
            .count();
        let black = image
            .pixels()
            .filter(|p| p[0] < 50 && p[1] < 50 && p[2] < 50)
            .count();
        assert_eq!(red > 0, stroke, "{name}: red={red}");
        assert_eq!(black > 0, fill, "{name}: black={black}");
        let _ = std::fs::remove_file(target);
    }
}
