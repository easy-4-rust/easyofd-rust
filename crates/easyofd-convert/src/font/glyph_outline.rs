//! 字形轮廓复用，供SVG和PNG后端使用。
use skrifa::MetadataProvider;
use std::fmt::Write;
use tiny_skia::{Path, PathBuilder};
/// 字体单位中的矢量字形轮廓。
pub(crate) struct GlyphOutline {
    /// SVG路径数据。
    pub svg: String,
    /// 栅格后端矢量路径。
    pub path: Option<Path>,
}
/// 将Fontations真实轮廓同步写入SVG与tiny-skia路径。
struct GlyphOutlineBuilder {
    svg: String,
    path: PathBuilder,
}
impl skrifa::outline::OutlinePen for GlyphOutlineBuilder {
    fn move_to(&mut self, x: f32, y: f32) {
        let _ = write!(self.svg, "M{x} {y} ");
        self.path.move_to(x, y);
    }
    fn line_to(&mut self, x: f32, y: f32) {
        let _ = write!(self.svg, "L{x} {y} ");
        self.path.line_to(x, y);
    }
    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let _ = write!(self.svg, "Q{x1} {y1} {x} {y} ");
        self.path.quad_to(x1, y1, x, y);
    }
    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        let _ = write!(self.svg, "C{x1} {y1} {x2} {y2} {x} {y} ");
        self.path.cubic_to(x1, y1, x2, y2, x, y);
    }
    fn close(&mut self) {
        self.svg.push_str("Z ");
        self.path.close();
    }
}
impl GlyphOutline {
    /// 参数为已解析字体与字形编号，返回真实轮廓或绘制错误；空白字形没有path。
    pub(crate) fn load(
        face: &super::font_face::FontFace<'_>,
        glyph: super::font_face::GlyphId,
    ) -> easyofd_core::OfdResult<Self> {
        let mut b = GlyphOutlineBuilder {
            svg: String::new(),
            path: PathBuilder::new(),
        };
        let outline = face
            .font
            .outline_glyphs()
            .get(skrifa::GlyphId::new(u32::from(glyph.0)))
            .ok_or_else(|| easyofd_core::OfdError::Conversion("字体缺少轮廓表或字形越界".into()))?;
        outline
            .draw(
                skrifa::outline::DrawSettings::unhinted(
                    skrifa::instance::Size::unscaled(),
                    skrifa::instance::LocationRef::default(),
                ),
                &mut b,
            )
            .map_err(|e| easyofd_core::OfdError::Conversion(format!("字形轮廓无效: {e}")))?;
        Ok(Self {
            svg: b.svg,
            path: b.path.finish(),
        })
    }
}
