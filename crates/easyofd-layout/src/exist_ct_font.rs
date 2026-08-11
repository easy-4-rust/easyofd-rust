//! 已存在的 CT_Font 引用。
//!
//! 对应 Java: org.ofdrw.layout.engine.ExistCTFont

/// 已存在的 CT_Font 引用，用于字体去重。
///
/// 对应 Java: ofdrw layout engine ExistCTFont。
#[derive(Debug, Clone, PartialEq)]
pub struct ExistCtFont {
    /// 字体 ID。
    pub font_id: u32,
    /// 字体名称。
    pub font_name: String,
    /// 字体族名称（可选）。
    pub family_name: Option<String>,
    /// 是否已嵌入。
    pub embedded: bool,
}

impl ExistCtFont {
    /// 创建已存在字体引用。
    #[must_use]
    pub fn new(font_id: u32, font_name: impl Into<String>) -> Self {
        Self {
            font_id,
            font_name: font_name.into(),
            family_name: None,
            embedded: false,
        }
    }

    /// 设置字体族名称。
    #[must_use]
    pub fn family_name(mut self, name: impl Into<String>) -> Self {
        self.family_name = Some(name.into());
        self
    }

    /// 设置是否已嵌入。
    #[must_use]
    pub fn embedded(mut self, embedded: bool) -> Self {
        self.embedded = embedded;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let font = ExistCtFont::new(1, "SimSun");
        assert_eq!(font.font_id, 1);
        assert_eq!(font.font_name, "SimSun");
        assert!(font.family_name.is_none());
        assert!(!font.embedded);
    }

    #[test]
    fn test_builders() {
        let font = ExistCtFont::new(2, "SimHei")
            .family_name("黑体")
            .embedded(true);
        assert_eq!(font.family_name.as_deref(), Some("黑体"));
        assert!(font.embedded);
    }

    #[test]
    fn test_clone_eq() {
        let a = ExistCtFont::new(1, "SimSun").embedded(true);
        let b = a.clone();
        assert_eq!(a, b);
    }
}
