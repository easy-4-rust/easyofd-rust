//! 大纲树。
//!
//! 对应 Java: org.ofdrw.core.basicStructure.outlines.Outlines

/// 大纲树。
///
/// 对应 Java: org.ofdrw.core.basicStructure.outlines.Outlines
#[derive(Debug, Clone, Default)]
pub struct Outlines {
    /// 大纲元素列表。
    pub elements: Vec<CT_OutlineElem>,
}

/// 大纲元素。
///
/// 对应 Java: org.ofdrw.core.basicStructure.outlines.CT_OutlineElem
#[allow(non_camel_case_types)]
#[derive(Debug, Clone)]
pub struct CT_OutlineElem {
    /// 标题。
    pub title: String,
    /// 目标页码（可选）。
    pub page: Option<u32>,
    /// 子大纲元素。
    pub children: Vec<CT_OutlineElem>,
}

impl CT_OutlineElem {
    /// 创建大纲元素。
    #[must_use]
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            page: None,
            children: Vec::new(),
        }
    }

    /// 设置目标页码。
    #[must_use]
    pub fn page(mut self, page: u32) -> Self {
        self.page = Some(page);
        self
    }

    /// 添加子元素。
    pub fn add_child(&mut self, child: CT_OutlineElem) {
        self.children.push(child);
    }
}

impl Outlines {
    /// 创建空大纲。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 添加大纲元素。
    pub fn add(&mut self, elem: CT_OutlineElem) {
        self.elements.push(elem);
    }

    /// 获取元素数量。
    #[must_use]
    pub fn len(&self) -> usize {
        self.elements.len()
    }

    /// 是否为空。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn outlines_new() {
        let o = Outlines::new();
        assert!(o.is_empty());
    }

    #[test]
    fn outlines_add() {
        let mut o = Outlines::new();
        o.add(CT_OutlineElem::new("Chapter 1").page(1));
        o.add(CT_OutlineElem::new("Chapter 2").page(5));
        assert_eq!(o.len(), 2);
    }

    #[test]
    fn outline_elem_children() {
        let mut root = CT_OutlineElem::new("Root");
        root.add_child(CT_OutlineElem::new("Child 1"));
        root.add_child(CT_OutlineElem::new("Child 2"));
        assert_eq!(root.children.len(), 2);
    }

    #[test]
    fn outline_elem_page() {
        let elem = CT_OutlineElem::new("Test").page(10);
        assert_eq!(elem.page, Some(10));
    }
}
