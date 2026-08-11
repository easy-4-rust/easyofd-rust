//! 文本编码点。
//!
//! 对应 Java: org.ofdrw.converter.point.TextCodePoint

/// 文本编码点，表示经过坐标变换后单个字符在页面上的最终位置。
///
/// 对应 Java `TextCodePoint`。用于 OFD → PDF 转换时记录每个字符的
/// 精确坐标（已考虑 CTM、DeltaX/DeltaY、Boundary 偏移等变换）。
#[derive(Debug, Clone, PartialEq)]
pub struct TextCodePoint {
    /// 字符在页面上的 X 坐标（已变换，单位取决于调用方）。
    pub x: f64,
    /// 字符在页面上的 Y 坐标（已变换，单位取决于调用方）。
    pub y: f64,
    /// 文本内容（单个字符或字形）。
    text: String,
    /// 字形名称（可选，用于 CGTransform 映射）。
    glyph: Option<String>,
}

impl TextCodePoint {
    /// 创建文本编码点。
    ///
    /// # 参数
    /// - `x`：X 坐标
    /// - `y`：Y 坐标
    /// - `text`：文本内容
    pub fn new(x: f64, y: f64, text: impl Into<String>) -> Self {
        Self {
            x,
            y,
            text: text.into(),
            glyph: None,
        }
    }

    /// 返回文本内容。
    pub fn text(&self) -> &str {
        &self.text
    }

    /// 设置文本内容。
    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = text.into();
    }

    /// 返回字形名称。
    pub fn glyph(&self) -> Option<&str> {
        self.glyph.as_deref()
    }

    /// 设置字形名称。
    pub fn set_glyph(&mut self, glyph: impl Into<String>) {
        self.glyph = Some(glyph.into());
    }

    /// 追加字形名称（空格分隔）。
    pub fn append_glyph(&mut self, glyph: &str) {
        match &mut self.glyph {
            Some(existing) => {
                existing.push(' ');
                existing.push_str(glyph);
            }
            None => {
                self.glyph = Some(glyph.to_string());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let tcp = TextCodePoint::new(10.5, 20.3, "A");
        assert!((tcp.x - 10.5).abs() < f64::EPSILON);
        assert!((tcp.y - 20.3).abs() < f64::EPSILON);
        assert_eq!(tcp.text(), "A");
        assert!(tcp.glyph().is_none());
    }

    #[test]
    fn test_set_text() {
        let mut tcp = TextCodePoint::new(0.0, 0.0, "X");
        tcp.set_text("Y");
        assert_eq!(tcp.text(), "Y");
    }

    #[test]
    fn test_glyph() {
        let mut tcp = TextCodePoint::new(0.0, 0.0, "a");
        assert!(tcp.glyph().is_none());
        tcp.set_glyph("glyph_a");
        assert_eq!(tcp.glyph(), Some("glyph_a"));
    }

    #[test]
    fn test_append_glyph() {
        let mut tcp = TextCodePoint::new(0.0, 0.0, "a");
        tcp.append_glyph("g1");
        assert_eq!(tcp.glyph(), Some("g1"));
        tcp.append_glyph("g2");
        assert_eq!(tcp.glyph(), Some("g1 g2"));
    }

    #[test]
    fn test_clone() {
        let tcp = TextCodePoint::new(1.0, 2.0, "x");
        let tcp2 = tcp.clone();
        assert_eq!(tcp, tcp2);
    }
}
