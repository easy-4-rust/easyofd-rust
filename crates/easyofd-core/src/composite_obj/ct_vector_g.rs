//! CT_VectorG 矢量图形复合对象。
//!
//! 对应 GB/T 33190-2016 第 13.6 节中的 CT_VectorG 类型。
//! 矢量图形复合对象用于描述由多条路径、文字等组成的矢量图形，
//! 支持描边、填充和变换操作。

use super::Content;

/// 对应 Java: org.ofdrw.core.compositeObj.CT_VectorG
///
/// 矢量图形复合对象。将多个矢量图元（路径、文本等）组合为
/// 一个可复用的矢量图形单元，支持描边和填充样式设置。
#[allow(non_camel_case_types)]
#[derive(Debug, Clone)]
pub struct CT_VectorG {
    /// 对象 ID，在页面内唯一。
    pub id: u32,
    /// 对象边界框，格式为 "x y width height"（单位 mm）。
    pub boundary: String,
    /// 线宽（mm），默认 0.35。
    pub line_width: f64,
    /// 描边颜色 RGB hex。None 表示不描边。
    pub stroke_color: Option<u32>,
    /// 填充颜色 RGB hex。None 表示不填充。
    pub fill_color: Option<u32>,
    /// 矢量内容列表。
    pub contents: Vec<Content>,
}

impl CT_VectorG {
    /// 创建新的矢量图形复合对象。
    #[must_use]
    pub fn new(id: u32, boundary: impl Into<String>) -> Self {
        Self {
            id,
            boundary: boundary.into(),
            line_width: 0.35,
            stroke_color: None,
            fill_color: None,
            contents: Vec::new(),
        }
    }

    /// 设置线宽。
    #[must_use]
    pub fn line_width(mut self, width: f64) -> Self {
        self.line_width = width;
        self
    }

    /// 设置描边颜色。
    #[must_use]
    pub fn stroke_color(mut self, color: u32) -> Self {
        self.stroke_color = Some(color);
        self
    }

    /// 设置填充颜色。
    #[must_use]
    pub fn fill_color(mut self, color: u32) -> Self {
        self.fill_color = Some(color);
        self
    }

    /// 添加矢量内容。
    pub fn add_content(&mut self, content: Content) {
        self.contents.push(content);
    }

    /// 内容数量。
    #[must_use]
    pub fn content_count(&self) -> usize {
        self.contents.len()
    }

    /// 序列化为 OFD XML 字符串。
    #[must_use]
    pub fn to_xml_string(&self) -> String {
        use std::fmt::Write;
        let mut xml = format!(
            "<ofd:CT_VectorG ID=\"{}\" Boundary=\"{}\"",
            self.id, self.boundary
        );
        write!(xml, " LineWidth=\"{}\"", self.line_width).expect("写入内存缓冲区不会失败");
        if let Some(sc) = self.stroke_color {
            write!(xml, " StrokeColor=\"{sc}\"").expect("写入内存缓冲区不会失败");
        }
        if let Some(fc) = self.fill_color {
            write!(xml, " FillColor=\"{fc}\"").expect("写入内存缓冲区不会失败");
        }
        xml.push_str(">\n");
        for content in &self.contents {
            xml.push_str(&content.to_xml_string());
        }
        xml.push_str("</ofd:CT_VectorG>\n");
        xml
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ct_vector_g_new() {
        let vg = CT_VectorG::new(1, "0 0 100 100");
        assert_eq!(vg.id, 1);
        assert_eq!(vg.boundary, "0 0 100 100");
        assert!((vg.line_width - 0.35).abs() < f64::EPSILON);
        assert!(vg.stroke_color.is_none());
        assert!(vg.fill_color.is_none());
        assert!(vg.contents.is_empty());
    }

    #[test]
    fn test_ct_vector_g_builder() {
        let vg = CT_VectorG::new(2, "10 10 50 50")
            .line_width(1.0)
            .stroke_color(0xFF_0000)
            .fill_color(0x00_FF00);
        assert!((vg.line_width - 1.0).abs() < f64::EPSILON);
        assert_eq!(vg.stroke_color, Some(0xFF_0000));
        assert_eq!(vg.fill_color, Some(0x00_FF00));
    }

    #[test]
    fn test_ct_vector_g_add_content() {
        let mut vg = CT_VectorG::new(3, "0 0 200 200");
        vg.add_content(Content::path("M0 0L10 10"));
        vg.add_content(Content::text("label", 5.0, 5.0));
        assert_eq!(vg.content_count(), 2);
    }

    #[test]
    fn test_ct_vector_g_to_xml_string() {
        let vg = CT_VectorG::new(10, "0 0 80 60")
            .line_width(0.5)
            .stroke_color(0x00_00FF);
        let xml = vg.to_xml_string();
        assert!(xml.contains("ID=\"10\""));
        assert!(xml.contains("Boundary=\"0 0 80 60\""));
        assert!(xml.contains("LineWidth=\"0.5\""));
        assert!(xml.contains("StrokeColor=\"255\""));
        assert!(xml.contains("<ofd:CT_VectorG"));
        assert!(xml.contains("</ofd:CT_VectorG>"));
    }

    #[test]
    fn test_ct_vector_g_to_xml_with_contents() {
        let mut vg = CT_VectorG::new(11, "0 0 50 50");
        vg.add_content(Content::path("M0 0L100 100"));
        let xml = vg.to_xml_string();
        assert!(xml.contains("ofd:PathObject"));
        assert!(xml.contains("M0 0L100 100"));
    }

    #[test]
    fn test_ct_vector_g_to_xml_with_fill() {
        let vg = CT_VectorG::new(12, "0 0 10 10").fill_color(0xCC_CCCC);
        let xml = vg.to_xml_string();
        assert!(xml.contains("FillColor=\"13421772\""));
    }

    #[test]
    fn test_ct_vector_g_clone_debug() {
        let vg = CT_VectorG::new(1, "0 0 1 1");
        let vg2 = vg.clone();
        assert_eq!(vg2.id, 1);
        assert!(format!("{vg:?}").contains("CT_VectorG"));
    }
}
