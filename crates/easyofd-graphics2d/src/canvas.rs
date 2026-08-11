//! 2D 绘图画布。
//!
//! 对应 Java 版 `ofdrw-graphics2d` 中的 `OFDGraphicsDocument`，
//! 提供类似 `java.awt.Graphics2D` 的绘图指令录制能力。

/// 绘制命令，记录画布上的每一次绘图操作。
///
/// 命令序列忠实反映调用顺序，供 [`converter`](crate::converter) 转译为 OFD 对象。
#[derive(Debug, Clone, PartialEq)]
pub enum DrawCommand {
    /// 移动画笔到指定坐标（mm）。
    MoveTo(f64, f64),
    /// 从当前位置画直线到指定坐标（mm）。
    LineTo(f64, f64),
    /// 绘制矩形（左上角 x, y；宽 w, 高 h，单位 mm）。
    Rect {
        /// 左上角 x 坐标（mm）。
        x: f64,
        /// 左上角 y 坐标（mm）。
        y: f64,
        /// 宽度（mm）。
        w: f64,
        /// 高度（mm）。
        h: f64,
    },
    /// 绘制椭圆（中心 cx, cy；半径 rx, ry，单位 mm）。
    Ellipse {
        /// 中心 x 坐标（mm）。
        cx: f64,
        /// 中心 y 坐标（mm）。
        cy: f64,
        /// x 方向半径（mm）。
        rx: f64,
        /// y 方向半径（mm）。
        ry: f64,
    },
    /// 设置描边颜色（RGB 十六进制，如 `0xFF0000` 表示红色）。
    SetStrokeColor(u32),
    /// 设置填充颜色（RGB 十六进制）。
    SetFillColor(u32),
    /// 设置线宽（mm）。
    SetLineWidth(f64),
    /// 以当前填充颜色填充当前路径。
    Fill,
    /// 以当前描边颜色和线宽描边当前路径。
    Stroke,
    /// 绘制文本。
    DrawText {
        /// 文本起始 x 坐标（mm）。
        x: f64,
        /// 文本起始 y 坐标（mm）。
        y: f64,
        /// 文本内容。
        text: String,
        /// 字号（pt）。
        font_size: f64,
    },
}

/// 2D 绘图画布。
///
/// 对应 Java 版 `ofdrw-graphics2d` 的 `OFDGraphicsDocument`。
/// 通过 [`push`](Canvas::push) 录制绘图命令，再由
/// [`canvas_to_page`](crate::converter::canvas_to_page) 转为 OFD 页面。
#[derive(Debug, Clone)]
pub struct Canvas {
    /// 画布宽度（mm）。
    width: f64,
    /// 画布高度（mm）。
    height: f64,
    /// 已录制的绘图命令序列。
    commands: Vec<DrawCommand>,
}

impl Canvas {
    /// 创建指定尺寸的画布。
    ///
    /// # 参数
    /// - `width`：画布宽度，单位 mm。
    /// - `height`：画布高度，单位 mm。
    #[must_use]
    pub fn new(width: f64, height: f64) -> Self {
        Self {
            width,
            height,
            commands: Vec::new(),
        }
    }

    /// 返回画布宽度（mm）。
    #[must_use]
    pub fn width(&self) -> f64 {
        self.width
    }

    /// 返回画布高度（mm）。
    #[must_use]
    pub fn height(&self) -> f64 {
        self.height
    }

    /// 追加一条绘图命令。
    pub fn push(&mut self, cmd: DrawCommand) {
        self.commands.push(cmd);
    }

    /// 返回已录制的命令列表（只读借用）。
    #[must_use]
    pub fn commands(&self) -> &[DrawCommand] {
        &self.commands
    }

    /// 消费画布，返回命令列表。
    #[must_use]
    pub fn into_commands(self) -> Vec<DrawCommand> {
        self.commands
    }

    // ── 便捷方法 ──────────────────────────────────────────────────────

    /// 移动画笔到指定坐标。
    pub fn move_to(&mut self, x: f64, y: f64) {
        self.push(DrawCommand::MoveTo(x, y));
    }

    /// 从当前位置画直线到指定坐标。
    pub fn line_to(&mut self, x: f64, y: f64) {
        self.push(DrawCommand::LineTo(x, y));
    }

    /// 绘制矩形。
    pub fn rect(&mut self, x: f64, y: f64, w: f64, h: f64) {
        self.push(DrawCommand::Rect { x, y, w, h });
    }

    /// 绘制椭圆。
    pub fn ellipse(&mut self, cx: f64, cy: f64, rx: f64, ry: f64) {
        self.push(DrawCommand::Ellipse { cx, cy, rx, ry });
    }

    /// 设置描边颜色。
    pub fn set_stroke_color(&mut self, color: u32) {
        self.push(DrawCommand::SetStrokeColor(color));
    }

    /// 设置填充颜色。
    pub fn set_fill_color(&mut self, color: u32) {
        self.push(DrawCommand::SetFillColor(color));
    }

    /// 设置线宽。
    pub fn set_line_width(&mut self, width: f64) {
        self.push(DrawCommand::SetLineWidth(width));
    }

    /// 以当前填充颜色填充路径。
    pub fn fill(&mut self) {
        self.push(DrawCommand::Fill);
    }

    /// 以当前描边颜色和线宽描边路径。
    pub fn stroke(&mut self) {
        self.push(DrawCommand::Stroke);
    }

    /// 绘制文本。
    pub fn draw_text(&mut self, x: f64, y: f64, text: impl Into<String>, font_size: f64) {
        self.push(DrawCommand::DrawText {
            x,
            y,
            text: text.into(),
            font_size,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canvas_new() {
        let c = Canvas::new(210.0, 297.0);
        assert!((c.width() - 210.0).abs() < f64::EPSILON);
        assert!((c.height() - 297.0).abs() < f64::EPSILON);
        assert!(c.commands().is_empty());
    }

    #[test]
    fn test_canvas_push_and_commands() {
        let mut c = Canvas::new(100.0, 100.0);
        c.push(DrawCommand::MoveTo(0.0, 0.0));
        c.push(DrawCommand::LineTo(10.0, 10.0));
        assert_eq!(c.commands().len(), 2);
        assert_eq!(c.commands()[0], DrawCommand::MoveTo(0.0, 0.0));
        assert_eq!(c.commands()[1], DrawCommand::LineTo(10.0, 10.0));
    }

    #[test]
    fn test_canvas_into_commands() {
        let mut c = Canvas::new(50.0, 50.0);
        c.push(DrawCommand::SetLineWidth(0.5));
        let cmds = c.into_commands();
        assert_eq!(cmds.len(), 1);
        assert_eq!(cmds[0], DrawCommand::SetLineWidth(0.5));
    }

    #[test]
    fn test_canvas_convenience_methods() {
        let mut c = Canvas::new(210.0, 297.0);
        c.move_to(10.0, 20.0);
        c.line_to(30.0, 40.0);
        c.rect(0.0, 0.0, 50.0, 50.0);
        c.ellipse(100.0, 100.0, 20.0, 10.0);
        c.set_stroke_color(0xFF_0000);
        c.set_fill_color(0x00_FF00);
        c.set_line_width(1.0);
        c.fill();
        c.stroke();
        c.draw_text(5.0, 5.0, "hello", 12.0);
        assert_eq!(c.commands().len(), 10);
    }

    #[test]
    fn test_draw_command_clone_eq_debug() {
        let cmd = DrawCommand::Rect {
            x: 1.0,
            y: 2.0,
            w: 3.0,
            h: 4.0,
        };
        let cmd2 = cmd.clone();
        assert_eq!(cmd, cmd2);
        assert!(format!("{cmd:?}").contains("Rect"));
    }

    #[test]
    fn test_canvas_clone_debug() {
        let c = Canvas::new(100.0, 200.0);
        let c2 = c.clone();
        assert!((c2.width() - 100.0).abs() < f64::EPSILON);
        assert!(format!("{c:?}").contains("Canvas"));
    }

    #[test]
    fn test_draw_text_command_fields() {
        let cmd = DrawCommand::DrawText {
            x: 10.0,
            y: 20.0,
            text: "test".into(),
            font_size: 14.0,
        };
        if let DrawCommand::DrawText {
            x,
            y,
            text,
            font_size,
        } = cmd
        {
            assert!((x - 10.0).abs() < f64::EPSILON);
            assert!((y - 20.0).abs() < f64::EPSILON);
            assert_eq!(text, "test");
            assert!((font_size - 14.0).abs() < f64::EPSILON);
        } else {
            panic!("expected DrawText");
        }
    }
}
