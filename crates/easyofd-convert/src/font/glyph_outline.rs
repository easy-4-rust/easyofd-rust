//! 字形轮廓复用，供SVG和PNG后端使用。
use std::fmt::Write;
use tiny_skia::{Path, PathBuilder};
/// 字体单位中的矢量字形轮廓。
pub(crate) struct GlyphOutline {
    /// SVG路径数据。
    pub svg: String,
    /// 栅格后端矢量路径。
    pub path: Option<Path>,
}
struct GlyphOutlineBuilder {
    svg: String,
    path: PathBuilder,
}
impl ttf_parser::OutlineBuilder for GlyphOutlineBuilder {
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
    /// 从已解析字体读取指定字形轮廓，空白字形没有path。
    pub(crate) fn load(face: &ttf_parser::Face<'_>, glyph: ttf_parser::GlyphId) -> Self {
        let mut b = GlyphOutlineBuilder {
            svg: String::new(),
            path: PathBuilder::new(),
        };
        face.outline_glyph(glyph, &mut b);
        Self {
            svg: b.svg,
            path: b.path.finish(),
        }
    }
}
