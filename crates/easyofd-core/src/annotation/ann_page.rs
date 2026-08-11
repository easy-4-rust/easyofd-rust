//! 注释所在页信息。

use std::fmt::Write;

/// 对应 Java: org.ofdrw.core.annotation.AnnPage
///
/// 描述注释所在的页面，包含页码和该页的注释列表。
#[derive(Debug, Clone)]
pub struct AnnPage {
    /// 页面索引（0-based）。
    pub page_index: u32,
    /// 该页的注释文件路径。
    pub annot_file: Option<String>,
}

impl AnnPage {
    /// 创建一个新的注释页。
    #[must_use]
    pub fn new(page_index: u32) -> Self {
        Self {
            page_index,
            annot_file: None,
        }
    }

    /// 设置注释文件路径。
    #[must_use]
    pub fn annot_file(mut self, file: impl Into<String>) -> Self {
        self.annot_file = Some(file.into());
        self
    }

    /// 序列化为 XML 字符串。
    #[must_use]
    pub fn to_xml_string(&self) -> String {
        let mut xml = format!(r#"<ofd:AnnPage PageIndex="{}""#, self.page_index);
        if let Some(ref file) = self.annot_file {
            let _ = write!(xml, r#" AnnotFile="{file}""#);
        }
        xml.push_str(" />");
        xml
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ann_page_new() {
        let p = AnnPage::new(0);
        assert_eq!(p.page_index, 0);
        assert!(p.annot_file.is_none());
    }

    #[test]
    fn test_ann_page_builder() {
        let p = AnnPage::new(3).annot_file("Page_0/Annot.xml");
        assert_eq!(p.page_index, 3);
        assert_eq!(p.annot_file.as_deref(), Some("Page_0/Annot.xml"));
    }

    #[test]
    fn test_ann_page_to_xml_string_basic() {
        let p = AnnPage::new(2);
        let xml = p.to_xml_string();
        assert!(xml.contains(r#"PageIndex="2""#));
        assert!(!xml.contains("AnnotFile"));
    }

    #[test]
    fn test_ann_page_to_xml_string_with_file() {
        let p = AnnPage::new(1).annot_file("Page_1/ann.xml");
        let xml = p.to_xml_string();
        assert!(xml.contains(r#"AnnotFile="Page_1/ann.xml""#));
    }
}
