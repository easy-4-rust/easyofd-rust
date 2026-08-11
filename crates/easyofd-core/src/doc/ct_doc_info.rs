//! 文档元数据信息（CT_DocInfo）。
//!
//! 对应 Java: org.ofdrw.core.basicStructure.ofd.docInfo.CT_DocInfo
//!
//! 描述 OFD 文档的标题、作者、创建日期、修改日期、DocID 等元数据。

use std::fmt::Write;

/// 文档用途枚举。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocUsage {
    /// 普通文档（默认值）。
    Normal,
    /// 电子书。
    EBook,
    /// 电子报纸。
    ENewsPaper,
    /// 电子期刊。
    EMagazine,
}

impl DocUsage {
    /// 获取枚举的字符串表示。
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Normal => "Normal",
            Self::EBook => "EBook",
            Self::ENewsPaper => "ENewsPaper",
            Self::EMagazine => "EMagazine",
        }
    }

    /// 从字符串解析。
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "Normal" => Ok(Self::Normal),
            "EBook" => Ok(Self::EBook),
            "ENewsPaper" => Ok(Self::ENewsPaper),
            "EMagazine" => Ok(Self::EMagazine),
            _ => Err(format!("未知的文档用途: {s}")),
        }
    }
}

impl std::fmt::Display for DocUsage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// 文档元数据信息。
///
/// 描述文档的标识符、标题、作者、创建日期、修改日期等信息。
#[derive(Debug, Clone)]
pub struct CtDocInfo {
    /// 文件标识符，UUID 格式（必选）。每个 DocID 在文件创建时分配。
    pub doc_id: String,
    /// 文档标题（可选），标题可以与文件名不同。
    pub title: Option<String>,
    /// 文档作者（可选）。
    pub author: Option<String>,
    /// 文档主题（可选）。
    pub subject: Option<String>,
    /// 文档摘要与注释（可选）。
    pub abstract_text: Option<String>,
    /// 文件创建日期（可选），格式 `YYYY-MM-DD`。
    pub creation_date: Option<String>,
    /// 文档最近修改日期（可选），格式 `YYYY-MM-DD`。
    pub mod_date: Option<String>,
    /// 文档分类（可选），默认 Normal。
    pub doc_usage: Option<DocUsage>,
    /// 文档封面路径（可选），指向一个图片文件。
    pub cover: Option<String>,
    /// 关键词列表（可选）。
    pub keywords: Vec<String>,
    /// 创建文档的应用程序（可选）。
    pub creator: Option<String>,
    /// 创建文档的应用程序版本（可选）。
    pub creator_version: Option<String>,
}

impl CtDocInfo {
    /// 创建新的文档元数据信息。
    #[must_use]
    pub fn new(doc_id: impl Into<String>) -> Self {
        Self {
            doc_id: doc_id.into(),
            title: None,
            author: None,
            subject: None,
            abstract_text: None,
            creation_date: None,
            mod_date: None,
            doc_usage: None,
            cover: None,
            keywords: Vec::new(),
            creator: None,
            creator_version: None,
        }
    }

    /// 设置文档标题。
    #[must_use]
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// 设置文档作者。
    #[must_use]
    pub fn author(mut self, author: impl Into<String>) -> Self {
        self.author = Some(author.into());
        self
    }

    /// 设置文档主题。
    #[must_use]
    pub fn subject(mut self, subject: impl Into<String>) -> Self {
        self.subject = Some(subject.into());
        self
    }

    /// 设置文档摘要。
    #[must_use]
    pub fn abstract_text(mut self, text: impl Into<String>) -> Self {
        self.abstract_text = Some(text.into());
        self
    }

    /// 设置创建日期。
    #[must_use]
    pub fn creation_date(mut self, date: impl Into<String>) -> Self {
        self.creation_date = Some(date.into());
        self
    }

    /// 设置修改日期。
    #[must_use]
    pub fn mod_date(mut self, date: impl Into<String>) -> Self {
        self.mod_date = Some(date.into());
        self
    }

    /// 设置文档用途。
    #[must_use]
    pub fn doc_usage(mut self, usage: DocUsage) -> Self {
        self.doc_usage = Some(usage);
        self
    }

    /// 设置封面路径。
    #[must_use]
    pub fn cover(mut self, cover: impl Into<String>) -> Self {
        self.cover = Some(cover.into());
        self
    }

    /// 添加关键词。
    #[must_use]
    pub fn add_keyword(mut self, keyword: impl Into<String>) -> Self {
        self.keywords.push(keyword.into());
        self
    }

    /// 设置创建者应用程序。
    #[must_use]
    pub fn creator(mut self, creator: impl Into<String>) -> Self {
        self.creator = Some(creator.into());
        self
    }

    /// 设置创建者版本。
    #[must_use]
    pub fn creator_version(mut self, version: impl Into<String>) -> Self {
        self.creator_version = Some(version.into());
        self
    }

    /// 序列化为 XML 字符串。
    #[must_use]
    pub fn to_xml_string(&self) -> String {
        let mut xml = String::from("<ofd:DocInfo>");

        let _ = write!(xml, "\n<ofd:DocID>{}</ofd:DocID>", self.doc_id);

        if let Some(ref title) = self.title {
            let _ = write!(xml, "\n<ofd:Title>{title}</ofd:Title>");
        }
        if let Some(ref author) = self.author {
            let _ = write!(xml, "\n<ofd:Author>{author}</ofd:Author>");
        }
        if let Some(ref subject) = self.subject {
            let _ = write!(xml, "\n<ofd:Subject>{subject}</ofd:Subject>");
        }
        if let Some(ref abs) = self.abstract_text {
            let _ = write!(xml, "\n<ofd:Abstract>{abs}</ofd:Abstract>");
        }
        if let Some(ref date) = self.creation_date {
            let _ = write!(xml, "\n<ofd:CreationDate>{date}</ofd:CreationDate>");
        }
        if let Some(ref date) = self.mod_date {
            let _ = write!(xml, "\n<ofd:ModDate>{date}</ofd:ModDate>");
        }
        if let Some(ref usage) = self.doc_usage {
            let _ = write!(xml, "\n<ofd:DocUsage>{}</ofd:DocUsage>", usage.as_str());
        }
        if let Some(ref cover) = self.cover {
            let _ = write!(xml, "\n<ofd:Cover>{cover}</ofd:Cover>");
        }
        if !self.keywords.is_empty() {
            xml.push_str("\n<ofd:Keywords>");
            for kw in &self.keywords {
                let _ = write!(xml, "\n<ofd:Keyword>{kw}</ofd:Keyword>");
            }
            xml.push_str("\n</ofd:Keywords>");
        }
        if let Some(ref c) = self.creator {
            let _ = write!(xml, "\n<ofd:Creator>{c}</ofd:Creator>");
        }
        if let Some(ref v) = self.creator_version {
            let _ = write!(xml, "\n<ofd:CreatorVersion>{v}</ofd:CreatorVersion>");
        }

        xml.push_str("\n</ofd:DocInfo>");
        xml
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ct_doc_info_new() {
        let info = CtDocInfo::new("550e8400e29b41d4a716446655440000");
        assert_eq!(info.doc_id, "550e8400e29b41d4a716446655440000");
        assert!(info.title.is_none());
        assert!(info.author.is_none());
        assert!(info.keywords.is_empty());
    }

    #[test]
    fn test_ct_doc_info_builder() {
        let info = CtDocInfo::new("abc123")
            .title("测试文档")
            .author("张三")
            .subject("测试主题")
            .creation_date("2025-01-01")
            .mod_date("2025-06-01")
            .doc_usage(DocUsage::EBook)
            .add_keyword("OFD")
            .add_keyword("测试")
            .creator("easyofd-rust")
            .creator_version("1.0");
        assert_eq!(info.title.as_deref(), Some("测试文档"));
        assert_eq!(info.author.as_deref(), Some("张三"));
        assert_eq!(info.doc_usage, Some(DocUsage::EBook));
        assert_eq!(info.keywords.len(), 2);
    }

    #[test]
    fn test_ct_doc_info_xml() {
        let info = CtDocInfo::new("doc001").title("My Doc").author("Author1");
        let xml = info.to_xml_string();
        assert!(xml.contains("<ofd:DocID>doc001</ofd:DocID>"));
        assert!(xml.contains("<ofd:Title>My Doc</ofd:Title>"));
        assert!(xml.contains("<ofd:Author>Author1</ofd:Author>"));
        assert!(xml.contains("</ofd:DocInfo>"));
    }

    #[test]
    fn test_doc_usage_display() {
        assert_eq!(DocUsage::Normal.to_string(), "Normal");
        assert_eq!(DocUsage::EBook.to_string(), "EBook");
        assert_eq!(DocUsage::from_str("Normal").unwrap(), DocUsage::Normal);
        assert!(DocUsage::from_str("Invalid").is_err());
    }
}
