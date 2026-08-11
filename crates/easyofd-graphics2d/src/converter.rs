//! Canvas 绘图命令到 OFD 页面的转换器。
//!
//! 将 [`Canvas`](crate::canvas::Canvas) 中录制的 [`DrawCommand`](crate::canvas::DrawCommand)
//! 序列转译为 [`OfdPage`] 上的 [`ContentObject`]。

use std::fmt::Write as _;

use easyofd_core::model::{OfdPage, PathObject, TextObject};

use crate::canvas::{Canvas, DrawCommand};

/// 贝塞尔曲线近似椭圆的 kappa 常数。
const KAPPA: f64 = 0.552_284_749_8;

/// 将 Canvas 的绘制命令转换为 `OfdPage`。
///
/// 转换规则：
/// - `MoveTo` / `LineTo` 累积路径数据；`Fill` / `Stroke` 时输出 `PathObject`。
/// - `Rect` / `Ellipse` 立即输出为独立的 `PathObject`。
/// - `SetStrokeColor` / `SetFillColor` / `SetLineWidth` 更新当前绘图状态。
/// - `DrawText` 输出为 `TextObject`。
#[must_use]
pub fn canvas_to_page(canvas: &Canvas) -> OfdPage {
    let mut page = OfdPage::new(canvas.width(), canvas.height());
    let mut state = DrawState::default();
    let mut path_data = String::new();
    let mut path_origin: Option<(f64, f64)> = None;

    for cmd in canvas.commands() {
        match *cmd {
            DrawCommand::MoveTo(x, y) => {
                if path_data.is_empty() {
                    path_origin = Some((x, y));
                }
                let _ = write!(path_data, "M{x} {y}");
            }
            DrawCommand::LineTo(x, y) => {
                let _ = write!(path_data, "L{x} {y}");
            }
            DrawCommand::Rect { x, y, w, h } => {
                flush_path(&mut page, &state, &mut path_data, &mut path_origin);
                let d = format!("M{x} {y}L{} {y}L{} {}L{x} {}Z", x + w, x + w, y + h, y + h);
                page.add_path(build_path(&state, x, y, &d));
            }
            DrawCommand::Ellipse { cx, cy, rx, ry } => {
                flush_path(&mut page, &state, &mut path_data, &mut path_origin);
                let d = ellipse_path_data(cx, cy, rx, ry);
                page.add_path(build_path(&state, cx - rx, cy - ry, &d));
            }
            DrawCommand::SetStrokeColor(c) => state.stroke_color = c,
            DrawCommand::SetFillColor(c) => state.fill_color = Some(c),
            DrawCommand::SetLineWidth(w) => state.line_width = w,
            DrawCommand::Fill => {
                state.fill_pending = true;
                flush_path(&mut page, &state, &mut path_data, &mut path_origin);
                state.fill_pending = false;
            }
            DrawCommand::Stroke => {
                flush_path(&mut page, &state, &mut path_data, &mut path_origin);
            }
            DrawCommand::DrawText {
                x,
                y,
                ref text,
                font_size,
            } => {
                flush_path(&mut page, &state, &mut path_data, &mut path_origin);
                let mut tobj = TextObject::new(x, y, text.clone());
                tobj.size = font_size;
                tobj.color = state.stroke_color;
                page.add_text(tobj);
            }
        }
    }

    // 收尾：如有未 flush 的路径则输出。
    flush_path(&mut page, &state, &mut path_data, &mut path_origin);
    page
}

/// 当前绘图状态。
#[derive(Debug, Clone)]
struct DrawState {
    stroke_color: u32,
    fill_color: Option<u32>,
    line_width: f64,
    fill_pending: bool,
}

impl Default for DrawState {
    fn default() -> Self {
        Self {
            stroke_color: 0x000_000,
            fill_color: None,
            line_width: 0.35,
            fill_pending: false,
        }
    }
}

/// 将累积的路径数据 flush 为 `PathObject` 并添加到页面。
fn flush_path(
    page: &mut OfdPage,
    state: &DrawState,
    path_data: &mut String,
    path_origin: &mut Option<(f64, f64)>,
) {
    if path_data.is_empty() {
        return;
    }
    let (ox, oy) = path_origin.unwrap_or((0.0, 0.0));
    let mut p = PathObject::new(ox, oy, path_data.clone());
    p.stroke_color = state.stroke_color;
    p.stroke_width = state.line_width;
    if state.fill_pending {
        p.fill_color = state.fill_color;
    }
    page.add_path(p);
    path_data.clear();
    *path_origin = None;
}

/// 根据当前状态和路径数据构建 `PathObject`（用于 Rect / Ellipse 立即输出）。
fn build_path(state: &DrawState, x: f64, y: f64, data: &str) -> PathObject {
    let mut p = PathObject::new(x, y, data);
    p.stroke_color = state.stroke_color;
    p.stroke_width = state.line_width;
    p
}

/// 用四段贝塞尔曲线生成椭圆的 SVG 路径数据。
fn ellipse_path_data(cx: f64, cy: f64, rx: f64, ry: f64) -> String {
    let kx = KAPPA * rx;
    let ky = KAPPA * ry;
    format!(
        "M{cx} {top}\
         C{r_cx} {top} {right} {r_cy} {right} {cy}\
         C{right} {b_cy} {r_cx} {bottom} {cx} {bottom}\
         C{l_cx} {bottom} {left} {b_cy} {left} {cy}\
         C{left} {r_cy} {l_cx} {top} {cx} {top}\
         Z",
        top = cy - ry,
        bottom = cy + ry,
        left = cx - rx,
        right = cx + rx,
        r_cx = cx + kx,
        l_cx = cx - kx,
        r_cy = cy + ky,
        b_cy = cy - ky,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canvas::Canvas;
    use easyofd_core::model::ContentObject;

    #[test]
    fn test_empty_canvas_produces_empty_page() {
        let c = Canvas::new(210.0, 297.0);
        let page = canvas_to_page(&c);
        assert!((page.width - 210.0).abs() < f64::EPSILON);
        assert!((page.height - 297.0).abs() < f64::EPSILON);
        assert!(page.content.is_empty());
    }

    #[test]
    fn test_move_line_stroke_produces_path() {
        let mut c = Canvas::new(100.0, 100.0);
        c.move_to(10.0, 10.0);
        c.line_to(50.0, 50.0);
        c.stroke();
        let page = canvas_to_page(&c);
        assert_eq!(page.content.len(), 1);
        match &page.content[0] {
            ContentObject::Path(p) => {
                assert!(p.path_data.contains("M10"));
                assert!(p.path_data.contains("L50"));
            }
            other => panic!("expected Path, got {other:?}"),
        }
    }

    #[test]
    fn test_rect_produces_path() {
        let mut c = Canvas::new(100.0, 100.0);
        c.rect(5.0, 5.0, 20.0, 30.0);
        let page = canvas_to_page(&c);
        assert_eq!(page.content.len(), 1);
        match &page.content[0] {
            ContentObject::Path(p) => {
                assert!(p.path_data.starts_with('M'));
                assert!(p.path_data.ends_with('Z'));
            }
            other => panic!("expected Path, got {other:?}"),
        }
    }

    #[test]
    fn test_ellipse_produces_path() {
        let mut c = Canvas::new(100.0, 100.0);
        c.ellipse(50.0, 50.0, 20.0, 10.0);
        let page = canvas_to_page(&c);
        assert_eq!(page.content.len(), 1);
        match &page.content[0] {
            ContentObject::Path(p) => {
                assert!(p.path_data.contains('C'));
                assert!(p.path_data.ends_with('Z'));
            }
            other => panic!("expected Path, got {other:?}"),
        }
    }

    #[test]
    fn test_draw_text_produces_text_object() {
        let mut c = Canvas::new(210.0, 297.0);
        c.draw_text(10.0, 20.0, "你好", 14.0);
        let page = canvas_to_page(&c);
        assert_eq!(page.content.len(), 1);
        match &page.content[0] {
            ContentObject::Text(t) => {
                assert_eq!(t.text, "你好");
                assert!((t.size - 14.0).abs() < f64::EPSILON);
            }
            other => panic!("expected Text, got {other:?}"),
        }
    }

    #[test]
    fn test_set_color_and_line_width_applied() {
        let mut c = Canvas::new(100.0, 100.0);
        c.set_stroke_color(0xFF_0000);
        c.set_line_width(1.5);
        c.rect(0.0, 0.0, 10.0, 10.0);
        let page = canvas_to_page(&c);
        match &page.content[0] {
            ContentObject::Path(p) => {
                assert_eq!(p.stroke_color, 0xFF_0000);
                assert!((p.stroke_width - 1.5).abs() < f64::EPSILON);
            }
            other => panic!("expected Path, got {other:?}"),
        }
    }

    #[test]
    fn test_fill_sets_fill_color() {
        let mut c = Canvas::new(100.0, 100.0);
        c.set_fill_color(0x00_FF00);
        c.move_to(0.0, 0.0);
        c.line_to(10.0, 0.0);
        c.line_to(10.0, 10.0);
        c.fill();
        let page = canvas_to_page(&c);
        match &page.content[0] {
            ContentObject::Path(p) => {
                assert_eq!(p.fill_color, Some(0x00_FF00));
            }
            other => panic!("expected Path, got {other:?}"),
        }
    }

    #[test]
    fn test_mixed_commands_produce_multiple_objects() {
        let mut c = Canvas::new(210.0, 297.0);
        c.rect(0.0, 0.0, 50.0, 50.0);
        c.draw_text(10.0, 10.0, "标题", 24.0);
        c.ellipse(100.0, 100.0, 30.0, 20.0);
        let page = canvas_to_page(&c);
        assert_eq!(page.content.len(), 3);
        assert!(matches!(&page.content[0], ContentObject::Path(_)));
        assert!(matches!(&page.content[1], ContentObject::Text(_)));
        assert!(matches!(&page.content[2], ContentObject::Path(_)));
    }

    #[test]
    fn test_unflushed_path_flushed_at_end() {
        let mut c = Canvas::new(100.0, 100.0);
        c.move_to(0.0, 0.0);
        c.line_to(10.0, 10.0);
        // 没有显式 stroke/fill，converter 应在末尾自动 flush
        let page = canvas_to_page(&c);
        assert_eq!(page.content.len(), 1);
        assert!(matches!(&page.content[0], ContentObject::Path(_)));
    }

    #[test]
    fn test_ellipse_path_data_format() {
        let d = ellipse_path_data(50.0, 50.0, 10.0, 5.0);
        assert!(d.starts_with('M'));
        assert!(d.ends_with('Z'));
        // 椭圆路径应包含四段 C 命令
        assert_eq!(d.matches('C').count(), 4);
    }
}
