use crate::font_descriptor::FontDescriptor;

/// 字体注册表：管理字体 ID 到 [`FontDescriptor`] 的映射。
///
/// 字体以注册顺序分配从 0 开始递增的 ID。
pub struct FontRegistry {
    fonts: Vec<FontDescriptor>,
}

impl FontRegistry {
    /// 创建空的字体注册表。
    #[must_use]
    pub fn new() -> Self {
        Self { fonts: Vec::new() }
    }

    /// 注册一个字体，返回其分配的 ID。
    pub fn register(&mut self, font: FontDescriptor) -> usize {
        let id = self.fonts.len();
        self.fonts.push(font);
        id
    }

    /// 根据 ID 获取字体描述符的引用。
    pub fn get(&self, id: usize) -> Option<&FontDescriptor> {
        self.fonts.get(id)
    }

    /// 根据字体名称查找第一个匹配的字体 ID。
    ///
    /// 同时匹配 `font_name` 和 `family_name`，优先返回 `font_name` 匹配的结果。
    pub fn find_by_name(&self, name: &str) -> Option<usize> {
        // 优先精确匹配 font_name
        if let Some(idx) = self.fonts.iter().position(|f| f.font_name == name) {
            return Some(idx);
        }
        // 其次匹配 family_name
        self.fonts.iter().position(|f| f.family_name == name)
    }

    /// 返回注册表中的字体数量。
    pub fn len(&self) -> usize {
        self.fonts.len()
    }

    /// 判断注册表是否为空。
    pub fn is_empty(&self) -> bool {
        self.fonts.is_empty()
    }
}

impl Default for FontRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_and_get() {
        let mut reg = FontRegistry::new();
        let id = reg.register(FontDescriptor::new("SimSun", "宋体", 10.0));
        assert_eq!(id, 0);
        let fd = reg.get(id).expect("字体应存在");
        assert_eq!(fd.font_name, "SimSun");
    }

    #[test]
    fn test_find_by_font_name() {
        let mut reg = FontRegistry::new();
        reg.register(FontDescriptor::new("SimHei", "黑体", 11.0));
        reg.register(FontDescriptor::new("SimSun", "宋体", 10.0));
        assert_eq!(reg.find_by_name("SimSun"), Some(1));
    }

    #[test]
    fn test_find_by_family_name() {
        let mut reg = FontRegistry::new();
        reg.register(FontDescriptor::new("SimHei", "黑体", 11.0));
        assert_eq!(reg.find_by_name("黑体"), Some(0));
    }

    #[test]
    fn test_find_not_found() {
        let reg = FontRegistry::new();
        assert_eq!(reg.find_by_name("不存在"), None);
    }

    #[test]
    fn test_len_and_is_empty() {
        let mut reg = FontRegistry::new();
        assert!(reg.is_empty());
        assert_eq!(reg.len(), 0);
        reg.register(FontDescriptor::new("X", "Y", 1.0));
        assert!(!reg.is_empty());
        assert_eq!(reg.len(), 1);
    }
}
