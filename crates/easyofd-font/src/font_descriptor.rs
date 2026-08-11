/// OFD 字体描述符，对应 Java 版 `CT_Font`。
///
/// 记录字体的基本属性，用于排版计算与字体管理。
#[derive(Debug, Clone, PartialEq)]
pub struct FontDescriptor {
    /// 字体标识名称（如 "SimSun"、"SimHei"）
    pub font_name: String,
    /// 字体族名称（如 "宋体"、"黑体"）
    pub family_name: String,
    /// 是否等宽字体
    pub fixed_width: bool,
    /// 是否斜体
    pub italic: bool,
    /// 是否粗体
    pub bold: bool,
    /// 平均字符宽度（单位：pt）
    pub char_width: f64,
}

impl FontDescriptor {
    /// 创建新的字体描述符。
    pub fn new(
        font_name: impl Into<String>,
        family_name: impl Into<String>,
        char_width: f64,
    ) -> Self {
        Self {
            font_name: font_name.into(),
            family_name: family_name.into(),
            fixed_width: false,
            italic: false,
            bold: false,
            char_width,
        }
    }

    /// 设置是否等宽字体，返回自身以支持链式调用。
    #[must_use]
    pub fn with_fixed_width(mut self, fixed_width: bool) -> Self {
        self.fixed_width = fixed_width;
        self
    }

    /// 设置是否斜体，返回自身以支持链式调用。
    #[must_use]
    pub fn with_italic(mut self, italic: bool) -> Self {
        self.italic = italic;
        self
    }

    /// 设置是否粗体，返回自身以支持链式调用。
    #[must_use]
    pub fn with_bold(mut self, bold: bool) -> Self {
        self.bold = bold;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_font_descriptor() {
        let fd = FontDescriptor::new("SimSun", "宋体", 10.0);
        assert_eq!(fd.font_name, "SimSun");
        assert_eq!(fd.family_name, "宋体");
        assert!(!fd.fixed_width);
        assert!(!fd.italic);
        assert!(!fd.bold);
        assert!((fd.char_width - 10.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_builder_chain() {
        let fd = FontDescriptor::new("Courier", "等宽", 12.0)
            .with_fixed_width(true)
            .with_italic(true)
            .with_bold(true);
        assert!(fd.fixed_width);
        assert!(fd.italic);
        assert!(fd.bold);
    }

    #[test]
    fn test_partial_eq() {
        let a = FontDescriptor::new("A", "B", 8.0);
        let b = FontDescriptor::new("A", "B", 8.0);
        let c = FontDescriptor::new("A", "B", 9.0);
        assert_eq!(a, b);
        assert_ne!(a, c);
    }
}
