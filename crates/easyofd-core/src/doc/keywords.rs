//! 文档关键词。
//!
//! 对应 Java: org.ofdrw.core.basicStructure.ofd.docInfo.Keywords

/// 文档关键词。
///
/// 对应 Java: org.ofdrw.core.basicStructure.ofd.docInfo.Keywords
#[derive(Debug, Clone, Default)]
pub struct Keywords {
    /// 关键词列表。
    pub keywords: Vec<String>,
}

impl Keywords {
    /// 创建空关键词列表。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 添加关键词。
    pub fn add(&mut self, keyword: impl Into<String>) {
        self.keywords.push(keyword.into());
    }

    /// 获取关键词数量。
    #[must_use]
    pub fn len(&self) -> usize {
        self.keywords.len()
    }

    /// 是否为空。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.keywords.is_empty()
    }

    /// 序列化为逗号分隔的字符串。
    #[must_use]
    pub fn to_string_joined(&self) -> String {
        self.keywords.join(",")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keywords_new() {
        let k = Keywords::new();
        assert!(k.is_empty());
    }

    #[test]
    fn keywords_add() {
        let mut k = Keywords::new();
        k.add("OFD");
        k.add("PDF");
        assert_eq!(k.len(), 2);
    }

    #[test]
    fn keywords_to_string_joined() {
        let mut k = Keywords::new();
        k.add("A");
        k.add("B");
        assert_eq!(k.to_string_joined(), "A,B");
    }
}
