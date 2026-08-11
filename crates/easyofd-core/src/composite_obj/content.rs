//! Content 矢量内容描述。
//!
//! 对应 GB/T 33190-2016 第 13.6 节中的 Content 类型。
//! 矢量内容描述用于在矢量图形复合对象中定义具体的绘制图元，
//! 包括路径、文字和图像等。

/// 对应 Java: org.ofdrw.core.compositeObj.Content
///
/// 矢量内容描述。表示矢量图形中的单个绘制图元，
/// 可以是路径、文本或图像。
#[derive(Debug, Clone)]
pub enum Content {
    /// 路径绘制图元。
    Path(PathContent),
    /// 文本绘制图元。
    Text(TextContent),
    /// 图像绘制图元。
    Image(ImageContent),
}

/// 路径图元内容。
#[derive(Debug, Clone)]
pub struct PathContent {
    /// 路径数据（SVG 风格的缩略数据）。
    pub data: String,
    /// 描边颜色 RGB hex。
    pub stroke_color: u32,
    /// 线宽（mm）。
    pub line_width: f64,
    /// 填充颜色 RGB hex（可选）。
    pub fill_color: Option<u32>,
}

/// 文本图元内容。
#[derive(Debug, Clone)]
pub struct TextContent {
    /// 文本内容。
    pub text: String,
    /// X 坐标（mm）。
    pub x: f64,
    /// Y 坐标（mm）。
    pub y: f64,
    /// 字号（pt）。
    pub font_size: f64,
    /// 字体名称。
    pub font: String,
    /// 文本颜色 RGB hex。
    pub color: u32,
}

/// 图像图元内容。
#[derive(Debug, Clone)]
pub struct ImageContent {
    /// 图像数据（原始字节）。
    pub data: Vec<u8>,
    /// X 坐标（mm）。
    pub x: f64,
    /// Y 坐标（mm）。
    pub y: f64,
    /// 宽度（mm）。
    pub width: f64,
    /// 高度（mm）。
    pub height: f64,
}

impl Content {
    /// 创建路径内容。
    #[must_use]
    pub fn path(data: impl Into<String>) -> Self {
        Self::Path(PathContent {
            data: data.into(),
            stroke_color: 0x00_0000,
            line_width: 0.35,
            fill_color: None,
        })
    }

    /// 创建文本内容。
    #[must_use]
    pub fn text(text: impl Into<String>, x: f64, y: f64) -> Self {
        Self::Text(TextContent {
            text: text.into(),
            x,
            y,
            font_size: 12.0,
            font: "SimSun".into(),
            color: 0x00_0000,
        })
    }

    /// 创建图像内容。
    #[must_use]
    pub fn image(data: Vec<u8>, x: f64, y: f64, width: f64, height: f64) -> Self {
        Self::Image(ImageContent {
            data,
            x,
            y,
            width,
            height,
        })
    }

    /// 是否为路径内容。
    #[must_use]
    pub fn is_path(&self) -> bool {
        matches!(self, Self::Path(_))
    }

    /// 是否为文本内容。
    #[must_use]
    pub fn is_text(&self) -> bool {
        matches!(self, Self::Text(_))
    }

    /// 是否为图像内容。
    #[must_use]
    pub fn is_image(&self) -> bool {
        matches!(self, Self::Image(_))
    }

    /// 序列化为 OFD XML 字符串。
    #[must_use]
    pub fn to_xml_string(&self) -> String {
        use std::fmt::Write;
        match self {
            Self::Path(p) => {
                let mut xml = String::new();
                write!(
                    xml,
                    "<ofd:PathObject StrokeColor=\"{}\" LineWidth=\"{}\"",
                    p.stroke_color, p.line_width
                )
                .unwrap();
                if let Some(fc) = p.fill_color {
                    write!(xml, " FillColor=\"{fc}\"").unwrap();
                }
                writeln!(
                    xml,
                    "><ofd:AbbreviatedData>{}</ofd:AbbreviatedData></ofd:PathObject>",
                    p.data
                )
                .unwrap();
                xml
            }
            Self::Text(t) => {
                format!(
                    "<ofd:TextObject X=\"{}\" Y=\"{}\" FontSize=\"{}\" \
                     Font=\"{}\" Color=\"{}\">{}</ofd:TextObject>\n",
                    t.x, t.y, t.font_size, t.font, t.color, t.text
                )
            }
            Self::Image(img) => {
                format!(
                    "<ofd:ImageObject X=\"{}\" Y=\"{}\" Width=\"{}\" Height=\"{}\" />\n",
                    img.x, img.y, img.width, img.height
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_path() {
        let c = Content::path("M0 0L10 10");
        assert!(c.is_path());
        assert!(!c.is_text());
        assert!(!c.is_image());
        if let Content::Path(p) = &c {
            assert_eq!(p.data, "M0 0L10 10");
            assert_eq!(p.stroke_color, 0x00_0000);
            assert!((p.line_width - 0.35).abs() < f64::EPSILON);
            assert!(p.fill_color.is_none());
        }
    }

    #[test]
    fn test_content_text() {
        let c = Content::text("hello", 10.0, 20.0);
        assert!(!c.is_path());
        assert!(c.is_text());
        assert!(!c.is_image());
        if let Content::Text(t) = &c {
            assert_eq!(t.text, "hello");
            assert!((t.x - 10.0).abs() < f64::EPSILON);
            assert!((t.y - 20.0).abs() < f64::EPSILON);
            assert!((t.font_size - 12.0).abs() < f64::EPSILON);
            assert_eq!(t.font, "SimSun");
            assert_eq!(t.color, 0x00_0000);
        }
    }

    #[test]
    fn test_content_image() {
        let c = Content::image(vec![0x89, 0x50], 0.0, 0.0, 50.0, 50.0);
        assert!(!c.is_path());
        assert!(!c.is_text());
        assert!(c.is_image());
        if let Content::Image(img) = &c {
            assert_eq!(img.data, vec![0x89, 0x50]);
            assert!((img.width - 50.0).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn test_content_path_to_xml() {
        let c = Content::path("M0 0Z");
        let xml = c.to_xml_string();
        assert!(xml.contains("ofd:PathObject"));
        assert!(xml.contains("M0 0Z"));
        assert!(xml.contains("StrokeColor=\"0\""));
    }

    #[test]
    fn test_content_text_to_xml() {
        let c = Content::text("test", 1.0, 2.0);
        let xml = c.to_xml_string();
        assert!(xml.contains("ofd:TextObject"));
        assert!(xml.contains("test"));
        assert!(xml.contains("X=\"1\""));
        assert!(xml.contains("Y=\"2\""));
    }

    #[test]
    fn test_content_image_to_xml() {
        let c = Content::image(vec![1, 2, 3], 5.0, 10.0, 30.0, 40.0);
        let xml = c.to_xml_string();
        assert!(xml.contains("ofd:ImageObject"));
        assert!(xml.contains("Width=\"30\""));
        assert!(xml.contains("Height=\"40\""));
    }

    #[test]
    fn test_content_clone_debug() {
        let c = Content::path("M0 0");
        let c2 = c.clone();
        assert!(c2.is_path());
        assert!(format!("{c:?}").contains("Path"));
    }
}
