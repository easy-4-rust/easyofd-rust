//! 裸 OFD 文档。
//!
//! 对应 Java: org.ofdrw.tool.merge.BareOFDDoc
//!
//! 表示一个待合并的 OFD 文档的最小信息。

/// 裸 OFD 文档。
///
/// 对应 Java: `org.ofdrw.tool.merge.BareOFDDoc`
///
/// 仅包含文档路径和页面数量的轻量描述，
/// 用于合并前的预检和页面统计。
#[derive(Debug, Clone)]
pub struct BareOFDDoc {
    /// 文档路径。
    path: String,
    /// 页面数量。
    page_count: usize,
}

impl BareOFDDoc {
    /// 创建裸 OFD 文档描述。
    #[must_use]
    pub fn new(path: impl Into<String>, page_count: usize) -> Self {
        Self {
            path: path.into(),
            page_count,
        }
    }

    /// 获取文档路径。
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    /// 获取页面数量。
    #[must_use]
    pub fn page_count(&self) -> usize {
        self.page_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_doc() {
        let doc = BareOFDDoc::new("/tmp/test.ofd", 5);
        assert_eq!(doc.path(), "/tmp/test.ofd");
        assert_eq!(doc.page_count(), 5);
    }

    #[test]
    fn clone_doc() {
        let doc = BareOFDDoc::new("/tmp/a.ofd", 3);
        let cloned = doc.clone();
        assert_eq!(cloned.path(), "/tmp/a.ofd");
        assert_eq!(cloned.page_count(), 3);
    }
}
