//! 实际PDF路径中的逐字定位。
use crate::pdf_backend as printpdf;
use crate::utils::text_layout::layout_text_run;
use easyofd_core::{OfdError, OfdResult};
use easyofd_reader::TextRenderRun;
use printpdf::{
    IndirectFontRef, PdfDocumentReference, PdfLayerReference, TextMatrix, TextRenderingMode,
};

fn failure(s: impl Into<String>) -> OfdError {
    OfdError::Conversion(s.into())
}
/// 将元数据绘制到PDF，缺少嵌入字体时使用既有缓存字体降级。
pub(crate) fn render(
    doc: &PdfDocumentReference,
    layer: &PdfLayerReference,
    run: &TextRenderRun,
    fallback: &IndirectFontRef,
    height: f64,
) -> OfdResult<()> {
    let size_mm = run.text.size.unwrap_or(12. * 25.4 / 72.);
    if !size_mm.is_finite() || size_mm <= 0. {
        return Err(failure("文字字号无效"));
    }
    let size_pt = size_mm * 72. / 25.4;
    let face = run
        .font_data
        .as_deref()
        .map(|d| ttf_parser::Face::parse(d, 0).map_err(|e| failure(format!("嵌入字体无效: {e:?}"))))
        .transpose()?;
    let font = if let Some(data) = &run.font_data {
        doc.add_external_font(std::io::Cursor::new(data))
            .map_err(|e| failure(format!("PDF字体: {e}")))?
    } else {
        fallback.clone()
    };
    let chars: Vec<char> = run
        .text
        .text_codes
        .iter()
        .flat_map(|c| c.content.chars())
        .collect();
    let advances: Vec<f64> = chars
        .iter()
        .map(|c| {
            face.as_ref()
                .and_then(|f| {
                    f.glyph_index(*c).and_then(|g| {
                        f.glyph_hor_advance(g)
                            .map(|a| a as f64 / f.units_per_em() as f64 * size_mm)
                    })
                })
                .unwrap_or(size_mm * 0.5)
        })
        .collect();
    let glyphs = layout_text_run(run, &advances)?;
    layer.save_graphics_state();
    let rgb = |c: u32| {
        printpdf::Color::Rgb(printpdf::Rgb::new(
            ((c >> 16) & 255) as f32 / 255.,
            ((c >> 8) & 255) as f32 / 255.,
            (c & 255) as f32 / 255.,
            None,
        ))
    };
    layer.set_fill_color(rgb(run.text.fill_color.unwrap_or(0)));
    layer.set_outline_color(rgb(run.text.stroke_color.unwrap_or(0)));
    layer.set_outline_thickness((run.line_width_mm * 72. / 25.4) as f32);
    layer.begin_text_section();
    layer.set_font(&font, size_pt as f32);
    layer.set_text_rendering_mode(match (run.text.fill, run.text.stroke) {
        (true, true) => TextRenderingMode::FillStroke,
        (true, false) => TextRenderingMode::Fill,
        (false, true) => TextRenderingMode::Stroke,
        (false, false) => TextRenderingMode::Invisible,
    });
    for g in glyphs {
        let m = g.linear_ctm;
        let p = g.origin_mm;
        layer.set_text_matrix(TextMatrix::Raw([
            m[0] as f32,
            -m[1] as f32,
            -m[2] as f32,
            m[3] as f32,
            (p[0] * 72. / 25.4) as f32,
            ((height - p[1]) * 72. / 25.4) as f32,
        ]));
        if let Some(id) = g.glyph_id {
            let f = face
                .as_ref()
                .ok_or_else(|| failure("显式glyph需要嵌入字体"))?;
            if id >= u32::from(f.number_of_glyphs()) {
                return Err(failure("glyph索引越界"));
            }
            layer.write_glyph(id as u16, g.character);
        } else {
            if let Some(face) = &face {
                if face.glyph_index(g.character).is_none() {
                    return Err(failure(format!("嵌入字体缺少字形: {}", g.character)));
                }
            }
            layer.write_text(g.character.to_string(), &font);
        }
    }
    layer.end_text_section();
    layer.restore_graphics_state();
    Ok(())
}
