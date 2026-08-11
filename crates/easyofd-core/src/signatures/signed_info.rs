//! 签名信息（SignedInfo）。
//!
//! 对应 Java: org.ofdrw.core.signatures.sig.SignedInfo
//!
//! 签名要保护的原文及本次签名相关的信息。
//! GB/T 33190 第 18.2.1 节 图 86 表 67。

use super::references::References;
use super::seal::Seal;
use super::stamp_annot::StampAnnot;

/// 签章组件提供者信息。
#[derive(Debug, Clone)]
pub struct Provider {
    /// 提供者名称。
    pub name: String,
    /// 提供者版本。
    pub version: Option<String>,
    /// 提供者公司。
    pub company: Option<String>,
}

impl Provider {
    /// 创建新的提供者信息。
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: None,
            company: None,
        }
    }

    /// 设置版本。
    #[must_use]
    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    /// 设置公司。
    #[must_use]
    pub fn company(mut self, company: impl Into<String>) -> Self {
        self.company = Some(company.into());
        self
    }

    /// 序列化为 XML 字符串。
    #[must_use]
    pub fn to_xml_string(&self) -> String {
        let mut xml = format!(r#"<ofd:Provider Name="{}""#, self.name);
        if let Some(ref v) = self.version {
            use std::fmt::Write;
            let _ = write!(xml, r#" Version="{v}""#);
        }
        if let Some(ref c) = self.company {
            use std::fmt::Write;
            let _ = write!(xml, r#" Company="{c}""#);
        }
        xml.push_str(" />");
        xml
    }
}

/// 签名要保护的原文及本次签名相关的信息。
///
/// 包含签章组件提供者、签名方法、签名时间、
/// 保护文件列表、签名外观和印章信息。
#[derive(Debug, Clone)]
pub struct SignedInfo {
    /// 创建签名时所用的签章组件提供者信息（必选）。
    pub provider: Provider,
    /// 签名方法（可选），记录安全模块返回的签名算法代码。
    pub signature_method: Option<String>,
    /// 签名时间（可选），记录安全模块返回的签名时间。
    pub signature_datetime: Option<String>,
    /// 包内文件计算所得的摘要记录列表（必选）。
    pub references: References,
    /// 本签名关联的外观列表（可选），可出现多次。
    pub stamp_annots: Vec<StampAnnot>,
    /// 电子印章信息（可选）。
    pub seal: Option<Seal>,
}

impl SignedInfo {
    /// 创建新的签名信息。
    #[must_use]
    pub fn new(provider: Provider, references: References) -> Self {
        Self {
            provider,
            signature_method: None,
            signature_datetime: None,
            references,
            stamp_annots: Vec::new(),
            seal: None,
        }
    }

    /// 设置签名方法。
    #[must_use]
    pub fn signature_method(mut self, method: impl Into<String>) -> Self {
        self.signature_method = Some(method.into());
        self
    }

    /// 设置签名时间。
    #[must_use]
    pub fn signature_datetime(mut self, datetime: impl Into<String>) -> Self {
        self.signature_datetime = Some(datetime.into());
        self
    }

    /// 添加签名外观。
    #[must_use]
    pub fn add_stamp_annot(mut self, stamp_annot: StampAnnot) -> Self {
        self.stamp_annots.push(stamp_annot);
        self
    }

    /// 设置电子印章信息。
    #[must_use]
    pub fn seal(mut self, seal: Seal) -> Self {
        self.seal = Some(seal);
        self
    }

    /// 序列化为 XML 字符串。
    #[must_use]
    pub fn to_xml_string(&self) -> String {
        use std::fmt::Write;

        let mut xml = String::from("<ofd:SignedInfo>");
        xml.push('\n');
        xml.push_str(&self.provider.to_xml_string());

        if let Some(ref method) = self.signature_method {
            xml.push('\n');
            let _ = write!(xml, "<ofd:SignatureMethod>{method}</ofd:SignatureMethod>");
        }
        if let Some(ref dt) = self.signature_datetime {
            xml.push('\n');
            let _ = write!(xml, "<ofd:SignatureDateTime>{dt}</ofd:SignatureDateTime>");
        }

        xml.push('\n');
        xml.push_str(&self.references.to_xml_string());

        for sa in &self.stamp_annots {
            xml.push('\n');
            xml.push_str(&sa.to_xml_string());
        }

        if let Some(ref seal) = self.seal {
            xml.push('\n');
            xml.push_str(&seal.to_xml_string());
        }

        xml.push_str("\n</ofd:SignedInfo>");
        xml
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::basic_type::{ST_Box, ST_RefID};

    #[test]
    fn test_signed_info_new() {
        let si = SignedInfo::new(
            Provider::new("TestProvider"),
            References::new().add_reference(crate::signatures::reference::Reference::new(
                "/Doc_0/Document.xml",
                "hash",
            )),
        );
        assert_eq!(si.provider.name, "TestProvider");
        assert!(si.signature_method.is_none());
        assert!(si.seal.is_none());
        assert!(si.stamp_annots.is_empty());
    }

    #[test]
    fn test_signed_info_builder() {
        let si = SignedInfo::new(
            Provider::new("MyProvider").version("1.0").company("MyCo"),
            References::new().check_method("SM3"),
        )
        .signature_method("1.2.156.10197.1.501")
        .signature_datetime("2025-01-01T00:00:00")
        .add_stamp_annot(StampAnnot::new(
            "s1",
            ST_RefID::new(1),
            ST_Box::new(0.0, 0.0, 100.0, 100.0),
        ))
        .seal(Seal::new("/Doc_0/Signs/Sign_0/Seal.esl"));
        assert_eq!(si.signature_method.as_deref(), Some("1.2.156.10197.1.501"));
        assert_eq!(
            si.signature_datetime.as_deref(),
            Some("2025-01-01T00:00:00")
        );
        assert_eq!(si.stamp_annots.len(), 1);
        assert!(si.seal.is_some());
    }

    #[test]
    fn test_signed_info_xml() {
        let si = SignedInfo::new(
            Provider::new("Prov"),
            References::new().add_reference(crate::signatures::reference::Reference::new(
                "/Doc_0/Document.xml",
                "abc",
            )),
        )
        .signature_method("SM2");
        let xml = si.to_xml_string();
        assert!(xml.contains("<ofd:SignedInfo>"));
        assert!(xml.contains("Name=\"Prov\""));
        assert!(xml.contains("<ofd:SignatureMethod>SM2</ofd:SignatureMethod>"));
        assert!(xml.contains("</ofd:SignedInfo>"));
    }

    #[test]
    fn test_provider_builder() {
        let p = Provider::new("Test").version("2.0").company("ACME");
        assert_eq!(p.name, "Test");
        assert_eq!(p.version.as_deref(), Some("2.0"));
        assert_eq!(p.company.as_deref(), Some("ACME"));
        let xml = p.to_xml_string();
        assert!(xml.contains("Name=\"Test\""));
        assert!(xml.contains("Version=\"2.0\""));
        assert!(xml.contains("Company=\"ACME\""));
    }
}
