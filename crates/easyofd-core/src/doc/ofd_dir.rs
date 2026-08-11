//! OFD 包根目录（OFDDir）。
//!
//! 对应 Java: org.ofdrw.pkg.container.OFDDir
//!
//! OFD 文档包的根目录，对应 ZIP 包的根。
//! 包含 OFD.xml 主入口文件和多个文档目录（Doc_N/）。

/// OFD 包根目录。
///
/// 描述 OFD 文档包的顶层结构，管理文档目录列表。
/// 每个 OFD 包可以包含多个文档（Doc_0, Doc_1, ...）。
#[derive(Debug, Clone)]
pub struct OfdDir {
    /// OFD 文档主入口文件名称。
    pub ofd_file_name: String,
    /// 解密入口文件名称。
    pub encryptions_file_name: String,
    /// OFD 防止夹带文件名称。
    pub ofd_entries_file_name: String,
    /// 文档目录列表（如 Doc_0, Doc_1, ...）。
    pub doc_dirs: Vec<String>,
    /// 最大文档索引 + 1。
    pub max_doc_index: u32,
}

impl OfdDir {
    /// OFD 主入口文件名。
    pub const OFD_FILE_NAME: &'static str = "OFD.xml";
    /// 解密入口文件名。
    pub const ENCRYPTIONS_FILE_NAME: &'static str = "Encryptions.xml";
    /// 防止夹带文件名。
    pub const OFD_ENTRIES_FILE_NAME: &'static str = "OFDEntries.xml";

    /// 创建新的 OFD 包根目录。
    #[must_use]
    pub fn new() -> Self {
        Self {
            ofd_file_name: Self::OFD_FILE_NAME.to_string(),
            encryptions_file_name: Self::ENCRYPTIONS_FILE_NAME.to_string(),
            ofd_entries_file_name: Self::OFD_ENTRIES_FILE_NAME.to_string(),
            doc_dirs: Vec::new(),
            max_doc_index: 0,
        }
    }

    /// 新建一个文档目录，返回目录名。
    pub fn new_doc(&mut self) -> String {
        let name = format!("Doc_{}", self.max_doc_index);
        self.max_doc_index += 1;
        self.doc_dirs.push(name.clone());
        name
    }

    /// 获取指定索引的文档目录名。
    #[must_use]
    pub fn doc_dir_name(index: u32) -> String {
        format!("Doc_{index}")
    }

    /// 获取文档目录数量。
    pub fn doc_count(&self) -> usize {
        self.doc_dirs.len()
    }
}

impl Default for OfdDir {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ofd_dir_new() {
        let dir = OfdDir::new();
        assert_eq!(dir.ofd_file_name, "OFD.xml");
        assert_eq!(dir.encryptions_file_name, "Encryptions.xml");
        assert_eq!(dir.ofd_entries_file_name, "OFDEntries.xml");
        assert!(dir.doc_dirs.is_empty());
        assert_eq!(dir.max_doc_index, 0);
    }

    #[test]
    fn test_ofd_dir_new_doc() {
        let mut dir = OfdDir::new();
        let doc0 = dir.new_doc();
        assert_eq!(doc0, "Doc_0");
        let doc1 = dir.new_doc();
        assert_eq!(doc1, "Doc_1");
        assert_eq!(dir.doc_count(), 2);
        assert_eq!(dir.max_doc_index, 2);
    }

    #[test]
    fn test_ofd_dir_doc_dir_name() {
        assert_eq!(OfdDir::doc_dir_name(0), "Doc_0");
        assert_eq!(OfdDir::doc_dir_name(5), "Doc_5");
    }

    #[test]
    fn test_ofd_dir_default() {
        let dir = OfdDir::default();
        assert_eq!(dir.ofd_file_name, "OFD.xml");
    }
}
