//! 文档版本。

use super::FileList;

/// 对应 Java: org.ofdrw.core.basicStructure.DocVersion
///
/// 描述文档的一个版本，包含文档根路径和文件列表。
#[derive(Debug, Clone)]
pub struct DocVersion {
    /// 文档根路径（如 "Doc_0"）。
    pub doc_root: String,
    /// 该版本的文件列表。
    pub file_list: Option<FileList>,
}

impl DocVersion {
    /// 创建指定文档根路径的文档版本。
    #[must_use]
    pub fn new(doc_root: impl Into<String>) -> Self {
        Self {
            doc_root: doc_root.into(),
            file_list: None,
        }
    }

    /// 设置文件列表。
    #[must_use]
    pub fn with_file_list(mut self, file_list: FileList) -> Self {
        self.file_list = Some(file_list);
        self
    }

    /// 序列化为 XML 字符串。
    #[must_use]
    pub fn to_xml_string(&self) -> String {
        let inner = match &self.file_list {
            Some(fl) => fl.to_xml_string(),
            None => String::new(),
        };
        format!(
            "<DocVersion DocRoot=\"{}\">{inner}</DocVersion>",
            self.doc_root
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::versions::File;

    #[test]
    fn test_doc_version_new() {
        let dv = DocVersion::new("Doc_0");
        assert_eq!(dv.doc_root, "Doc_0");
        assert!(dv.file_list.is_none());
    }

    #[test]
    fn test_doc_version_with_file_list_and_xml() {
        let fl = FileList::new().with(File::new("OFD.xml", 1024));
        let dv = DocVersion::new("Doc_0").with_file_list(fl);
        let xml = dv.to_xml_string();
        assert!(xml.contains("DocRoot=\"Doc_0\""));
        assert!(xml.contains("OFD.xml"));
        assert!(xml.contains("1024"));
    }
}
