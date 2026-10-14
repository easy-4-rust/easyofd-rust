//! SVG文字的变换、填充与描边。
use crate::font::glyph_outline::GlyphOutline;
use crate::utils::text_layout::layout_text_run;
use easyofd_core::{OfdError, OfdResult};
use easyofd_reader::TextRenderRun;
use std::fmt::Write;
/// 生成真实glyph路径或无嵌入字体的text，坐标单位为毫米。
pub(crate) fn render(run: &TextRenderRun) -> OfdResult<String> {
    let size = run.text.size.unwrap_or(12. * 25.4 / 72.);
    if !size.is_finite() || size <= 0. {
        return Err(OfdError::Conversion("文字字号无效".into()));
    }
    let face = run
        .font_data
        .as_deref()
        .map(|d| {
            crate::font::font_face::FontFace::parse(d, 0)
                .map_err(|_| OfdError::Conversion("字体无效".into()))
        })
        .transpose()?;
    let advances: Vec<f64> = run
        .text
        .text_codes
        .iter()
        .flat_map(|c| c.content.chars())
        .map(|c| {
            face.as_ref()
                .and_then(|f| {
                    f.glyph_index(c).and_then(|g| {
                        f.glyph_hor_advance(g)
                            .map(|a| a as f64 / f.units_per_em() as f64 * size)
                    })
                })
                .unwrap_or(size * 0.5)
        })
        .collect();
    let positions = layout_text_run(run, &advances)?;
    let color = |value: u32| format!("#{:06X}", value & 0x00ff_ffff);
    let fill = if run.text.fill {
        color(run.text.fill_color.unwrap_or(0))
    } else {
        "none".into()
    };
    let stroke = if run.text.stroke {
        color(run.text.stroke_color.unwrap_or(0))
    } else {
        "none".into()
    };
    let mut out = String::new();
    for p in positions {
        let m = p.linear_ctm;
        let [x, y] = p.origin_mm;
        if let Some(face) = &face {
            let id = if let Some(g) = p.glyph_id {
                if g >= u32::from(face.number_of_glyphs()) {
                    return Err(OfdError::Conversion("glyph越界".into()));
                }
                crate::font::font_face::GlyphId(g as u16)
            } else {
                face.glyph_index(p.character)
                    .ok_or_else(|| OfdError::Conversion(format!("字体缺少字形: {}", p.character)))?
            };
            let outline = GlyphOutline::load(face, id)?;
            let s = size / f64::from(face.units_per_em());
            let _ = writeln!(
                out,
                "<g transform=\"matrix({} {} {} {} {x} {y})\"><path d=\"{}\" fill=\"{fill}\" stroke=\"{stroke}\" stroke-width=\"{}\"/></g>",
                m[0] * s,
                m[1] * s,
                -m[2] * s,
                -m[3] * s,
                outline.svg,
                run.line_width_mm / s
            );
        } else {
            if p.glyph_id.is_some() {
                return Err(OfdError::Conversion("显式glyph需要嵌入字体".into()));
            }
            let text = p
                .character
                .to_string()
                .replace('&', "&amp;")
                .replace('<', "&lt;")
                .replace('>', "&gt;");
            let _ = writeln!(
                out,
                "<text transform=\"matrix({} {} {} {} {x} {y})\" font-size=\"{size}\" fill=\"{fill}\" stroke=\"{stroke}\" stroke-width=\"{}\">{text}</text>",
                m[0], m[1], m[2], m[3], run.line_width_mm
            );
        }
    }
    Ok(out)
}
