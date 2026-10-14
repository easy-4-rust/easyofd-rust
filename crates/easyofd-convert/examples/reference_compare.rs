//! 对比当前Rust与固定Java修复源码的PNG/SVG描边行为。
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target");
    for name in [
        "stroke_false",
        "stroke_omitted",
        "stroke_true",
        "stroke_explicit",
        "stroke_only",
    ] {
        for (java_suffix, rust_suffix) in [("png", "png"), ("svg", "svg.png")] {
            let j = root
                .join("upstream-reference/output")
                .join(format!("{name}-{java_suffix}/0.png"));
            let r = root
                .join("upstream-output")
                .join(format!("{name}.{rust_suffix}"));
            let count = |p: &std::path::Path| -> Result<usize, image::ImageError> {
                Ok(image::open(p)?
                    .into_rgb8()
                    .pixels()
                    .filter(|v| {
                        i32::from(v[0]) > i32::from(v[1]) + 50
                            && i32::from(v[0]) > i32::from(v[2]) + 50
                    })
                    .count())
            };
            let jr = count(&j)?;
            let rr = count(&r)?;
            let enabled = !matches!(name, "stroke_false" | "stroke_omitted");
            assert_eq!(jr > 0, enabled, "Java {name} {java_suffix}");
            assert_eq!(rr > 0, enabled, "Rust {name} {java_suffix}");
            println!("{name} {java_suffix}: Java red={jr}, Rust red={rr}; behavior matches");
        }
    }
    Ok(())
}
