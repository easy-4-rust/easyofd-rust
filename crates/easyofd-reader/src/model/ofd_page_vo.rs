//! OFD 页面视图对象。
//!
//! 对应 Java: org.ofdrw.reader.model.OfdPageVo
//!
//! 已废弃：Java 原始类标记为 `@Deprecated`，建议使用 `PageInfo`。
//! 此处保留类型以保持 API 兼容。

/// OFD 页面视图对象，包含内容页面和可选的模板页面。
///
/// 对应 Java: `org.ofdrw.reader.model.OfdPageVo`
///
/// **废弃**：Java 原始类标记为 `@Deprecated`，建议使用 [`PageInfo`](crate::PageInfo)。
#[derive(Debug, Clone)]
#[deprecated(since = "1.0.0", note = "使用 PageInfo 替代")]
#[allow(deprecated)]
pub struct OfdPageVo {
    /// 内容页面路径。
    pub content_page_path: String,
    /// 模板页面路径（可选）。
    pub template_page_path: Option<String>,
}

#[allow(deprecated)]
impl OfdPageVo {
    /// 创建新的页面视图对象。
    #[must_use]
    pub fn new(content_page_path: impl Into<String>, template_page_path: Option<String>) -> Self {
        Self {
            content_page_path: content_page_path.into(),
            template_page_path,
        }
    }

    /// 是否有模板页面。
    #[must_use]
    pub fn has_template(&self) -> bool {
        self.template_page_path.is_some()
    }
}

#[allow(deprecated)]
#[cfg(test)]
mod tests {
    #[allow(deprecated)]
    use super::*;

    #[test]
    #[allow(deprecated)]
    fn test_ofd_page_vo_new() {
        let vo = OfdPageVo::new("Pages/Page_0/Content.xml", None);
        assert_eq!(vo.content_page_path, "Pages/Page_0/Content.xml");
        assert!(!vo.has_template());
    }

    #[test]
    #[allow(deprecated)]
    fn test_ofd_page_vo_with_template() {
        let vo = OfdPageVo::new(
            "Pages/Page_0/Content.xml",
            Some("Pages/Page_0/Template.xml".into()),
        );
        assert!(vo.has_template());
    }
}
