//! PDF 字体包装器。
//!
//! 对应 Java: org.ofdrw.converter.font.PdfFontWrapper
//!
//! Java 版 `PdfFontWrapper` 将 OFD 字体信息与 PDF 字体引用关联。
//! Rust 版提供简化的数据结构，记录字体在 PDF 导出中的映射关系。

/// PDF 字体包装器。
///
/// 对应 Java: `org.ofdrw.converter.font.PdfFontWrapper`
///
/// 记录 OFD 字体在 PDF 转换中的映射信息。
#[derive(Debug, Clone)]
pub struct PdfFontWrapper {
    /// OFD 字体名称。
    pub font_name: String,
    /// PDF 中使用的字体名称（可能经过子集化重命名）。
    pub pdf_font_name: String,
    /// 字体文件路径（如果从文件加载）。
    pub font_path: Option<String>,
    /// 是否已嵌入到 PDF 中。
    pub embedded: bool,
    /// 字体子集标签（如 "ABCDEF+SimSun" 中的前缀）。
    pub subset_tag: Option<String>,
}

impl PdfFontWrapper {
    /// 创建新的 PDF 字体包装器。
    #[must_use]
    pub fn new(font_name: impl Into<String>, pdf_font_name: impl Into<String>) -> Self {
        Self {
            font_name: font_name.into(),
            pdf_font_name: pdf_font_name.into(),
            font_path: None,
            embedded: false,
            subset_tag: None,
        }
    }

    /// 设置字体文件路径。
    #[must_use]
    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.font_path = Some(path.into());
        self
    }

    /// 设置嵌入标志。
    #[must_use]
    pub fn with_embedded(mut self, embedded: bool) -> Self {
        self.embedded = embedded;
        self
    }

    /// 设置子集标签。
    #[must_use]
    pub fn with_subset_tag(mut self, tag: impl Into<String>) -> Self {
        self.subset_tag = Some(tag.into());
        self
    }

    /// 返回带子集标签的完整字体名称。
    ///
    /// 格式: `"ABCDEF+SimSun"` 或仅 `"SimSun"`（无子集标签时）。
    #[must_use]
    pub fn full_name(&self) -> String {
        match &self.subset_tag {
            Some(tag) => format!("{}+{}", tag, self.pdf_font_name),
            None => self.pdf_font_name.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let wrapper = PdfFontWrapper::new("SimSun", "SimSun");
        assert_eq!(wrapper.font_name, "SimSun");
        assert_eq!(wrapper.pdf_font_name, "SimSun");
        assert!(!wrapper.embedded);
    }

    #[test]
    fn test_with_path() {
        let wrapper =
            PdfFontWrapper::new("SimSun", "SimSun").with_path("/usr/share/fonts/simsun.ttf");
        assert_eq!(
            wrapper.font_path.as_deref(),
            Some("/usr/share/fonts/simsun.ttf")
        );
    }

    #[test]
    fn test_with_embedded() {
        let wrapper = PdfFontWrapper::new("SimSun", "SimSun").with_embedded(true);
        assert!(wrapper.embedded);
    }

    #[test]
    fn test_full_name_with_subset() {
        let wrapper = PdfFontWrapper::new("SimSun", "SimSun").with_subset_tag("ABCDEF");
        assert_eq!(wrapper.full_name(), "ABCDEF+SimSun");
    }

    #[test]
    fn test_full_name_without_subset() {
        let wrapper = PdfFontWrapper::new("SimSun", "SimSun");
        assert_eq!(wrapper.full_name(), "SimSun");
    }

    #[test]
    fn test_clone() {
        let w1 = PdfFontWrapper::new("Arial", "ArialMT").with_embedded(true);
        let w2 = w1.clone();
        assert_eq!(w1.font_name, w2.font_name);
        assert_eq!(w1.embedded, w2.embedded);
    }
}
