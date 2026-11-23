//! 保留旧栅格入口布局的逐字形 Fontations / tiny-skia 光栅化。
use super::{
    font_face::{FontFace, GlyphId},
    glyph_outline::GlyphOutline,
};
use easyofd_core::{OfdError, OfdResult};
use tiny_skia::{FillRule, Paint, Pixmap, Transform};

/// 字形像素度量和覆盖率，坐标遵循旧渲染入口的基线约定。
pub(crate) struct GlyphBitmap {
    pub(crate) xmin: f32,
    pub(crate) ymin: f32,
    pub(crate) advance: f32,
    pub(crate) pixels: Option<Pixmap>,
}
impl GlyphBitmap {
    /// 参数为字体、字符及有限正像素字号，返回受预算保护的字形覆盖率。
    pub(crate) fn rasterize(face: &FontFace<'_>, character: char, px_size: f32) -> OfdResult<Self> {
        if !px_size.is_finite() || px_size <= 0. || px_size > 1024. {
            return Err(OfdError::Conversion("文字字号超出安全栅格预算".into()));
        }
        let id = face.glyph_index(character).unwrap_or(GlyphId(0));
        let scale = px_size / f32::from(face.units_per_em());
        let advance = face.glyph_hor_advance(id).unwrap_or(0.) * scale;
        let outline = GlyphOutline::load(face, id)?;
        let Some(path) = outline.path else {
            return Ok(Self {
                xmin: 0.,
                ymin: 0.,
                advance,
                pixels: None,
            });
        };
        let bounds = path
            .compute_tight_bounds()
            .ok_or_else(|| OfdError::Conversion("字形边界无效".into()))?;
        let xmin = (bounds.left() * scale).floor();
        let ymin = (bounds.top() * scale).floor();
        let xmax = (bounds.right() * scale).ceil();
        let ymax = (bounds.bottom() * scale).ceil();
        let width = xmax - xmin;
        let height = ymax - ymin;
        // 使用真实轮廓的像素边界，所有预算检查均先于 Pixmap 的分配。
        if ![xmin, ymin, xmax, ymax, width, height]
            .iter()
            .all(|v| v.is_finite())
            || width < 0.
            || height < 0.
            || width > 4096.
            || height > 4096.
            || width * height > 4_194_304.
        {
            return Err(OfdError::Conversion("实际字形位图超出预算".into()));
        }
        if width == 0. || height == 0. {
            return Ok(Self {
                xmin,
                ymin,
                advance,
                pixels: None,
            });
        }
        let mut pixels = Pixmap::new(width as u32, height as u32)
            .ok_or_else(|| OfdError::Conversion("字形位图分配失败".into()))?;
        let mut paint = Paint {
            anti_alias: true,
            ..Paint::default()
        };
        paint.set_color_rgba8(0, 0, 0, 255);
        pixels.fill_path(
            &path,
            &paint,
            FillRule::Winding,
            Transform::from_row(scale, 0., 0., -scale, -xmin, ymax),
            None,
        );
        Ok(Self {
            xmin,
            ymin,
            advance,
            pixels: Some(pixels),
        })
    }
}
