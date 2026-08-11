//! 字体资源列表。
//!
//! 对应 Java: org.ofdrw.core.basicStructure.res.resources.Fonts

/// 字体资源列表。
///
/// 对应 Java: org.ofdrw.core.basicStructure.res.resources.Fonts
#[derive(Debug, Clone, Default)]
pub struct Fonts {
    /// 字体列表（XML 片段或字体描述）。
    pub fonts: Vec<String>,
}

impl Fonts {
    /// 创建空字体列表。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 添加字体。
    pub fn add_font(&mut self, font: impl Into<String>) {
        self.fonts.push(font.into());
    }

    /// 获取字体数量。
    #[must_use]
    pub fn len(&self) -> usize {
        self.fonts.len()
    }

    /// 是否为空。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.fonts.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fonts_new() {
        let f = Fonts::new();
        assert!(f.is_empty());
    }

    #[test]
    fn fonts_add() {
        let mut f = Fonts::new();
        f.add_font("<ofd:Font ID=\"1\"/>");
        assert_eq!(f.len(), 1);
    }
}
