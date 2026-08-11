//! 文档目录（DocDir）。
//!
//! 对应 Java: org.ofdrw.pkg.container.DocDir
//!
//! 文档容器，对应 OFD 包中的 Doc_N/ 目录。
//! 包含 Document.xml、资源、页面、签名等子目录和文件。

/// 文档目录。
///
/// 描述单个 OFD 文档的目录结构，管理文档内的
/// 各种子容器（页面、资源、签名、注释等）。
#[derive(Debug, Clone)]
pub struct DocDir {
    /// 文档索引（从 0 开始）。
    pub index: u32,
    /// 文档根节点描述文件名称。
    pub document_file_name: String,
    /// 公共资源索引描述文件名称。
    pub public_res_file_name: String,
    /// 文档自身资源索引描述文件名称。
    pub document_res_file_name: String,
    /// 数字签名容器名称。
    pub signs_dir_name: String,
    /// 页面容器名称。
    pub pages_dir_name: String,
    /// 资源容器名称。
    pub res_dir_name: String,
    /// 注释容器名称。
    pub annots_dir_name: String,
    /// 注释入口文件名称。
    pub annotations_file_name: String,
    /// 附件入口文件名称。
    pub attachments_file_name: String,
}

impl DocDir {
    /// 文档容器名称前缀。
    pub const DOC_CONTAINER_PREFIX: &'static str = "Doc_";
    /// 文档根节点描述文件名。
    pub const DOCUMENT_FILE_NAME: &'static str = "Document.xml";
    /// 公共资源索引文件名。
    pub const PUBLIC_RES_FILE_NAME: &'static str = "PublicRes.xml";
    /// 文档自身资源索引文件名。
    pub const DOCUMENT_RES_FILE_NAME: &'static str = "DocumentRes.xml";
    /// 数字签名容器名。
    pub const SIGNS_DIR_NAME: &'static str = "Signs";
    /// 数字签名容器名称前缀。
    pub const SIGN_CONTAINER_PREFIX: &'static str = "Sign_";
    /// 自定义标签容器名。
    pub const TAGS_DIR_NAME: &'static str = "Tags";
    /// 临时文件容器名。
    pub const TEMPS_DIR_NAME: &'static str = "Temps";
    /// 页面容器名。
    pub const PAGES_DIR_NAME: &'static str = "Pages";
    /// 页面容器名称前缀。
    pub const PAGE_CONTAINER_PREFIX: &'static str = "Page_";
    /// 资源容器名。
    pub const RES_DIR_NAME: &'static str = "Res";
    /// 注释容器名。
    pub const ANNOTS_DIR_NAME: &'static str = "Annots";
    /// 注释入口文件名。
    pub const ANNOTATIONS_FILE_NAME: &'static str = "Annotations.xml";
    /// 附件入口文件名。
    pub const ATTACHMENTS_FILE_NAME: &'static str = "Attachments.xml";

    /// 创建新的文档目录。
    #[must_use]
    pub fn new(index: u32) -> Self {
        Self {
            index,
            document_file_name: Self::DOCUMENT_FILE_NAME.to_string(),
            public_res_file_name: Self::PUBLIC_RES_FILE_NAME.to_string(),
            document_res_file_name: Self::DOCUMENT_RES_FILE_NAME.to_string(),
            signs_dir_name: Self::SIGNS_DIR_NAME.to_string(),
            pages_dir_name: Self::PAGES_DIR_NAME.to_string(),
            res_dir_name: Self::RES_DIR_NAME.to_string(),
            annots_dir_name: Self::ANNOTS_DIR_NAME.to_string(),
            annotations_file_name: Self::ANNOTATIONS_FILE_NAME.to_string(),
            attachments_file_name: Self::ATTACHMENTS_FILE_NAME.to_string(),
        }
    }

    /// 获取文档容器目录名。
    #[must_use]
    pub fn container_name(&self) -> String {
        format!("{}{}", Self::DOC_CONTAINER_PREFIX, self.index)
    }

    /// 获取文档索引。
    pub fn index(&self) -> u32 {
        self.index
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_doc_dir_new() {
        let dd = DocDir::new(0);
        assert_eq!(dd.index, 0);
        assert_eq!(dd.document_file_name, "Document.xml");
        assert_eq!(dd.signs_dir_name, "Signs");
        assert_eq!(dd.pages_dir_name, "Pages");
    }

    #[test]
    fn test_doc_dir_container_name() {
        assert_eq!(DocDir::new(0).container_name(), "Doc_0");
        assert_eq!(DocDir::new(3).container_name(), "Doc_3");
    }

    #[test]
    fn test_doc_dir_constants() {
        assert_eq!(DocDir::DOC_CONTAINER_PREFIX, "Doc_");
        assert_eq!(DocDir::DOCUMENT_FILE_NAME, "Document.xml");
        assert_eq!(DocDir::SIGNS_DIR_NAME, "Signs");
        assert_eq!(DocDir::PAGES_DIR_NAME, "Pages");
        assert_eq!(DocDir::RES_DIR_NAME, "Res");
        assert_eq!(DocDir::SIGN_CONTAINER_PREFIX, "Sign_");
    }

    #[test]
    fn test_doc_dir_index() {
        let dd = DocDir::new(5);
        assert_eq!(dd.index(), 5);
    }
}
