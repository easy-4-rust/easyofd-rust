//! 导出固定上游样本的实际SVG、PNG、PDF验收产物。
use easyofd_convert::converter::AWTMaker;
use easyofd_convert::exporter::{Exporter, SvgExporter};
use easyofd_convert::{ConvertOptions, ofd_to_pdf};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = std::env::args()
        .nth(1)
        .ok_or("usage: upstream_render OUTPUT")?;
    let output = std::path::Path::new(&output);
    std::fs::create_dir_all(output)?;
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/upstream_20261014");
    for name in [
        "stroke_false",
        "stroke_omitted",
        "stroke_true",
        "stroke_explicit",
        "stroke_only",
        "vertical",
        "ab",
    ] {
        let source = root.join(format!("{name}.ofd"));
        SvgExporter::with_defaults().convert(&source, &output.join(format!("{name}.svg")))?;
        AWTMaker::from_file(&source)?.convert(&output.join(format!("{name}.png")))?;
        ofd_to_pdf(
            &source,
            output.join(format!("{name}.pdf")),
            &ConvertOptions::default(),
        )?;
    }
    Ok(())
}
