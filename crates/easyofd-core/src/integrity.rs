//! OFD 条目列表。
//!
//! 对应 Java: org.ofdrw.core.integrity.OFDEntries

/// OFD 条目列表。
///
/// 用于 OFD 文件完整性保护，记录所有文件的摘要信息。
///
/// 对应 Java: org.ofdrw.core.integrity.OFDEntries
#[derive(Debug, Clone, Default)]
pub struct OFDEntries {
    /// 条目列表（路径, 摘要值）。
    pub entries: Vec<(String, String)>,
}

impl OFDEntries {
    /// 创建空条目列表。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 添加条目。
    pub fn add(&mut self, path: impl Into<String>, digest: impl Into<String>) {
        self.entries.push((path.into(), digest.into()));
    }

    /// 获取条目数量。
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// 是否为空。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// 查找指定路径的摘要。
    #[must_use]
    pub fn find(&self, path: &str) -> Option<&str> {
        self.entries
            .iter()
            .find(|(p, _)| p == path)
            .map(|(_, d)| d.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ofd_entries_new() {
        let e = OFDEntries::new();
        assert!(e.is_empty());
    }

    #[test]
    fn ofd_entries_add() {
        let mut e = OFDEntries::new();
        e.add("OFD.xml", "abc123");
        e.add("Doc_0/Document.xml", "def456");
        assert_eq!(e.len(), 2);
    }

    #[test]
    fn ofd_entries_find() {
        let mut e = OFDEntries::new();
        e.add("OFD.xml", "hash1");
        assert_eq!(e.find("OFD.xml"), Some("hash1"));
        assert_eq!(e.find("missing"), None);
    }
}
