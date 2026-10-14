//! 按OFD分段定位文字，区分点与位移向量。
use super::point_util::ctm_transform_point;
use crate::point::GlyphPosition;
use easyofd_core::{OfdError, OfdResult};
use easyofd_reader::TextRenderRun;

fn invalid(message: &str) -> OfdError {
    OfdError::Conversion(message.to_owned())
}
/// 参数为绘制信息及每字符字体备用步距（mm），返回最终页面字形位置。
/// 不合法矩阵、步距或字形映射返回错误。
pub fn layout_text_run(run: &TextRenderRun, advances_mm: &[f64]) -> OfdResult<Vec<GlyphPosition>> {
    let text = &run.text;
    let count: usize = text
        .text_codes
        .iter()
        .map(|c| c.content.chars().count())
        .sum();
    if advances_mm.len() != count || advances_mm.iter().any(|n| !n.is_finite() || *n < 0.) {
        return Err(invalid("字体步距长度或数值无效"));
    }
    let b: Vec<f64> = text
        .boundary
        .split_whitespace()
        .map(|s| s.parse().map_err(|_| invalid("Boundary无效")))
        .collect::<OfdResult<_>>()?;
    if b.len() != 4 || b.iter().any(|n| !n.is_finite()) {
        return Err(invalid("Boundary无效"));
    }
    let ctm = run.ctm.unwrap_or([1., 0., 0., 1., 0., 0.]);
    if ctm.iter().any(|n| !n.is_finite()) {
        return Err(invalid("CTM无效"));
    }
    let mut glyphs = vec![None; count];
    for cg in &text.cg_transforms {
        let start = cg.code_position.unwrap_or(0) as usize;
        let codes = cg.code_count.unwrap_or(1) as usize;
        let n = cg.glyph_count.unwrap_or(1) as usize;
        if codes != n
            || n != cg.glyphs.len()
            || start.checked_add(codes).is_none_or(|end| end > count)
        {
            return Err(invalid("CGTransform映射范围或数量无效（仅支持一对一映射）"));
        }
        for (offset, id) in cg.glyphs.iter().enumerate() {
            if glyphs[start + offset].is_some() {
                return Err(invalid("CGTransform重叠"));
            }
            glyphs[start + offset] = Some(*id);
        }
    }
    let read = text.read_direction.unwrap_or(0);
    let char_dir = text.char_direction.unwrap_or(0);
    if ![0, 90, 180, 270].contains(&read) || ![0, 90, 180, 270].contains(&char_dir) {
        return Err(invalid("文字方向无效"));
    }
    let angle = (read as f64).to_radians();
    let (sin, cos) = angle.sin_cos();
    let glyph_angle = (char_dir as f64).to_radians();
    let (gs, gc) = glyph_angle.sin_cos();
    let scale = text.h_scale.unwrap_or(1.);
    if !scale.is_finite() || scale <= 0. {
        return Err(invalid("HScale无效"));
    }
    let linear = [
        (ctm[0] * gc + ctm[2] * gs) * scale,
        (ctm[1] * gc + ctm[3] * gs) * scale,
        -ctm[0] * gs + ctm[2] * gc,
        -ctm[1] * gs + ctm[3] * gc,
    ];
    let mut cursor = [0., 0.];
    let mut out = Vec::with_capacity(count);
    let mut global = 0;
    for code in &text.text_codes {
        if let Some(x) = code.x {
            cursor[0] = x;
        }
        if let Some(y) = code.y {
            cursor[1] = y;
        }
        if cursor
            .iter()
            .chain(code.delta_x.iter())
            .chain(code.delta_y.iter())
            .any(|n| !n.is_finite())
        {
            return Err(invalid("TextCode坐标或步距无效"));
        }
        let chars: Vec<char> = code.content.chars().collect();
        if code.delta_x.len() > chars.len() || code.delta_y.len() > chars.len() {
            return Err(invalid("delta超过字符数"));
        }
        for (i, character) in chars.into_iter().enumerate() {
            let p = ctm_transform_point(cursor[0], cursor[1], &ctm);
            out.push(GlyphPosition {
                character,
                glyph_id: glyphs[global],
                origin_mm: [p[0] + b[0], p[1] + b[1]],
                linear_ctm: linear,
            });
            let dx = code
                .delta_x
                .get(i)
                .or_else(|| code.delta_x.last())
                .copied()
                .unwrap_or(advances_mm[global])
                * scale;
            let dy = code
                .delta_y
                .get(i)
                .or_else(|| code.delta_y.last())
                .copied()
                .unwrap_or(0.);
            cursor[0] += dx * cos - dy * sin;
            cursor[1] += dx * sin + dy * cos;
            global += 1;
        }
    }
    Ok(out)
}
