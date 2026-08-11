//! TextCode 文字定位。

/// 对应 Java: org.ofdrw.core.text.TextCode
///
/// 文字定位信息，使用严格的定位数据来确定文字在页面上的位置。
/// 文字对象使用 TextCode 列表来存储文字内容和每个字符的精确定位。
/// 对应 GB/T 33190-2016 第 11.3 节图 61 表 46。
#[derive(Debug, Clone)]
pub struct TextCode {
    /// 文本内容（字形编码序列）。
    pub content: String,
    /// 文本起始 X 坐标（mm）。None 表示使用上一个 TextCode 的结束位置。
    pub x: Option<f64>,
    /// 文本起始 Y 坐标（mm）。None 表示使用上一个 TextCode 的 Y 坐标。
    pub y: Option<f64>,
    /// X 方向偏移量列表（每个字形的增量偏移，单位 mm）。
    pub delta_x: Vec<f64>,
    /// Y 方向偏移量列表（每个字形的增量偏移，单位 mm）。
    pub delta_y: Vec<f64>,
}

impl TextCode {
    /// 创建空的文字定位。
    #[must_use]
    pub fn new() -> Self {
        Self {
            content: String::new(),
            x: None,
            y: None,
            delta_x: Vec::new(),
            delta_y: Vec::new(),
        }
    }

    /// 使用内容创建文字定位。
    #[must_use]
    pub fn with_content(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            ..Self::new()
        }
    }

    /// 设置文本内容。
    #[must_use]
    pub fn content(mut self, content: impl Into<String>) -> Self {
        self.content = content.into();
        self
    }

    /// 设置坐标。
    #[must_use]
    pub fn coordinate(mut self, x: f64, y: f64) -> Self {
        self.x = Some(x);
        self.y = Some(y);
        self
    }

    /// 设置 X 坐标。
    #[must_use]
    pub fn x(mut self, x: f64) -> Self {
        self.x = Some(x);
        self
    }

    /// 设置 Y 坐标。
    #[must_use]
    pub fn y(mut self, y: f64) -> Self {
        self.y = Some(y);
        self
    }

    /// 设置 X 方向偏移量列表。
    #[must_use]
    pub fn delta_x(mut self, deltas: Vec<f64>) -> Self {
        self.delta_x = deltas;
        self
    }

    /// 设置 Y 方向偏移量列表。
    #[must_use]
    pub fn delta_y(mut self, deltas: Vec<f64>) -> Self {
        self.delta_y = deltas;
        self
    }

    /// 获取内容。
    #[must_use]
    pub fn get_content(&self) -> &str {
        &self.content
    }

    /// 获取 X 坐标。
    #[must_use]
    pub fn get_x(&self) -> Option<f64> {
        self.x
    }

    /// 获取 Y 坐标。
    #[must_use]
    pub fn get_y(&self) -> Option<f64> {
        self.y
    }

    /// 序列化为 OFD XML 字符串。
    #[must_use]
    pub fn to_xml_string(&self) -> String {
        use std::fmt::Write;
        let mut xml = String::from("<ofd:TextCode");
        if let Some(x) = self.x {
            write!(xml, " X=\"{x}\"").unwrap();
        }
        if let Some(y) = self.y {
            write!(xml, " Y=\"{y}\"").unwrap();
        }
        if !self.delta_x.is_empty() {
            xml.push_str(" DeltaX=\"");
            for (i, d) in self.delta_x.iter().enumerate() {
                if i > 0 {
                    xml.push(' ');
                }
                write!(xml, "{d}").unwrap();
            }
            xml.push('"');
        }
        if !self.delta_y.is_empty() {
            xml.push_str(" DeltaY=\"");
            for (i, d) in self.delta_y.iter().enumerate() {
                if i > 0 {
                    xml.push(' ');
                }
                write!(xml, "{d}").unwrap();
            }
            xml.push('"');
        }
        xml.push('>');
        xml.push_str(&self.content);
        xml.push_str("</ofd:TextCode>");
        xml
    }
}

impl Default for TextCode {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_code_new() {
        let tc = TextCode::new();
        assert!(tc.content.is_empty());
        assert!(tc.x.is_none());
        assert!(tc.y.is_none());
        assert!(tc.delta_x.is_empty());
        assert!(tc.delta_y.is_empty());
    }

    #[test]
    fn test_text_code_with_content() {
        let tc = TextCode::with_content("Hello");
        assert_eq!(tc.get_content(), "Hello");
        assert!(tc.x.is_none());
    }

    #[test]
    fn test_text_code_builder() {
        let tc = TextCode::new()
            .content("World")
            .coordinate(10.0, 20.0)
            .delta_x(vec![6.0, 6.0, 6.0]);
        assert_eq!(tc.get_content(), "World");
        assert!((tc.get_x().unwrap() - 10.0).abs() < f64::EPSILON);
        assert!((tc.get_y().unwrap() - 20.0).abs() < f64::EPSILON);
        assert_eq!(tc.delta_x.len(), 3);
    }

    #[test]
    fn test_text_code_to_xml_minimal() {
        let tc = TextCode::with_content("A");
        let xml = tc.to_xml_string();
        assert!(xml.contains("<ofd:TextCode"));
        assert!(xml.contains(">A</ofd:TextCode>"));
    }

    #[test]
    fn test_text_code_to_xml_with_position() {
        let tc = TextCode::with_content("Hi").coordinate(5.0, 10.0);
        let xml = tc.to_xml_string();
        assert!(xml.contains("X=\"5\""));
        assert!(xml.contains("Y=\"10\""));
        assert!(xml.contains(">Hi</ofd:TextCode>"));
    }

    #[test]
    fn test_text_code_to_xml_with_deltas() {
        let tc = TextCode::with_content("ABC")
            .x(0.0)
            .delta_x(vec![6.0, 6.0, 6.0])
            .delta_y(vec![0.0, 0.0]);
        let xml = tc.to_xml_string();
        assert!(xml.contains("DeltaX=\"6 6 6\""));
        assert!(xml.contains("DeltaY=\"0 0\""));
    }

    #[test]
    fn test_text_code_clone_debug() {
        let tc = TextCode::with_content("x");
        let tc2 = tc.clone();
        assert_eq!(tc2.get_content(), "x");
        assert!(format!("{tc:?}").contains("TextCode"));
    }
}
