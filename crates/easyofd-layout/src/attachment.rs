//! 附件。
//!
//! 对应 Java: org.ofdrw.layout.edit.Attachment

use std::path::PathBuf;

/// OFD 附件（ofdrw layout Attachment）。
///
/// 对应 Java: ofdrw Attachment，用于向 OFD 文档附加外部文件。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attachment {
    /// 附件名称。
    pub name: String,
    /// 附件文件路径。
    pub file: Option<PathBuf>,
    /// 附件格式（MIME 类型，可选）。
    pub format: Option<String>,
    /// 是否禁止替换（同名附件已存在时报错）。
    pub disable_replace: bool,
}

impl Attachment {
    /// 创建附件（对应 Java: Attachment(name, file)）。
    #[must_use]
    pub fn new(name: impl Into<String>, file: PathBuf) -> Self {
        Self {
            name: name.into(),
            file: Some(file),
            format: None,
            disable_replace: false,
        }
    }

    /// 设置附件名称（对应 Java: Attachment#setName）。
    #[must_use]
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// 设置附件格式（对应 Java: Attachment#setFormat）。
    #[must_use]
    pub fn format(mut self, format: impl Into<String>) -> Self {
        self.format = Some(format.into());
        self
    }

    /// 设置是否禁止替换（对应 Java: Attachment#setDisableReplace）。
    #[must_use]
    pub fn disable_replace(mut self, disable: bool) -> Self {
        self.disable_replace = disable;
        self
    }

    /// 附件文件大小（读取失败时返回 None）。
    #[must_use]
    pub fn size(&self) -> Option<u64> {
        self.file
            .as_ref()
            .and_then(|p| std::fs::metadata(p).ok())
            .map(|m| m.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attachment_new() {
        let a = Attachment::new("doc.pdf", PathBuf::from("/tmp/doc.pdf"));
        assert_eq!(a.name, "doc.pdf");
        assert!(a.file.is_some());
        assert!(!a.disable_replace);
    }

    #[test]
    fn test_builders() {
        let a = Attachment::new("a", PathBuf::from("/tmp/a"))
            .name("b.bin")
            .format("application/octet-stream")
            .disable_replace(true);
        assert_eq!(a.name, "b.bin");
        assert_eq!(a.format.as_deref(), Some("application/octet-stream"));
        assert!(a.disable_replace);
    }

    #[test]
    fn test_clone_eq() {
        let a = Attachment::new("x", PathBuf::from("/tmp/x"));
        assert_eq!(a, a.clone());
    }
}
