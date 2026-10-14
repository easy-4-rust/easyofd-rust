//! SVG文字描边阳性/阴性矩阵。
use easyofd_convert::exporter::{Exporter, SvgExporter};
#[test]
fn honors_text_stroke_and_fill_switches() {
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
            std::env::temp_dir().join(format!("easyofd-{name}-{}.svg", std::process::id()));
        SvgExporter::with_defaults()
            .convert(&root.join(format!("{name}.ofd")), &target)
            .unwrap();
        let s = std::fs::read_to_string(&target).unwrap();
        assert_eq!(s.contains("stroke=\"#FF0000\""), stroke, "{name}: {s}");
        assert_eq!(s.contains("fill=\"none\""), !fill, "{name}: {s}");
        let _ = std::fs::remove_file(target);
    }
}
