//! CT_Font 字体描述。

/// 对应 Java: org.ofdrw.core.text.font.CT_Font
///
/// 字体描述，定义文档中使用的字体属性。
/// 对应 GB/T 33190-2016 第 11.1 节图 58 表 44。
#[allow(non_camel_case_types, clippy::struct_excessive_bools)]
#[derive(Debug, Clone)]
pub struct CT_Font {
    /// 字体 ID。
    pub id: u32,
    /// 字体名称（如 "SimSun"）。
    pub font_name: String,
    /// 字体族名称（可选）。
    pub family_name: Option<String>,
    /// 字符集（可选，如 "GB2312"、"Unicode"）。
    pub charset: Option<String>,
    /// 是否斜体。
    pub italic: bool,
    /// 是否粗体。
    pub bold: bool,
    /// 是否衬线字体。
    pub serif: bool,
    /// 是否等宽字体。
    pub fixed_width: bool,
    /// 字体文件路径（可选）。
    pub font_file: Option<String>,
}

impl CT_Font {
    /// 创建新的字体描述。
    #[must_use]
    pub fn new(id: u32, font_name: impl Into<String>) -> Self {
        Self {
            id,
            font_name: font_name.into(),
            family_name: None,
            charset: None,
            italic: false,
            bold: false,
            serif: false,
            fixed_width: false,
            font_file: None,
        }
    }

    /// 设置字体名称。
    #[must_use]
    pub fn font_name(mut self, name: impl Into<String>) -> Self {
        self.font_name = name.into();
        self
    }

    /// 设置字体族名称。
    #[must_use]
    pub fn family_name(mut self, name: impl Into<String>) -> Self {
        self.family_name = Some(name.into());
        self
    }

    /// 设置字符集。
    #[must_use]
    pub fn charset(mut self, charset: impl Into<String>) -> Self {
        self.charset = Some(charset.into());
        self
    }

    /// 设置斜体。
    #[must_use]
    pub fn italic(mut self, italic: bool) -> Self {
        self.italic = italic;
        self
    }

    /// 设置粗体。
    #[must_use]
    pub fn bold(mut self, bold: bool) -> Self {
        self.bold = bold;
        self
    }

    /// 设置衬线字体。
    #[must_use]
    pub fn serif(mut self, serif: bool) -> Self {
        self.serif = serif;
        self
    }

    /// 设置等宽字体。
    #[must_use]
    pub fn fixed_width(mut self, fixed: bool) -> Self {
        self.fixed_width = fixed;
        self
    }

    /// 设置字体文件路径。
    #[must_use]
    pub fn font_file(mut self, path: impl Into<String>) -> Self {
        self.font_file = Some(path.into());
        self
    }

    /// 获取字体 ID。
    #[must_use]
    pub fn get_id(&self) -> u32 {
        self.id
    }

    /// 获取字体名称。
    #[must_use]
    pub fn get_font_name(&self) -> &str {
        &self.font_name
    }

    /// 获取字体族名称。
    #[must_use]
    pub fn get_family_name(&self) -> Option<&str> {
        self.family_name.as_deref()
    }

    /// 获取字符集。
    #[must_use]
    pub fn get_charset(&self) -> Option<&str> {
        self.charset.as_deref()
    }

    /// 是否斜体。
    #[must_use]
    pub fn is_italic(&self) -> bool {
        self.italic
    }

    /// 是否粗体。
    #[must_use]
    pub fn is_bold(&self) -> bool {
        self.bold
    }

    /// 是否衬线字体。
    #[must_use]
    pub fn is_serif(&self) -> bool {
        self.serif
    }

    /// 是否等宽字体。
    #[must_use]
    pub fn is_fixed_width(&self) -> bool {
        self.fixed_width
    }

    /// 序列化为 OFD XML 字符串。
    #[must_use]
    pub fn to_xml_string(&self) -> String {
        use std::fmt::Write;
        let mut xml = format!(
            "<ofd:Font ID=\"{}\" FontName=\"{}\"",
            self.id, self.font_name
        );
        if let Some(ref fn_) = self.family_name {
            write!(xml, " FamilyName=\"{fn_}\"").expect("写入内存缓冲区不会失败");
        }
        if let Some(ref cs) = self.charset {
            write!(xml, " Charset=\"{cs}\"").expect("写入内存缓冲区不会失败");
        }
        if self.italic {
            xml.push_str(" Italic=\"true\"");
        }
        if self.bold {
            xml.push_str(" Bold=\"true\"");
        }
        if self.serif {
            xml.push_str(" Serif=\"true\"");
        }
        if self.fixed_width {
            xml.push_str(" FixedWidth=\"true\"");
        }
        if let Some(ref ff) = self.font_file {
            write!(xml, " FontFile=\"{ff}\"").expect("写入内存缓冲区不会失败");
        }
        xml.push_str(" />");
        xml
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ct_font_new() {
        let font = CT_Font::new(1, "SimSun");
        assert_eq!(font.id, 1);
        assert_eq!(font.font_name, "SimSun");
        assert!(!font.italic);
        assert!(!font.bold);
        assert!(!font.serif);
        assert!(!font.fixed_width);
        assert!(font.family_name.is_none());
        assert!(font.font_file.is_none());
    }

    #[test]
    fn test_ct_font_builder() {
        let font = CT_Font::new(2, "SimHei")
            .family_name("Sans-Serif")
            .charset("Unicode")
            .italic(true)
            .bold(true)
            .serif(false)
            .fixed_width(true)
            .font_file("simhei.ttf");
        assert_eq!(font.get_font_name(), "SimHei");
        assert_eq!(font.get_family_name(), Some("Sans-Serif"));
        assert_eq!(font.get_charset(), Some("Unicode"));
        assert!(font.is_italic());
        assert!(font.is_bold());
        assert!(!font.is_serif());
        assert!(font.is_fixed_width());
        assert_eq!(font.font_file.as_deref(), Some("simhei.ttf"));
    }

    #[test]
    fn test_ct_font_to_xml_minimal() {
        let font = CT_Font::new(1, "SimSun");
        let xml = font.to_xml_string();
        assert!(xml.contains("ID=\"1\""));
        assert!(xml.contains("FontName=\"SimSun\""));
        assert!(xml.contains("<ofd:Font"));
        assert!(xml.ends_with(" />"));
    }

    #[test]
    fn test_ct_font_to_xml_full() {
        let font = CT_Font::new(5, "Arial")
            .family_name("Sans-Serif")
            .charset("Unicode")
            .bold(true)
            .italic(true)
            .serif(true)
            .fixed_width(true)
            .font_file("arial.ttf");
        let xml = font.to_xml_string();
        assert!(xml.contains("ID=\"5\""));
        assert!(xml.contains("FontName=\"Arial\""));
        assert!(xml.contains("FamilyName=\"Sans-Serif\""));
        assert!(xml.contains("Charset=\"Unicode\""));
        assert!(xml.contains("Bold=\"true\""));
        assert!(xml.contains("Italic=\"true\""));
        assert!(xml.contains("Serif=\"true\""));
        assert!(xml.contains("FixedWidth=\"true\""));
        assert!(xml.contains("FontFile=\"arial.ttf\""));
    }

    #[test]
    fn test_ct_font_clone_debug() {
        let font = CT_Font::new(1, "x");
        let font2 = font.clone();
        assert_eq!(font2.get_font_name(), "x");
        assert!(format!("{font:?}").contains("CT_Font"));
    }
}
