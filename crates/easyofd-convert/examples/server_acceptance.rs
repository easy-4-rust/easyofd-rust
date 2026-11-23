//! 服务端并发转换验收烟测；真实公开OFD输入，不替代业务流量/SLA验收。
use easyofd_convert::converter::AWTMaker;
use easyofd_convert::exporter::{Exporter, SvgExporter};
use easyofd_convert::{ConvertOptions, ofd_to_pdf};
use easyofd_reader::{OfdReader, ReadOptions};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/upstream_20261014");
    let out =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/server-acceptance");
    std::fs::create_dir_all(&out)?;
    let start = std::time::Instant::now();
    let mut handles = Vec::new();
    for worker in 0..4 {
        let root = root.clone();
        let out = out.clone();
        handles.push(std::thread::spawn(move || -> Result<usize, String> {
            let mut outputs = 0;
            for name in [
                "ab",
                "vertical",
                "stroke_false",
                "stroke_omitted",
                "stroke_true",
                "stroke_explicit",
                "stroke_only",
            ] {
                let source = root.join(format!("{name}.ofd"));
                let mut limits = ReadOptions::default();
                limits.package_limits.max_entries = 4096;
                limits.package_limits.max_entry_uncompressed_size = 16 * 1024 * 1024;
                limits.package_limits.max_total_uncompressed_size = 64 * 1024 * 1024;
                limits.package_limits.max_compression_ratio = 200;
                let reader =
                    OfdReader::open_with_options(&source, limits).map_err(|e| e.to_string())?;
                if reader.page_count() != 1 {
                    return Err("样本页数错误".into());
                }
                let prefix = format!("{worker}-{name}");
                let svg = out.join(format!("{prefix}.svg"));
                SvgExporter::new(ConvertOptions::default())
                    .convert(&source, &svg)
                    .map_err(|e| e.to_string())?;
                if !std::fs::read_to_string(&svg)
                    .map_err(|e| e.to_string())?
                    .contains("<svg")
                {
                    return Err("SVG输出错误".into());
                }
                let pdf = out.join(format!("{prefix}.pdf"));
                ofd_to_pdf(&source, &pdf, &ConvertOptions::default()).map_err(|e| e.to_string())?;
                let document = lopdf::Document::load(&pdf).map_err(|e| e.to_string())?;
                if document.get_pages().len() != 1 {
                    return Err("PDF输出页数错误".into());
                }
                outputs += 2;
                // 5个嵌入字体描边样本跨系统不依赖系统字体；坐标样本已由PDF矩阵测试验收。
                if name.starts_with("stroke_") {
                    let png = out.join(format!("{prefix}.png"));
                    AWTMaker::from_file(&source)
                        .map_err(|e| e.to_string())?
                        .convert(&png)
                        .map_err(|e| e.to_string())?;
                    let image = image::open(png).map_err(|e| e.to_string())?;
                    if image.width() == 0 || image.height() == 0 {
                        return Err("PNG输出尺寸错误".into());
                    }
                    outputs += 1;
                }
            }
            Ok(outputs)
        }));
    }
    let mut total = 0;
    for handle in handles {
        total += handle.join().map_err(|_| "worker panic")??;
    }
    println!(
        "workers=4 input_jobs=28 validated_outputs={total} elapsed_ms={}",
        start.elapsed().as_millis()
    );
    Ok(())
}
