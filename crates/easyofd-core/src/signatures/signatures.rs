//! 签名列表根节点（Signatures）。
//!
//! 对应 Java: org.ofdrw.core.signatures.Signatures
//!
//! 签名列表文件的入口点，可包含多个签名（例如联合发文等情况）。
//! 当允许下次继续添加签名时，该文件不会被包含到本次签名的
//! 保护文件列表（References）中。
//! GB/T 33190 第 18.1 节 图 85 表 66。

use super::signature::Signature;

/// 签名列表根节点。
///
/// 包含文档中所有数字签名和安全签章的注册信息。
#[derive(Debug, Clone)]
pub struct Signatures {
    /// 安全标识的最大值（可选）。
    /// 作用与文档入口文件 Document.xml 中的 MaxID 相同，
    /// 推荐使用 `sNNN` 的编码方式，NNN 从 1 开始。
    pub max_sign_id: Option<String>,
    /// 数字签名或安全签章在列表中的注册信息序列。
    pub signatures: Vec<Signature>,
}

impl Signatures {
    /// 创建空的签名列表。
    #[must_use]
    pub fn new() -> Self {
        Self {
            max_sign_id: None,
            signatures: Vec::new(),
        }
    }

    /// 设置安全标识的最大值。
    #[must_use]
    pub fn max_sign_id(mut self, max_sign_id: impl Into<String>) -> Self {
        self.max_sign_id = Some(max_sign_id.into());
        self
    }

    /// 添加签名注册信息。
    #[must_use]
    pub fn add_signature(mut self, signature: Signature) -> Self {
        self.signatures.push(signature);
        self
    }

    /// 序列化为 XML 字符串。
    #[must_use]
    pub fn to_xml_string(&self) -> String {
        use std::fmt::Write;

        let mut xml = String::from(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
        xml.push('\n');
        xml.push_str(r#"<ofd:Signatures xmlns:ofd="http://www.ofdspec.org/2016">"#);

        if let Some(ref max_id) = self.max_sign_id {
            xml.push('\n');
            let _ = write!(xml, "<ofd:MaxSignId>{max_id}</ofd:MaxSignId>");
        }

        for sig in &self.signatures {
            xml.push('\n');
            xml.push_str(&sig.to_xml_string());
        }

        xml.push_str("\n</ofd:Signatures>");
        xml
    }
}

impl Default for Signatures {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signatures_new() {
        let sigs = Signatures::new();
        assert!(sigs.max_sign_id.is_none());
        assert!(sigs.signatures.is_empty());
    }

    #[test]
    fn test_signatures_builder() {
        let sigs = Signatures::new()
            .max_sign_id("s2")
            .add_signature(Signature::new("s1", "/Doc_0/Signs/Sign_0/Signature.xml"))
            .add_signature(Signature::new("s2", "/Doc_0/Signs/Sign_1/Signature.xml"));
        assert_eq!(sigs.max_sign_id.as_deref(), Some("s2"));
        assert_eq!(sigs.signatures.len(), 2);
    }

    #[test]
    fn test_signatures_xml() {
        let sigs = Signatures::new()
            .max_sign_id("s1")
            .add_signature(Signature::new("s1", "/Doc_0/Signs/Sign_0/Signature.xml"));
        let xml = sigs.to_xml_string();
        assert!(xml.contains("<?xml version=\"1.0\""));
        assert!(xml.contains("ofd:Signatures"));
        assert!(xml.contains("<ofd:MaxSignId>s1</ofd:MaxSignId>"));
        assert!(xml.contains(r#"ID="s1""#));
        assert!(xml.contains("</ofd:Signatures>"));
    }

    #[test]
    fn test_signatures_default() {
        let sigs = Signatures::default();
        assert!(sigs.signatures.is_empty());
    }
}
