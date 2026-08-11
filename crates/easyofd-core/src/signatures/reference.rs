//! 针对一个文件的摘要节点（Reference）。
//!
//! 对应 Java: org.ofdrw.core.signatures.range.Reference
//!
//! GB/T 33190 第 18.2.2 节 图 87 表 68。

use std::fmt::Write;

/// 针对一个文件的摘要节点。
///
/// 描述签名保护范围内某个文件的摘要计算值。
#[derive(Debug, Clone)]
pub struct Reference {
    /// 指向包内文件的绝对路径（必选）。
    pub file_ref: String,
    /// 对包内文件进行摘要计算的杂凑值，Base64 编码（必选）。
    pub check_value: String,
}

impl Reference {
    /// 创建新的文件摘要节点。
    #[must_use]
    pub fn new(file_ref: impl Into<String>, check_value: impl Into<String>) -> Self {
        Self {
            file_ref: file_ref.into(),
            check_value: check_value.into(),
        }
    }

    /// 序列化为 XML 字符串。
    #[must_use]
    pub fn to_xml_string(&self) -> String {
        let mut xml = String::from(r#"<ofd:Reference FileRef=""#);
        let _ = write!(xml, "{}", self.file_ref);
        xml.push_str(r#"">"#);
        let _ = write!(xml, "<ofd:CheckValue>{}</ofd:CheckValue>", self.check_value);
        xml.push_str("</ofd:Reference>");
        xml
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reference_new() {
        let r = Reference::new("/Doc_0/Document.xml", "abc123base64");
        assert_eq!(r.file_ref, "/Doc_0/Document.xml");
        assert_eq!(r.check_value, "abc123base64");
    }

    #[test]
    fn test_reference_xml() {
        let r = Reference::new("/Doc_0/Pages/Page_0/Content.xml", "dGVzdA==");
        let xml = r.to_xml_string();
        assert!(xml.contains(r#"FileRef="/Doc_0/Pages/Page_0/Content.xml""#));
        assert!(xml.contains("<ofd:CheckValue>dGVzdA==</ofd:CheckValue>"));
        assert!(xml.contains("</ofd:Reference>"));
    }

    #[test]
    fn test_reference_clone() {
        let r = Reference::new("/Res/font.ttf", "hash123");
        let r2 = r.clone();
        assert_eq!(r2.file_ref, "/Res/font.ttf");
        assert_eq!(r2.check_value, "hash123");
    }
}
