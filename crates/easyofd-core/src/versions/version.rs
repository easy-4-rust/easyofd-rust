//! 单个版本描述。

use super::DocVersion;

/// 对应 Java: org.ofdrw.core.basicStructure.Version
///
/// 描述 OFD 文档的一个版本，包含版本号和该版本的文档内容。
#[derive(Debug, Clone)]
pub struct Version {
    /// 版本号（如 "1.0"、"2.0"）。
    pub version: String,
    /// 该版本的文档版本列表。
    pub doc_versions: Vec<DocVersion>,
}

impl Version {
    /// 创建指定版本号的版本。
    #[must_use]
    pub fn new(version: impl Into<String>) -> Self {
        Self {
            version: version.into(),
            doc_versions: Vec::new(),
        }
    }

    /// 添加文档版本。
    pub fn push_doc_version(&mut self, doc_version: DocVersion) {
        self.doc_versions.push(doc_version);
    }

    /// 序列化为 XML 字符串。
    #[must_use]
    pub fn to_xml_string(&self) -> String {
        let inner: String = self
            .doc_versions
            .iter()
            .map(DocVersion::to_xml_string)
            .collect();
        format!("<Version Version=\"{}\">{inner}</Version>", self.version)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_new() {
        let v = Version::new("1.0");
        assert_eq!(v.version, "1.0");
        assert!(v.doc_versions.is_empty());
    }

    #[test]
    fn test_version_xml() {
        let mut v = Version::new("2.0");
        v.push_doc_version(DocVersion::new("doc1"));
        let xml = v.to_xml_string();
        assert!(xml.contains("Version=\"2.0\""));
        assert!(xml.contains("doc1"));
    }
}
