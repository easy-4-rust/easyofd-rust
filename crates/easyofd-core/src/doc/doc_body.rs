//! 文档体。
//!
//! 对应 Java: org.ofdrw.core.basicStructure.ofd.DocBody
//! OFD.xml 中的 DocBody 元素

use crate::basic_type::ST_Loc;

/// 文档体。
///
/// OFD.xml 中的 DocBody 元素，包含文档根节点路径和文档信息。
///
/// 对应 Java: org.ofdrw.core.basicStructure.ofd.DocBody
#[derive(Debug, Clone)]
pub struct DocBody {
    /// 文档根节点路径（Doc_0/Document.xml）。
    pub doc_root: ST_Loc,
    /// 文档信息（可选）。
    pub doc_info: Option<String>,
    /// 签名列表路径（可选）。
    pub signatures: Option<ST_Loc>,
}

impl DocBody {
    /// 创建文档体。
    #[must_use]
    pub fn new(doc_root: ST_Loc) -> Self {
        Self {
            doc_root,
            doc_info: None,
            signatures: None,
        }
    }

    /// 设置文档信息。
    #[must_use]
    pub fn doc_info(mut self, info: impl Into<String>) -> Self {
        self.doc_info = Some(info.into());
        self
    }

    /// 设置签名列表路径。
    #[must_use]
    pub fn signatures(mut self, path: ST_Loc) -> Self {
        self.signatures = Some(path);
        self
    }

    /// 获取文档根节点路径。
    #[must_use]
    pub fn doc_root(&self) -> &ST_Loc {
        &self.doc_root
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn doc_body_new() {
        let db = DocBody::new(ST_Loc::new("Doc_0/Document.xml"));
        assert_eq!(db.doc_root.loc(), "Doc_0/Document.xml");
        assert!(db.doc_info.is_none());
        assert!(db.signatures.is_none());
    }

    #[test]
    fn doc_body_builder() {
        let db = DocBody::new(ST_Loc::new("Doc_0/Document.xml"))
            .doc_info("test info")
            .signatures(ST_Loc::new("Doc_0/Signs/Signatures.xml"));
        assert!(db.doc_info.is_some());
        assert!(db.signatures.is_some());
    }

    #[test]
    fn doc_body_clone_debug() {
        let db = DocBody::new(ST_Loc::new("Doc_0/Document.xml"));
        let db2 = db.clone();
        assert_eq!(db2.doc_root.loc(), "Doc_0/Document.xml");
        assert!(format!("{db:?}").contains("DocBody"));
    }
}
