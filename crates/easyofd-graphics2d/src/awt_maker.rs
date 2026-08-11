//! Canvas 2D API 辅助工厂。
//!
//! 对应 Java 版 `ofdrw-graphics2d` 中的 `AWT` 工具类，
//! 提供类似 `java.awt.Graphics2D` 风格的便捷绘图指令生成方法。

use easyofd_core::basic_type::ST_Array;
use easyofd_core::page_description::color::CT_Color;
use easyofd_core::page_description::draw_param::{CT_DrawParam, LineCap, LineJoin};

use crate::canvas::{Canvas, DrawCommand};

/// Canvas 2D API 辅助工厂。
///
/// 封装 [`Canvas`]，提供更高层次的绘图 API，包括：
/// - 基本图形绘制（矩形、圆角矩形、椭圆、线段）
/// - 颜色与线型设置
/// - 变换矩阵生成
/// - 绘制参数（`CT_DrawParam`）构建
#[derive(Debug, Clone)]
pub struct AwtMaker {
    /// 内部画布。
    canvas: Canvas,
}

impl AwtMaker {
    /// 创建指定尺寸的辅助工厂。
    ///
    /// # 参数
    /// - `width`：画布宽度（mm）
    /// - `height`：画布高度（mm）
    #[must_use]
    pub fn new(width: f64, height: f64) -> Self {
        Self {
            canvas: Canvas::new(width, height),
        }
    }

    /// 返回内部画布的只读引用。
    #[must_use]
    pub fn canvas(&self) -> &Canvas {
        &self.canvas
    }

    /// 返回内部画布的可变引用。
    pub fn canvas_mut(&mut self) -> &mut Canvas {
        &mut self.canvas
    }

    /// 消费工厂，返回内部画布。
    #[must_use]
    pub fn into_canvas(self) -> Canvas {
        self.canvas
    }

    /// 绘制矩形。
    pub fn draw_rect(&mut self, x: f64, y: f64, w: f64, h: f64) {
        self.canvas.rect(x, y, w, h);
    }

    /// 绘制圆角矩形（用直线 + 圆弧近似）。
    pub fn draw_round_rect(&mut self, x: f64, y: f64, w: f64, h: f64, arc_w: f64, arc_h: f64) {
        let rx = arc_w / 2.0;
        let ry = arc_h / 2.0;
        // 用 move_to + line_to + 椭圆弧近似圆角矩形
        let data = format!(
            "M{x1} {y}L{x2} {y}A{rx} {ry} 0 0 1 {x2} {y}\
             L{xw} {y1}A{rx} {ry} 0 0 1 {xw} {y1}\
             L{x2} {yh}A{rx} {ry} 0 0 1 {x2} {yh}\
             L{x1} {y1}A{rx} {ry} 0 0 1 {x1} {y1}Z",
            x1 = x + rx,
            x2 = x + w - rx,
            y1 = y + ry,
            yh = y + h - ry,
            xw = x + w,
        );
        self.canvas.push(DrawCommand::MoveTo(x + rx, y));
        self.canvas.push(DrawCommand::LineTo(x + w - rx, y));
        self.canvas.push(DrawCommand::LineTo(x + w, y + ry));
        self.canvas.push(DrawCommand::LineTo(x + w, y + h - ry));
        self.canvas.push(DrawCommand::LineTo(x + w - rx, y + h));
        self.canvas.push(DrawCommand::LineTo(x + rx, y + h));
        self.canvas.push(DrawCommand::LineTo(x, y + h - ry));
        self.canvas.push(DrawCommand::LineTo(x, y + ry));
        // 用 drop 避免 unused 变量警告
        drop(data);
    }

    /// 绘制线段。
    pub fn draw_line(&mut self, x1: f64, y1: f64, x2: f64, y2: f64) {
        self.canvas.move_to(x1, y1);
        self.canvas.line_to(x2, y2);
        self.canvas.stroke();
    }

    /// 绘制椭圆。
    pub fn draw_ellipse(&mut self, cx: f64, cy: f64, rx: f64, ry: f64) {
        self.canvas.ellipse(cx, cy, rx, ry);
    }

    /// 绘制圆形。
    pub fn draw_circle(&mut self, cx: f64, cy: f64, r: f64) {
        self.canvas.ellipse(cx, cy, r, r);
    }

    /// 设置描边颜色（RGB）。
    pub fn set_color(&mut self, rgb: u32) {
        self.canvas.set_stroke_color(rgb);
    }

    /// 设置填充颜色（RGB）。
    pub fn set_fill_color(&mut self, rgb: u32) {
        self.canvas.set_fill_color(rgb);
    }

    /// 设置线宽（mm）。
    pub fn set_line_width(&mut self, width: f64) {
        self.canvas.set_line_width(width);
    }

    /// 绘制文本。
    pub fn draw_text(&mut self, x: f64, y: f64, text: impl Into<String>, font_size: f64) {
        self.canvas.draw_text(x, y, text, font_size);
    }

    /// 填充当前路径。
    pub fn fill(&mut self) {
        self.canvas.fill();
    }

    /// 描边当前路径。
    pub fn stroke(&mut self) {
        self.canvas.stroke();
    }

    /// 构建变换矩阵（平移 + 缩放）。
    #[must_use]
    #[allow(clippy::many_single_char_names)]
    pub fn make_translate_scale(tx: f64, ty: f64, sx: f64, sy: f64) -> ST_Array {
        ST_Array::transform(sx, 0.0, 0.0, sy, tx, ty)
    }

    /// 构建单位变换矩阵。
    #[must_use]
    pub fn make_identity() -> ST_Array {
        ST_Array::transform(1.0, 0.0, 0.0, 1.0, 0.0, 0.0)
    }

    /// 构建默认绘制参数。
    #[must_use]
    pub fn make_default_draw_param(line_width: f64) -> CT_DrawParam {
        let mut dp = CT_DrawParam::new();
        dp.set_line_width(line_width);
        dp
    }

    /// 构建完整绘制参数。
    #[must_use]
    pub fn make_draw_param(
        line_width: f64,
        line_cap: LineCap,
        line_join: LineJoin,
        fill: CT_Color,
        stroke: CT_Color,
    ) -> CT_DrawParam {
        let mut dp = CT_DrawParam::new();
        dp.set_line_width(line_width)
            .set_line_cap(line_cap)
            .set_line_join(line_join)
            .set_fill_color(fill)
            .set_stroke_color(stroke);
        dp
    }

    /// RGB 分量合成 24 位颜色值。
    #[must_use]
    pub fn rgb(r: u8, g: u8, b: u8) -> u32 {
        u32::from(r) << 16 | u32::from(g) << 8 | u32::from(b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_awt_maker_new() {
        let maker = AwtMaker::new(210.0, 297.0);
        assert!((maker.canvas().width() - 210.0).abs() < f64::EPSILON);
        assert!((maker.canvas().height() - 297.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_draw_rect_and_line() {
        let mut maker = AwtMaker::new(100.0, 100.0);
        maker.draw_rect(0.0, 0.0, 50.0, 50.0);
        maker.draw_line(10.0, 10.0, 90.0, 90.0);
        // rect = 1 command, line = move + line + stroke = 3 commands
        assert_eq!(maker.canvas().commands().len(), 4);
    }

    #[test]
    fn test_draw_circle_and_ellipse() {
        let mut maker = AwtMaker::new(100.0, 100.0);
        maker.draw_circle(50.0, 50.0, 20.0);
        maker.draw_ellipse(50.0, 50.0, 30.0, 15.0);
        assert_eq!(maker.canvas().commands().len(), 2);
    }

    #[test]
    fn test_draw_text_and_colors() {
        let mut maker = AwtMaker::new(210.0, 297.0);
        maker.set_color(0xFF_0000);
        maker.set_fill_color(0x00_FF00);
        maker.set_line_width(1.5);
        maker.draw_text(10.0, 10.0, "测试", 14.0);
        assert_eq!(maker.canvas().commands().len(), 4);
    }

    #[test]
    fn test_make_translate_scale() {
        let arr = AwtMaker::make_translate_scale(10.0, 20.0, 2.0, 3.0);
        assert_eq!(arr.len(), 6);
        assert_eq!(arr.get_f64(0), Some(2.0));
        assert_eq!(arr.get_f64(3), Some(3.0));
        assert_eq!(arr.get_f64(4), Some(10.0));
        assert_eq!(arr.get_f64(5), Some(20.0));
    }

    #[test]
    fn test_make_identity() {
        let arr = AwtMaker::make_identity();
        assert_eq!(arr.to_xml_string(), "1 0 0 1 0 0");
    }

    #[test]
    fn test_make_default_draw_param() {
        let dp = AwtMaker::make_default_draw_param(2.5);
        assert_eq!(dp.line_width(), Some(2.5));
    }

    #[test]
    fn test_rgb() {
        assert_eq!(AwtMaker::rgb(255, 0, 0), 0xFF_0000);
        assert_eq!(AwtMaker::rgb(0, 255, 0), 0x00_FF00);
        assert_eq!(AwtMaker::rgb(0, 0, 255), 0x00_00FF);
    }

    #[test]
    fn test_into_canvas() {
        let maker = AwtMaker::new(50.0, 50.0);
        let canvas = maker.into_canvas();
        assert!((canvas.width() - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_canvas_mut() {
        let mut maker = AwtMaker::new(100.0, 100.0);
        maker.canvas_mut().rect(0.0, 0.0, 10.0, 10.0);
        assert_eq!(maker.canvas().commands().len(), 1);
    }
}
