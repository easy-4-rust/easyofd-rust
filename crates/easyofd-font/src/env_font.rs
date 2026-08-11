//! 环境字体管理。
//!
//! 对应 Java: org.ofdrw.font.EnvFont

use crate::font_descriptor::FontDescriptor;
use crate::font_name::FontName;

/// 环境字体管理器。
///
/// 对应 Java: org.ofdrw.font.EnvFont
///
/// 管理系统环境中可用的字体，提供字体查找和匹配功能。
#[derive(Debug, Default)]
pub struct EnvFont {
    /// 已注册的字体列表。
    pub fonts: Vec<FontDescriptor>,
}

impl EnvFont {
    /// 创建新的环境字体管理器。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 注册字体。
    pub fn register(&mut self, font: FontDescriptor) {
        self.fonts.push(font);
    }

    /// 按名称查找字体。
    #[must_use]
    pub fn find_by_name(&self, name: &str) -> Option<&FontDescriptor> {
        self.fonts.iter().find(|f| f.family_name == name)
    }

    /// 按 FontName 枚举查找字体。
    #[must_use]
    pub fn find_by_font_name(&self, font_name: FontName) -> Option<&FontDescriptor> {
        self.find_by_name(font_name.as_str())
    }

    /// 获取已注册字体数量。
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
    fn test_env_font_new() {
        let env = EnvFont::new();
        assert!(env.is_empty());
    }

    #[test]
    fn test_env_font_register() {
        let mut env = EnvFont::new();
        env.register(FontDescriptor::new("SimSun", "SimSun", 10.0));
        assert_eq!(env.len(), 1);
    }

    #[test]
    fn test_env_font_find_by_name() {
        let mut env = EnvFont::new();
        env.register(FontDescriptor::new("SimHei", "SimHei", 10.0));
        assert!(env.find_by_name("SimHei").is_some());
        assert!(env.find_by_name("NotExist").is_none());
    }
}
