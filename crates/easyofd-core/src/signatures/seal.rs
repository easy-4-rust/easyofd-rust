//! 电子印章信息（Seal）。
//!
//! 对应 Java: org.ofdrw.core.signatures.appearance.Seal
//!
//! GB/T 33190 第 18.2.1 节 图 86 表 67。

/// 电子印章信息。
///
/// 指向包内的安全电子印章文件，遵循密码领域的相关规范。
#[derive(Debug, Clone)]
pub struct Seal {
    /// 指向包内的安全电子印章文件路径（必选）。
    pub base_loc: String,
    /// 印模图片存储位置（可选，OFD 2.0）。
    pub image_loc: Option<String>,
}

impl Seal {
    /// 创建新的印章引用。
    #[must_use]
    pub fn new(base_loc: impl Into<String>) -> Self {
        Self {
            base_loc: base_loc.into(),
            image_loc: None,
        }
    }

    /// 设置印模图片存储位置（OFD 2.0）。
    #[must_use]
    pub fn image_loc(mut self, image_loc: impl Into<String>) -> Self {
        self.image_loc = Some(image_loc.into());
        self
    }

    /// 序列化为 XML 字符串。
    #[must_use]
    pub fn to_xml_string(&self) -> String {
        use std::fmt::Write;
        let mut xml = String::from("<ofd:Seal>");
        let _ = write!(xml, "<ofd:BaseLoc>{}</ofd:BaseLoc>", self.base_loc);
        if let Some(ref loc) = self.image_loc {
            let _ = write!(xml, "<ofd:ImageLoc>{loc}</ofd:ImageLoc>");
        }
        xml.push_str("</ofd:Seal>");
        xml
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seal_new() {
        let s = Seal::new("/Doc_0/Signs/Sign_1/Seal.esl");
        assert_eq!(s.base_loc, "/Doc_0/Signs/Sign_1/Seal.esl");
        assert!(s.image_loc.is_none());
    }

    #[test]
    fn test_seal_with_image_loc() {
        let s = Seal::new("/Doc_0/Signs/Sign_1/Seal.esl").image_loc("/Doc_0/Signs/Sign_1/seal.png");
        assert!(s.image_loc.is_some());
        assert_eq!(s.image_loc.unwrap(), "/Doc_0/Signs/Sign_1/seal.png");
    }

    #[test]
    fn test_seal_xml_basic() {
        let s = Seal::new("/Doc_0/Signs/Sign_1/Seal.esl");
        let xml = s.to_xml_string();
        assert!(xml.contains("<ofd:Seal>"));
        assert!(xml.contains("<ofd:BaseLoc>/Doc_0/Signs/Sign_1/Seal.esl</ofd:BaseLoc>"));
        assert!(!xml.contains("ImageLoc"));
        assert!(xml.contains("</ofd:Seal>"));
    }

    #[test]
    fn test_seal_xml_with_image() {
        let s = Seal::new("/Doc_0/Signs/Sign_1/Seal.esl").image_loc("/Doc_0/Signs/Sign_1/seal.png");
        let xml = s.to_xml_string();
        assert!(xml.contains("<ofd:ImageLoc>/Doc_0/Signs/Sign_1/seal.png</ofd:ImageLoc>"));
    }
}
