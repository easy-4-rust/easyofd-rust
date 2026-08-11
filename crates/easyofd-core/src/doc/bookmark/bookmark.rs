//! 单个书签。

/// 对应 Java: org.ofdrw.core.pageDescription.Bookmark
///
/// 单个书签定义，包含书签名称和目标页码。
#[derive(Debug, Clone)]
pub struct Bookmark {
    /// 书签名称。
    pub name: String,
    /// 目标页码（1-based）。
    pub page: u32,
    /// Y 偏移量（mm），可选。
    pub y_offset: Option<f64>,
}

impl Bookmark {
    /// 创建新的书签。
    #[must_use]
    pub fn new(name: impl Into<String>, page: u32) -> Self {
        Self {
            name: name.into(),
            page,
            y_offset: None,
        }
    }

    /// 设置 Y 偏移量。
    #[must_use]
    pub fn with_y_offset(mut self, y_offset: f64) -> Self {
        self.y_offset = Some(y_offset);
        self
    }

    /// 序列化为 XML 字符串。
    #[must_use]
    pub fn to_xml_string(&self) -> String {
        let mut attrs = format!("Name=\"{}\" Page=\"{}\"", self.name, self.page);
        if let Some(y) = self.y_offset {
            use std::fmt::Write;
            let _ = write!(attrs, " YOffset=\"{y}\"");
        }
        format!("<Bookmark {attrs}/>")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bookmark_new() {
        let b = Bookmark::new("Introduction", 1);
        assert_eq!(b.name, "Introduction");
        assert_eq!(b.page, 1);
        assert!(b.y_offset.is_none());
    }

    #[test]
    fn test_bookmark_with_y_offset_and_xml() {
        let b = Bookmark::new("Chapter 1", 3).with_y_offset(120.5);
        let xml = b.to_xml_string();
        assert!(xml.contains("Name=\"Chapter 1\""));
        assert!(xml.contains("Page=\"3\""));
        assert!(xml.contains("YOffset=\"120.5\""));
    }
}
