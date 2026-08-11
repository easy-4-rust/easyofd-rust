//! 附件容器。

use super::ct_attachment::CTAttachment;

/// 对应 Java: org.ofdrw.core.attachment.Attachments
///
/// 附件根容器，对应 Attachments.xml，包含文档中所有附件。
#[derive(Debug, Clone)]
pub struct Attachments {
    /// 文档中所有附件列表。
    pub items: Vec<CTAttachment>,
}

impl Attachments {
    /// 创建一个空的附件容器。
    #[must_use]
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    /// 添加附件。
    #[must_use]
    pub fn add_attachment(mut self, attachment: CTAttachment) -> Self {
        self.items.push(attachment);
        self
    }

    /// 返回附件数量。
    #[must_use]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// 是否为空。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// 序列化为 XML 字符串。
    #[must_use]
    pub fn to_xml_string(&self) -> String {
        let mut xml = String::from(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
        xml.push('\n');
        xml.push_str(r#"<ofd:Attachments xmlns:ofd="http://www.ofdspec.org/2016">"#);
        for item in &self.items {
            xml.push('\n');
            xml.push_str(&item.to_xml_string());
        }
        xml.push_str("\n</ofd:Attachments>");
        xml
    }
}

impl Default for Attachments {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attachments_new() {
        let a = Attachments::new();
        assert!(a.items.is_empty());
        assert!(a.is_empty());
        assert_eq!(a.len(), 0);
    }

    #[test]
    fn test_attachments_default() {
        let a = Attachments::default();
        assert!(a.is_empty());
    }

    #[test]
    fn test_attachments_add() {
        let a = Attachments::new()
            .add_attachment(CTAttachment::new("a1", "readme.txt"))
            .add_attachment(CTAttachment::new("a2", "data.xlsx"));
        assert_eq!(a.len(), 2);
        assert!(!a.is_empty());
        assert_eq!(a.items[0].id, "a1");
        assert_eq!(a.items[1].id, "a2");
    }

    #[test]
    fn test_attachments_to_xml_string() {
        let a = Attachments::new()
            .add_attachment(CTAttachment::new("a1", "test.pdf").format("application/pdf"));
        let xml = a.to_xml_string();
        assert!(xml.contains(r#"<?xml version="1.0""#));
        assert!(xml.contains("ofd:Attachments"));
        assert!(xml.contains(r#"ID="a1""#));
        assert!(xml.contains(r#"Name="test.pdf""#));
        assert!(xml.contains("</ofd:Attachments>"));
    }

    #[test]
    fn test_attachments_to_xml_string_empty() {
        let a = Attachments::new();
        let xml = a.to_xml_string();
        assert!(xml.contains("ofd:Attachments"));
        assert!(xml.contains("</ofd:Attachments>"));
    }

    #[test]
    fn test_attachments_clone_debug() {
        let a = Attachments::new().add_attachment(CTAttachment::new("x", "y.txt"));
        let a2 = a.clone();
        assert_eq!(a2.len(), 1);
        assert!(format!("{a:?}").contains("Attachments"));
    }
}
