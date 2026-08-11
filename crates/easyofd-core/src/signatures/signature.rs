//! 数字签名注册信息（Signature）。
//!
//! 对应 Java: org.ofdrw.core.signatures.Signature
//!
//! GB/T 33190 第 18.1 节 图 85 表 66。

use std::fmt::Write;

/// 签名类型枚举。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SigType {
    /// 签章（默认值）。
    Seal,
    /// 签名。
    Sign,
}

impl SigType {
    /// 获取枚举的字符串表示。
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Seal => "Seal",
            Self::Sign => "Sign",
        }
    }

    /// 从字符串解析。
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "Seal" => Ok(Self::Seal),
            "Sign" => Ok(Self::Sign),
            _ => Err(format!("未知的签名类型: {s}")),
        }
    }
}

impl std::fmt::Display for SigType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// 数字签名或安全签章在列表中的注册信息。
///
/// 每个签名或签章对应一个节点。
/// 推荐使用 `sNNN` 的编码方式，NNN 从 1 开始。
#[derive(Debug, Clone)]
pub struct Signature {
    /// 签名或签章的标识（必选）。
    pub id: String,
    /// 签名节点类型（可选），默认 Seal。
    pub sig_type: Option<SigType>,
    /// 基于的签名 ID（可选，OFD 2.0）。
    /// 验证时应同时验证"基"签名。
    pub relative: Option<String>,
    /// 指向包内的签名描述文件路径（必选）。
    pub base_loc: String,
}

impl Signature {
    /// 创建新的签名注册信息。
    #[must_use]
    pub fn new(id: impl Into<String>, base_loc: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            sig_type: None,
            relative: None,
            base_loc: base_loc.into(),
        }
    }

    /// 设置签名类型。
    #[must_use]
    pub fn sig_type(mut self, sig_type: SigType) -> Self {
        self.sig_type = Some(sig_type);
        self
    }

    /// 设置基于的签名 ID（OFD 2.0）。
    #[must_use]
    pub fn relative(mut self, id: impl Into<String>) -> Self {
        self.relative = Some(id.into());
        self
    }

    /// 序列化为 XML 字符串。
    #[must_use]
    pub fn to_xml_string(&self) -> String {
        let mut xml = format!(r#"<ofd:Signature ID="{}""#, self.id);
        if let Some(ref sig_type) = self.sig_type {
            let _ = write!(xml, r#" Type="{}""#, sig_type.as_str());
        }
        if let Some(ref rel) = self.relative {
            let _ = write!(xml, r#" Relative="{rel}""#);
        }
        let _ = write!(xml, r#" BaseLoc="{}""#, self.base_loc);
        xml.push_str(" />");
        xml
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signature_new() {
        let sig = Signature::new("s1", "/Doc_0/Signs/Sign_0/Signature.xml");
        assert_eq!(sig.id, "s1");
        assert_eq!(sig.base_loc, "/Doc_0/Signs/Sign_0/Signature.xml");
        assert!(sig.sig_type.is_none());
        assert!(sig.relative.is_none());
    }

    #[test]
    fn test_signature_builder() {
        let sig = Signature::new("s2", "/Doc_0/Signs/Sign_1/Signature.xml")
            .sig_type(SigType::Sign)
            .relative("s1");
        assert_eq!(sig.sig_type, Some(SigType::Sign));
        assert_eq!(sig.relative.as_deref(), Some("s1"));
    }

    #[test]
    fn test_signature_xml_full() {
        let sig = Signature::new("s1", "/Doc_0/Signs/Sign_0/Signature.xml").sig_type(SigType::Seal);
        let xml = sig.to_xml_string();
        assert!(xml.contains(r#"ID="s1""#));
        assert!(xml.contains(r#"Type="Seal""#));
        assert!(xml.contains(r#"BaseLoc="/Doc_0/Signs/Sign_0/Signature.xml""#));
        assert!(!xml.contains("Relative"));
    }

    #[test]
    fn test_sig_type_display() {
        assert_eq!(SigType::Seal.to_string(), "Seal");
        assert_eq!(SigType::Sign.to_string(), "Sign");
        assert_eq!(SigType::from_str("Seal").unwrap(), SigType::Seal);
        assert!(SigType::from_str("Unknown").is_err());
    }
}
