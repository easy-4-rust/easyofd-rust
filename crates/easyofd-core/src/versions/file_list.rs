//! 文件列表。

use super::File;

/// 对应 Java: org.ofdrw.core.basicStructure.FileList
///
/// 文件列表容器，包含一个版本中所有文件的信息。
#[derive(Debug, Clone)]
pub struct FileList {
    /// 文件列表。
    pub files: Vec<File>,
}

impl FileList {
    /// 创建空的文件列表。
    #[must_use]
    pub fn new() -> Self {
        Self { files: Vec::new() }
    }

    /// 添加一个文件并返回自身（链式调用）。
    #[must_use]
    pub fn with(mut self, file: File) -> Self {
        self.files.push(file);
        self
    }

    /// 添加一个文件。
    pub fn push(&mut self, file: File) {
        self.files.push(file);
    }

    /// 序列化为 XML 字符串。
    #[must_use]
    pub fn to_xml_string(&self) -> String {
        let inner: String = self.files.iter().map(File::to_xml_string).collect();
        format!("<FileList>{inner}</FileList>")
    }
}

impl Default for FileList {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_list_new() {
        let fl = FileList::new();
        assert!(fl.files.is_empty());
        let fl2 = FileList::default();
        assert!(fl2.files.is_empty());
    }

    #[test]
    fn test_file_list_with_and_xml() {
        let fl = FileList::new()
            .with(File::new("a.xml", 100))
            .with(File::new("b.xml", 200));
        assert_eq!(fl.files.len(), 2);
        let xml = fl.to_xml_string();
        assert!(xml.contains("<FileList>"));
        assert!(xml.contains("a.xml"));
        assert!(xml.contains("b.xml"));
    }
}
