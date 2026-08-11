//! 元素收集器。
//!
//! 对应 Java: org.ofdrw.pkg.tool.ElemCup

/// 元素收集器。
///
/// 用于临时收集和处理 XML 元素。
///
/// 对应 Java: org.ofdrw.pkg.tool.ElemCup
#[derive(Debug, Clone, Default)]
pub struct ElemCup {
    /// 收集的元素内容。
    elements: Vec<String>,
}

impl ElemCup {
    /// 创建新的元素收集器。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 添加元素。
    pub fn add(&mut self, element: impl Into<String>) {
        self.elements.push(element.into());
    }

    /// 获取收集的元素数量。
    #[must_use]
    pub fn len(&self) -> usize {
        self.elements.len()
    }

    /// 是否为空。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    /// 获取所有元素。
    #[must_use]
    pub fn elements(&self) -> &[String] {
        &self.elements
    }

    /// 清空收集器。
    pub fn clear(&mut self) {
        self.elements.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn elem_cup_new() {
        let cup = ElemCup::new();
        assert!(cup.is_empty());
        assert_eq!(cup.len(), 0);
    }

    #[test]
    fn elem_cup_add() {
        let mut cup = ElemCup::new();
        cup.add("<element1/>");
        cup.add("<element2/>");
        assert_eq!(cup.len(), 2);
        assert!(!cup.is_empty());
    }

    #[test]
    fn elem_cup_elements() {
        let mut cup = ElemCup::new();
        cup.add("a");
        cup.add("b");
        assert_eq!(cup.elements(), &["a", "b"]);
    }

    #[test]
    fn elem_cup_clear() {
        let mut cup = ElemCup::new();
        cup.add("a");
        cup.clear();
        assert!(cup.is_empty());
    }
}
