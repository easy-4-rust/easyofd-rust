//! 单个文件信息。

/// 对应 Java: org.ofdrw.core.basicStructure.File
///
/// 描述 OFD 包中的一个文件，包含文件名和大小。
#[derive(Debug, Clone)]
pub struct File {
    /// 文件名（如 "OFD.xml"、"Doc_0/Content.xml"）。
    pub name: String,
    /// 文件大小（字节）。
    pub size: u64,
}

impl File {
    /// 创建新的文件信息。
    #[must_use]
    pub fn new(name: impl Into<String>, size: u64) -> Self {
        Self {
            name: name.into(),
            size,
        }
    }

    /// 序列化为 XML 字符串。
    #[must_use]
    pub fn to_xml_string(&self) -> String {
        format!("<File Name=\"{}\" Size=\"{}\"/>", self.name, self.size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_new() {
        let f = File::new("OFD.xml", 1024);
        assert_eq!(f.name, "OFD.xml");
        assert_eq!(f.size, 1024);
    }

    #[test]
    fn test_file_xml() {
        let f = File::new("Doc_0/Content.xml", 4096);
        let xml = f.to_xml_string();
        assert!(xml.contains("Name=\"Doc_0/Content.xml\""));
        assert!(xml.contains("Size=\"4096\""));
        assert!(xml.contains("/>"));
    }
}
