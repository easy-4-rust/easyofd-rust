//! CT_Text 文本对象。

use super::{CT_CGTransform, TextCode};

/// 对应 Java: org.ofdrw.core.text.text.CT_Text
///
/// 文本对象，扩展自图形单元，包含字体引用、文字方向、字间距等
/// 文本特有属性。对应 GB/T 33190-2016 第 11.2 节图 59 表 45。
#[allow(non_camel_case_types)]
#[derive(Debug, Clone)]
pub struct CT_Text {
    /// 对象 ID。
    pub id: u32,
    /// 边界框 "x y width height"（单位 mm）。
    pub boundary: String,
    /// 字体引用 ID。
    pub font_ref: Option<u32>,
    /// 字号（pt）。
    pub size: Option<f64>,
    /// 是否描边。
    pub stroke: bool,
    /// 是否填充。
    pub fill: bool,
    /// 水平缩放比例。
    pub h_scale: Option<f64>,
    /// 阅读方向（角度，0/90/180/270）。
    pub read_direction: Option<u32>,
    /// 字符排列方向（角度）。
    pub char_direction: Option<u32>,
    /// 字重（400=正常，700=粗体）。
    pub weight: Option<u32>,
    /// 是否斜体。
    pub italic: bool,
    /// 填充颜色 RGB hex。
    pub fill_color: Option<u32>,
    /// 描边颜色 RGB hex。
    pub stroke_color: Option<u32>,
    /// 字形变换列表。
    pub cg_transforms: Vec<CT_CGTransform>,
    /// 文字定位列表。
    pub text_codes: Vec<TextCode>,
}

/// 文字阅读方向。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// 从左到右（0度）。
    LeftToRight,
    /// 从上到下（90度）。
    TopToBottom,
    /// 从右到左（180度）。
    RightToLeft,
    /// 从下到上（270度）。
    BottomToTop,
}

impl Direction {
    /// 获取角度值。
    #[must_use]
    pub fn as_degrees(&self) -> u32 {
        match self {
            Self::LeftToRight => 0,
            Self::TopToBottom => 90,
            Self::RightToLeft => 180,
            Self::BottomToTop => 270,
        }
    }
}

impl CT_Text {
    /// 创建新的文本对象。
    #[must_use]
    pub fn new(id: u32, boundary: impl Into<String>) -> Self {
        Self {
            id,
            boundary: boundary.into(),
            font_ref: None,
            size: None,
            stroke: false,
            fill: true,
            h_scale: None,
            read_direction: None,
            char_direction: None,
            weight: None,
            italic: false,
            fill_color: None,
            stroke_color: None,
            cg_transforms: Vec::new(),
            text_codes: Vec::new(),
        }
    }

    /// 设置字体引用。
    #[must_use]
    pub fn font(mut self, ref_id: u32) -> Self {
        self.font_ref = Some(ref_id);
        self
    }

    /// 设置字号。
    #[must_use]
    pub fn size(mut self, size: f64) -> Self {
        self.size = Some(size);
        self
    }

    /// 设置描边。
    #[must_use]
    pub fn stroke(mut self, stroke: bool) -> Self {
        self.stroke = stroke;
        self
    }

    /// 设置填充。
    #[must_use]
    pub fn fill(mut self, fill: bool) -> Self {
        self.fill = fill;
        self
    }

    /// 设置水平缩放。
    #[must_use]
    pub fn h_scale(mut self, scale: f64) -> Self {
        self.h_scale = Some(scale);
        self
    }

    /// 设置阅读方向。
    #[must_use]
    pub fn read_direction(mut self, dir: Direction) -> Self {
        self.read_direction = Some(dir.as_degrees());
        self
    }

    /// 设置字符方向。
    #[must_use]
    pub fn char_direction(mut self, dir: Direction) -> Self {
        self.char_direction = Some(dir.as_degrees());
        self
    }

    /// 设置字重。
    #[must_use]
    pub fn weight(mut self, weight: u32) -> Self {
        self.weight = Some(weight);
        self
    }

    /// 设置斜体。
    #[must_use]
    pub fn italic(mut self, italic: bool) -> Self {
        self.italic = italic;
        self
    }

    /// 设置填充颜色。
    #[must_use]
    pub fn fill_color(mut self, color: u32) -> Self {
        self.fill_color = Some(color);
        self
    }

    /// 设置描边颜色。
    #[must_use]
    pub fn stroke_color(mut self, color: u32) -> Self {
        self.stroke_color = Some(color);
        self
    }

    /// 添加字形变换。
    pub fn add_cg_transform(&mut self, cg: CT_CGTransform) {
        self.cg_transforms.push(cg);
    }

    /// 添加文字定位。
    pub fn add_text_code(&mut self, tc: TextCode) {
        self.text_codes.push(tc);
    }

    /// 获取字体引用。
    #[must_use]
    pub fn get_font(&self) -> Option<u32> {
        self.font_ref
    }

    /// 获取字号。
    #[must_use]
    pub fn get_size(&self) -> Option<f64> {
        self.size
    }

    /// 获取字重。
    #[must_use]
    pub fn get_weight(&self) -> Option<u32> {
        self.weight
    }

    /// 获取字形变换列表。
    #[must_use]
    pub fn get_cg_transforms(&self) -> &[CT_CGTransform] {
        &self.cg_transforms
    }

    /// 获取文字定位列表。
    #[must_use]
    pub fn get_text_codes(&self) -> &[TextCode] {
        &self.text_codes
    }

    /// 序列化为 OFD XML 字符串。
    #[must_use]
    pub fn to_xml_string(&self) -> String {
        use std::fmt::Write;
        let mut xml = format!(
            "<ofd:TextObject ID=\"{}\" Boundary=\"{}\"",
            self.id, self.boundary
        );
        if let Some(fr) = self.font_ref {
            write!(xml, " Font=\"{fr}\"").unwrap();
        }
        if let Some(sz) = self.size {
            write!(xml, " Size=\"{sz}\"").unwrap();
        }
        if self.stroke {
            xml.push_str(" Stroke=\"true\"");
        }
        if !self.fill {
            xml.push_str(" Fill=\"false\"");
        }
        if let Some(hs) = self.h_scale {
            write!(xml, " HScale=\"{hs}\"").unwrap();
        }
        if let Some(rd) = self.read_direction {
            write!(xml, " ReadDirection=\"{rd}\"").unwrap();
        }
        if let Some(cd) = self.char_direction {
            write!(xml, " CharDirection=\"{cd}\"").unwrap();
        }
        if let Some(w) = self.weight {
            write!(xml, " Weight=\"{w}\"").unwrap();
        }
        if self.italic {
            xml.push_str(" Italic=\"true\"");
        }
        if let Some(fc) = self.fill_color {
            write!(xml, " FillColor=\"{fc}\"").unwrap();
        }
        if let Some(sc) = self.stroke_color {
            write!(xml, " StrokeColor=\"{sc}\"").unwrap();
        }
        xml.push_str(">\n");
        for cg in &self.cg_transforms {
            xml.push_str("  ");
            xml.push_str(&cg.to_xml_string());
            xml.push('\n');
        }
        for tc in &self.text_codes {
            xml.push_str("  ");
            xml.push_str(&tc.to_xml_string());
            xml.push('\n');
        }
        xml.push_str("</ofd:TextObject>\n");
        xml
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ct_text_new() {
        let t = CT_Text::new(1, "0 0 100 20");
        assert_eq!(t.id, 1);
        assert_eq!(t.boundary, "0 0 100 20");
        assert!(t.font_ref.is_none());
        assert!(t.fill);
        assert!(!t.stroke);
        assert!(!t.italic);
    }

    #[test]
    fn test_ct_text_builder() {
        let t = CT_Text::new(2, "10 20 50 15")
            .font(3)
            .size(14.0)
            .weight(700)
            .italic(true)
            .fill_color(0xFF_0000)
            .h_scale(1.2);
        assert_eq!(t.get_font(), Some(3));
        assert!((t.get_size().unwrap() - 14.0).abs() < f64::EPSILON);
        assert_eq!(t.get_weight(), Some(700));
        assert!(t.italic);
        assert_eq!(t.fill_color, Some(0xFF_0000));
        assert!((t.h_scale.unwrap() - 1.2).abs() < f64::EPSILON);
    }

    #[test]
    fn test_ct_text_direction() {
        let t = CT_Text::new(1, "0 0 10 10")
            .read_direction(Direction::TopToBottom)
            .char_direction(Direction::RightToLeft);
        assert_eq!(t.read_direction, Some(90));
        assert_eq!(t.char_direction, Some(180));
    }

    #[test]
    fn test_ct_text_add_text_code() {
        let mut t = CT_Text::new(1, "0 0 100 20");
        t.add_text_code(TextCode::with_content("Hello").coordinate(0.0, 10.0));
        t.add_text_code(TextCode::with_content("World").coordinate(30.0, 10.0));
        assert_eq!(t.get_text_codes().len(), 2);
    }

    #[test]
    fn test_ct_text_add_cg_transform() {
        let mut t = CT_Text::new(1, "0 0 100 20");
        t.add_cg_transform(
            CT_CGTransform::new()
                .code_position(0)
                .code_count(1)
                .glyph_count(1)
                .glyphs(vec![42]),
        );
        assert_eq!(t.get_cg_transforms().len(), 1);
    }

    #[test]
    fn test_direction_as_degrees() {
        assert_eq!(Direction::LeftToRight.as_degrees(), 0);
        assert_eq!(Direction::TopToBottom.as_degrees(), 90);
        assert_eq!(Direction::RightToLeft.as_degrees(), 180);
        assert_eq!(Direction::BottomToTop.as_degrees(), 270);
    }

    #[test]
    fn test_ct_text_to_xml_basic() {
        let t = CT_Text::new(1, "0 0 100 20");
        let xml = t.to_xml_string();
        assert!(xml.contains("ID=\"1\""));
        assert!(xml.contains("Boundary=\"0 0 100 20\""));
        assert!(xml.contains("<ofd:TextObject"));
        assert!(xml.contains("</ofd:TextObject>"));
    }

    #[test]
    fn test_ct_text_to_xml_full() {
        let mut t = CT_Text::new(5, "10 20 200 30")
            .font(3)
            .size(12.0)
            .weight(700)
            .italic(true)
            .stroke(true)
            .fill(false)
            .fill_color(0xFF_0000)
            .stroke_color(0x00_FF00)
            .read_direction(Direction::TopToBottom);
        t.add_text_code(TextCode::with_content("test").coordinate(10.0, 30.0));
        let xml = t.to_xml_string();
        assert!(xml.contains("Font=\"3\""));
        assert!(xml.contains("Size=\"12\""));
        assert!(xml.contains("Weight=\"700\""));
        assert!(xml.contains("Italic=\"true\""));
        assert!(xml.contains("Stroke=\"true\""));
        assert!(xml.contains("Fill=\"false\""));
        assert!(xml.contains("FillColor=\"16711680\""));
        assert!(xml.contains("StrokeColor=\"65280\""));
        assert!(xml.contains("ReadDirection=\"90\""));
        assert!(xml.contains("ofd:TextCode"));
        assert!(xml.contains("test"));
    }

    #[test]
    fn test_ct_text_clone_debug() {
        let t = CT_Text::new(1, "0 0 1 1");
        let t2 = t.clone();
        assert_eq!(t2.id, 1);
        assert!(format!("{t:?}").contains("CT_Text"));
    }
}
