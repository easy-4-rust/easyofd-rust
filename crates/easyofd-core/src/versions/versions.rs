//! 版本列表入口。

use super::Version;

/// 对应 Java: org.ofdrw.core.basicStructure.Versions
///
/// OFD 文档的版本列表容器，包含文档所有历史版本信息。
#[derive(Debug, Clone)]
pub struct Versions {
    /// 版本列表。
    pub versions: Vec<Version>,
}

impl Versions {
    /// 创建空的版本列表。
    #[must_use]
    pub fn new() -> Self {
        Self {
            versions: Vec::new(),
        }
    }

    /// 添加一个版本。
    pub fn push(&mut self, version: Version) {
        self.versions.push(version);
    }

    /// 序列化为 XML 字符串。
    #[must_use]
    pub fn to_xml_string(&self) -> String {
        let inner: String = self.versions.iter().map(Version::to_xml_string).collect();
        format!("<Versions>{inner}</Versions>")
    }
}

impl Default for Versions {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_versions_new() {
        let v = Versions::new();
        assert!(v.versions.is_empty());
        let v2 = Versions::default();
        assert!(v2.versions.is_empty());
    }

    #[test]
    fn test_versions_push_and_xml() {
        let mut v = Versions::new();
        v.push(Version::new("1.0"));
        v.push(Version::new("2.0"));
        assert_eq!(v.versions.len(), 2);
        let xml = v.to_xml_string();
        assert!(xml.contains("<Versions>"));
        assert!(xml.contains("</Versions>"));
        assert!(xml.contains("1.0"));
        assert!(xml.contains("2.0"));
    }
}
