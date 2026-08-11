//! 注释根容器。

use std::fmt::Write;

use super::ann_page::AnnPage;

/// 对应 Java: org.ofdrw.core.annotation.Annotations
///
/// 注释根容器，对应 Annotations.xml，包含文档中所有页的注释索引。
#[derive(Debug, Clone)]
pub struct Annotations {
    /// 注释所属的文档 ID。
    pub doc_id: Option<String>,
    /// 各页的注释索引。
    pub pages: Vec<AnnPage>,
}

impl Annotations {
    /// 创建一个空的注释容器。
    #[must_use]
    pub fn new() -> Self {
        Self {
            doc_id: None,
            pages: Vec::new(),
        }
    }

    /// 设置文档 ID。
    #[must_use]
    pub fn doc_id(mut self, id: impl Into<String>) -> Self {
        self.doc_id = Some(id.into());
        self
    }

    /// 添加注释页。
    #[must_use]
    pub fn add_page(mut self, page: AnnPage) -> Self {
        self.pages.push(page);
        self
    }

    /// 序列化为 XML 字符串。
    #[must_use]
    pub fn to_xml_string(&self) -> String {
        let mut xml = String::from(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
        xml.push('\n');
        xml.push_str(r#"<ofd:Annotations xmlns:ofd="http://www.ofdspec.org/2016""#);
        if let Some(ref doc_id) = self.doc_id {
            let _ = write!(xml, r#" DocID="{doc_id}""#);
        }
        xml.push('>');
        for page in &self.pages {
            xml.push('\n');
            xml.push_str(&page.to_xml_string());
        }
        xml.push_str("\n</ofd:Annotations>");
        xml
    }
}

impl Default for Annotations {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_annotations_new() {
        let a = Annotations::new();
        assert!(a.doc_id.is_none());
        assert!(a.pages.is_empty());
    }

    #[test]
    fn test_annotations_default() {
        let a = Annotations::default();
        assert!(a.pages.is_empty());
    }

    #[test]
    fn test_annotations_builder() {
        let a = Annotations::new()
            .doc_id("doc-001")
            .add_page(AnnPage::new(0))
            .add_page(AnnPage::new(1));
        assert_eq!(a.doc_id.as_deref(), Some("doc-001"));
        assert_eq!(a.pages.len(), 2);
    }

    #[test]
    fn test_annotations_to_xml_string() {
        let a = Annotations::new()
            .doc_id("doc-abc")
            .add_page(AnnPage::new(0));
        let xml = a.to_xml_string();
        assert!(xml.contains(r#"<?xml version="1.0""#));
        assert!(xml.contains("ofd:Annotations"));
        assert!(xml.contains(r#"DocID="doc-abc""#));
        assert!(xml.contains(r#"PageIndex="0""#));
        assert!(xml.contains("</ofd:Annotations>"));
    }

    #[test]
    fn test_annotations_to_xml_string_no_doc_id() {
        let a = Annotations::new();
        let xml = a.to_xml_string();
        assert!(!xml.contains("DocID"));
        assert!(xml.contains("</ofd:Annotations>"));
    }

    #[test]
    fn test_annotations_clone_debug() {
        let a = Annotations::new().doc_id("x");
        let a2 = a.clone();
        assert_eq!(a2.doc_id.as_deref(), Some("x"));
        assert!(format!("{a:?}").contains("Annotations"));
    }
}
