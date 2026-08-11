//! 字体包装器。
//!
//! 对应 Java: org.ofdrw.converter.font.FontWrapper

/// 字体包装器，标识字体是否为近似字体替换。
///
/// 对应 Java `FontWrapper<T>`。当原始字体不可用时，
/// 系统可能使用近似字体替代，此类型用于标记这种情况。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FontWrapper<T> {
    /// 字体对象。
    font: T,
    /// 是否启用了近似字体替换。
    enable_similar_font_replace: bool,
}

impl<T> FontWrapper<T> {
    /// 创建字体包装器。
    ///
    /// # 参数
    /// - `font`：字体对象
    /// - `enable_similar_font_replace`：是否为近似字体替换
    pub fn new(font: T, enable_similar_font_replace: bool) -> Self {
        Self {
            font,
            enable_similar_font_replace,
        }
    }

    /// 返回字体引用。
    pub fn font(&self) -> &T {
        &self.font
    }

    /// 返回字体可变引用。
    pub fn font_mut(&mut self) -> &mut T {
        &mut self.font
    }

    /// 是否启用了近似字体替换。
    pub fn is_similar_font_replace(&self) -> bool {
        self.enable_similar_font_replace
    }

    /// 设置近似字体替换标志。
    pub fn set_similar_font_replace(&mut self, enable: bool) {
        self.enable_similar_font_replace = enable;
    }

    /// 消耗包装器，返回内部字体。
    pub fn into_inner(self) -> T {
        self.font
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let fw = FontWrapper::new("Arial", false);
        assert_eq!(*fw.font(), "Arial");
        assert!(!fw.is_similar_font_replace());
    }

    #[test]
    fn test_similar_font_replace() {
        let mut fw = FontWrapper::new("Helvetica", true);
        assert!(fw.is_similar_font_replace());
        fw.set_similar_font_replace(false);
        assert!(!fw.is_similar_font_replace());
    }

    #[test]
    fn test_font_mut() {
        let mut fw = FontWrapper::new(String::from("Old"), false);
        *fw.font_mut() = String::from("New");
        assert_eq!(fw.font(), "New");
    }

    #[test]
    fn test_into_inner() {
        let fw = FontWrapper::new(42_u32, false);
        assert_eq!(fw.into_inner(), 42);
    }

    #[test]
    fn test_clone_eq() {
        let fw1 = FontWrapper::new("Arial", false);
        let fw2 = fw1.clone();
        assert_eq!(fw1, fw2);
    }
}
