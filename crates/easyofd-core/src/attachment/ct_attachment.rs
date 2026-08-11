//! 单个附件。

use std::fmt::Write;

/// 对应 Java: org.ofdrw.core.attachment.CT_Attachment
///
/// 表示一个单独的附件对象，包含附件的标识、名称、格式、
/// 创建日期、大小、可见性和文件数据。
#[derive(Debug, Clone)]
pub struct CTAttachment {
    /// 附件 ID。
    pub id: String,
    /// 附件名称。
    pub name: String,
    /// 附件格式（MIME 类型或文件扩展名）。
    pub format: Option<String>,
    /// 创建日期（ISO 8601 格式字符串）。
    pub creation_date: Option<String>,
    /// 附件大小（字节）。
    pub size: Option<u64>,
    /// 附件是否可见。
    pub visible: bool,
    /// 附件文件路径（OFD 包内相对路径）。
    pub file: Option<String>,
}

impl CTAttachment {
    /// 创建一个新的附件。
    #[must_use]
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            format: None,
            creation_date: None,
            size: None,
            visible: true,
            file: None,
        }
    }

    /// 设置格式。
    #[must_use]
    pub fn format(mut self, format: impl Into<String>) -> Self {
        self.format = Some(format.into());
        self
    }

    /// 设置创建日期。
    #[must_use]
    pub fn creation_date(mut self, date: impl Into<String>) -> Self {
        self.creation_date = Some(date.into());
        self
    }

    /// 设置大小（字节）。
    #[must_use]
    pub fn size(mut self, size: u64) -> Self {
        self.size = Some(size);
        self
    }

    /// 设置是否可见。
    #[must_use]
    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    /// 设置附件文件路径。
    #[must_use]
    pub fn file(mut self, file: impl Into<String>) -> Self {
        self.file = Some(file.into());
        self
    }

    /// 序列化为 XML 字符串。
    #[must_use]
    pub fn to_xml_string(&self) -> String {
        let mut xml = format!(r#"<ofd:Attachment ID="{}" Name="{}""#, self.id, self.name);

        if let Some(ref fmt) = self.format {
            let _ = write!(xml, r#" Format="{fmt}""#);
        }
        if let Some(ref date) = self.creation_date {
            let _ = write!(xml, r#" CreationDate="{date}""#);
        }
        if let Some(sz) = self.size {
            let _ = write!(xml, r#" Size="{sz}""#);
        }
        if !self.visible {
            xml.push_str(r#" Visible="false""#);
        }
        if let Some(ref file) = self.file {
            let _ = write!(xml, r#" File="{file}""#);
        }

        xml.push_str(" />");
        xml
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ct_attachment_new() {
        let a = CTAttachment::new("att1", "readme.pdf");
        assert_eq!(a.id, "att1");
        assert_eq!(a.name, "readme.pdf");
        assert!(a.format.is_none());
        assert!(a.creation_date.is_none());
        assert!(a.size.is_none());
        assert!(a.visible);
        assert!(a.file.is_none());
    }

    #[test]
    fn test_ct_attachment_builder() {
        let a = CTAttachment::new("att2", "data.xlsx")
            .format("application/vnd.ms-excel")
            .creation_date("2025-03-15")
            .size(1024)
            .visible(false)
            .file("Attachments/data.xlsx");
        assert_eq!(a.format.as_deref(), Some("application/vnd.ms-excel"));
        assert_eq!(a.creation_date.as_deref(), Some("2025-03-15"));
        assert_eq!(a.size, Some(1024));
        assert!(!a.visible);
        assert_eq!(a.file.as_deref(), Some("Attachments/data.xlsx"));
    }

    #[test]
    fn test_ct_attachment_to_xml_string_basic() {
        let a = CTAttachment::new("a1", "test.txt");
        let xml = a.to_xml_string();
        assert!(xml.contains(r#"ID="a1""#));
        assert!(xml.contains(r#"Name="test.txt""#));
        assert!(xml.contains(" />"));
    }

    #[test]
    fn test_ct_attachment_to_xml_string_full() {
        let a = CTAttachment::new("a2", "img.png")
            .format("image/png")
            .creation_date("2025-06-01")
            .size(2048)
            .visible(false)
            .file("Attachments/img.png");
        let xml = a.to_xml_string();
        assert!(xml.contains(r#"Format="image/png""#));
        assert!(xml.contains(r#"CreationDate="2025-06-01""#));
        assert!(xml.contains(r#"Size="2048""#));
        assert!(xml.contains(r#"Visible="false""#));
        assert!(xml.contains(r#"File="Attachments/img.png""#));
    }

    #[test]
    fn test_ct_attachment_visible_default_true() {
        let a = CTAttachment::new("a1", "x.txt");
        let xml = a.to_xml_string();
        assert!(!xml.contains("Visible"));
    }

    #[test]
    fn test_ct_attachment_clone_debug() {
        let a = CTAttachment::new("x", "y.txt");
        let a2 = a.clone();
        assert_eq!(a2.id, "x");
        assert!(format!("{a:?}").contains("CTAttachment"));
    }
}
