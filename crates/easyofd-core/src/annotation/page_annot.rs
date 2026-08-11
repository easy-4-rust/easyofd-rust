//! 分页注释文件。

use super::annot::Annot;

/// 对应 Java: org.ofdrw.core.annotation.PageAnnot
///
/// 分页注释文件，包含某一页的所有注释列表。
#[derive(Debug, Clone)]
pub struct PageAnnot {
    /// 页码。
    pub page_id: u32,
    /// 该页包含的注释列表。
    pub annotations: Vec<Annot>,
}

impl PageAnnot {
    /// 创建一个新的分页注释。
    #[must_use]
    pub fn new(page_id: u32) -> Self {
        Self {
            page_id,
            annotations: Vec::new(),
        }
    }

    /// 添加注释。
    #[must_use]
    pub fn add_annot(mut self, annot: Annot) -> Self {
        self.annotations.push(annot);
        self
    }

    /// 序列化为 XML 字符串。
    #[must_use]
    pub fn to_xml_string(&self) -> String {
        let mut xml = format!(r#"<ofd:PageAnnot PageID="{}">"#, self.page_id);
        for annot in &self.annotations {
            xml.push('\n');
            xml.push_str(&annot.to_xml_string());
        }
        xml.push_str("\n</ofd:PageAnnot>");
        xml
    }
}

#[cfg(test)]
mod tests {
    use super::super::annot_type::AnnotType;
    use super::*;

    #[test]
    fn test_page_annot_new() {
        let pa = PageAnnot::new(5);
        assert_eq!(pa.page_id, 5);
        assert!(pa.annotations.is_empty());
    }

    #[test]
    fn test_page_annot_add_annot() {
        let pa = PageAnnot::new(1)
            .add_annot(Annot::new("a1", AnnotType::Text))
            .add_annot(Annot::new("a2", AnnotType::Link));
        assert_eq!(pa.annotations.len(), 2);
        assert_eq!(pa.annotations[0].id, "a1");
        assert_eq!(pa.annotations[1].id, "a2");
    }

    #[test]
    fn test_page_annot_to_xml_string_empty() {
        let pa = PageAnnot::new(0);
        let xml = pa.to_xml_string();
        assert!(xml.contains(r#"PageID="0""#));
        assert!(xml.contains("</ofd:PageAnnot>"));
    }

    #[test]
    fn test_page_annot_to_xml_string_with_annots() {
        let pa = PageAnnot::new(2).add_annot(Annot::new("a1", AnnotType::Stamp));
        let xml = pa.to_xml_string();
        assert!(xml.contains(r#"ID="a1""#));
        assert!(xml.contains("ofd:Annot"));
    }
}
