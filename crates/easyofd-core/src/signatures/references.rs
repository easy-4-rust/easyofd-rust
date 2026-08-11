//! 签名的范围（References）。
//!
//! 对应 Java: org.ofdrw.core.signatures.range.References
//!
//! GB/T 33190 第 18.2.2 节 图 87 表 68。

use super::reference::Reference;

/// 签名的范围。
///
/// 包含受本次签名保护的文件摘要记录列表。
/// 一个受保护的包内文件对应一个 [`Reference`] 节点。
#[derive(Debug, Clone)]
pub struct References {
    /// 摘要方法（可选）。
    /// 视应用场景的不同使用不同的摘要方法。
    pub check_method: Option<String>,
    /// 针对各文件的摘要节点列表。
    pub references: Vec<Reference>,
}

impl References {
    /// 创建空的签名范围。
    #[must_use]
    pub fn new() -> Self {
        Self {
            check_method: None,
            references: Vec::new(),
        }
    }

    /// 设置摘要方法。
    #[must_use]
    pub fn check_method(mut self, method: impl Into<String>) -> Self {
        self.check_method = Some(method.into());
        self
    }

    /// 添加一个文件摘要节点。
    #[must_use]
    pub fn add_reference(mut self, reference: Reference) -> Self {
        self.references.push(reference);
        self
    }

    /// 检查是否包含指定路径的文件。
    pub fn has_file(&self, abs_loc: &str) -> bool {
        self.references.iter().any(|r| r.file_ref == abs_loc)
    }

    /// 序列化为 XML 字符串。
    #[must_use]
    pub fn to_xml_string(&self) -> String {
        let mut xml = String::from("<ofd:References");
        if let Some(ref method) = self.check_method {
            use std::fmt::Write;
            let _ = write!(xml, r#" CheckMethod="{method}""#);
        }
        xml.push('>');
        for r in &self.references {
            xml.push('\n');
            xml.push_str(&r.to_xml_string());
        }
        xml.push_str("\n</ofd:References>");
        xml
    }
}

impl Default for References {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_references_new() {
        let refs = References::new();
        assert!(refs.check_method.is_none());
        assert!(refs.references.is_empty());
    }

    #[test]
    fn test_references_builder() {
        let refs = References::new()
            .check_method("SM3")
            .add_reference(Reference::new("/Doc_0/Document.xml", "abc"))
            .add_reference(Reference::new("/Doc_0/Pages/Page_0/Content.xml", "def"));
        assert_eq!(refs.check_method.as_deref(), Some("SM3"));
        assert_eq!(refs.references.len(), 2);
    }

    #[test]
    fn test_references_has_file() {
        let refs = References::new().add_reference(Reference::new("/Doc_0/Document.xml", "abc"));
        assert!(refs.has_file("/Doc_0/Document.xml"));
        assert!(!refs.has_file("/Doc_0/Other.xml"));
    }

    #[test]
    fn test_references_xml() {
        let refs = References::new()
            .check_method("SHA256")
            .add_reference(Reference::new("/Doc_0/Document.xml", "hash1"));
        let xml = refs.to_xml_string();
        assert!(xml.contains("CheckMethod=\"SHA256\""));
        assert!(xml.contains("FileRef=\"/Doc_0/Document.xml\""));
        assert!(xml.contains("</ofd:References>"));
    }

    #[test]
    fn test_references_default() {
        let refs = References::default();
        assert!(refs.references.is_empty());
    }
}
