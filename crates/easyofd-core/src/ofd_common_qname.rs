//! OFD 公共 XML 限定名（QName）。
//!
//! 对应 Java: org.ofdrw.core.OFDCommonQName
//!
//! 封装 OFD 标准中常用的 XML 元素名与命名空间前缀组合，
//! 避免各处硬编码字符串。

/// OFD 公共 XML 限定名。
///
/// 对应 Java: org.ofdrw.core.OFDCommonQName
///
/// 表示一个带命名空间前缀的 XML 元素名，如 `ofd:Page`。
/// 提供常用 QName 常量与自定义构造。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OfdCommonQName {
    /// 命名空间前缀（如 "ofd"）。
    pub prefix: String,
    /// 本地元素名（如 "Page"）。
    pub local_name: String,
}

impl OfdCommonQName {
    /// 创建新的 QName。
    #[must_use]
    pub fn new(prefix: impl Into<String>, local_name: impl Into<String>) -> Self {
        Self {
            prefix: prefix.into(),
            local_name: local_name.into(),
        }
    }

    /// 获取完整的限定名（`prefix:local_name`）。
    #[must_use]
    pub fn qualified_name(&self) -> String {
        format!("{}:{}", self.prefix, self.local_name)
    }

    // ── 常用 QName 常量 ────────────────────────────────────────────────

    /// `ofd:OFD`。
    #[must_use]
    pub fn ofd() -> Self {
        Self::new("ofd", "OFD")
    }

    /// `ofd:Document`。
    #[must_use]
    pub fn document() -> Self {
        Self::new("ofd", "Document")
    }

    /// `ofd:Page`。
    #[must_use]
    pub fn page() -> Self {
        Self::new("ofd", "Page")
    }

    /// `ofd:Pages`。
    #[must_use]
    pub fn pages() -> Self {
        Self::new("ofd", "Pages")
    }

    /// `ofd:DocBody`。
    #[must_use]
    pub fn doc_body() -> Self {
        Self::new("ofd", "DocBody")
    }

    /// `ofd:DocInfo`。
    #[must_use]
    pub fn doc_info() -> Self {
        Self::new("ofd", "DocInfo")
    }

    /// `ofd:Res`。
    #[must_use]
    pub fn res() -> Self {
        Self::new("ofd", "Res")
    }

    /// `ofd:Layer`。
    #[must_use]
    pub fn layer() -> Self {
        Self::new("ofd", "Layer")
    }

    /// `ofd:TextObject`。
    #[must_use]
    pub fn text_object() -> Self {
        Self::new("ofd", "TextObject")
    }

    /// `ofd:PathObject`。
    #[must_use]
    pub fn path_object() -> Self {
        Self::new("ofd", "PathObject")
    }

    /// `ofd:ImageObject`。
    #[must_use]
    pub fn image_object() -> Self {
        Self::new("ofd", "ImageObject")
    }

    /// `ofd:CompositeObject`。
    #[must_use]
    pub fn composite_object() -> Self {
        Self::new("ofd", "CompositeObject")
    }

    /// `ofd:Signature`。
    #[must_use]
    pub fn signature() -> Self {
        Self::new("ofd", "Signature")
    }

    /// `ofd:StampAnnot`。
    #[must_use]
    pub fn stamp_annot() -> Self {
        Self::new("ofd", "StampAnnot")
    }

    /// `ofd:Attachment`。
    #[must_use]
    pub fn attachment() -> Self {
        Self::new("ofd", "Attachment")
    }

    /// `ofd:TemplatePage`。
    #[must_use]
    pub fn template_page() -> Self {
        Self::new("ofd", "TemplatePage")
    }
}

impl std::fmt::Display for OfdCommonQName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.prefix, self.local_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_and_qualified_name() {
        let qn = OfdCommonQName::new("ofd", "Page");
        assert_eq!(qn.prefix, "ofd");
        assert_eq!(qn.local_name, "Page");
        assert_eq!(qn.qualified_name(), "ofd:Page");
    }

    #[test]
    fn test_display() {
        let qn = OfdCommonQName::new("ofd", "Document");
        assert_eq!(qn.to_string(), "ofd:Document");
    }

    #[test]
    fn test_common_qnames() {
        assert_eq!(OfdCommonQName::ofd().qualified_name(), "ofd:OFD");
        assert_eq!(OfdCommonQName::document().qualified_name(), "ofd:Document");
        assert_eq!(OfdCommonQName::page().qualified_name(), "ofd:Page");
        assert_eq!(OfdCommonQName::pages().qualified_name(), "ofd:Pages");
        assert_eq!(OfdCommonQName::doc_body().qualified_name(), "ofd:DocBody");
        assert_eq!(OfdCommonQName::doc_info().qualified_name(), "ofd:DocInfo");
        assert_eq!(OfdCommonQName::res().qualified_name(), "ofd:Res");
        assert_eq!(OfdCommonQName::layer().qualified_name(), "ofd:Layer");
        assert_eq!(
            OfdCommonQName::text_object().qualified_name(),
            "ofd:TextObject"
        );
        assert_eq!(
            OfdCommonQName::path_object().qualified_name(),
            "ofd:PathObject"
        );
        assert_eq!(
            OfdCommonQName::image_object().qualified_name(),
            "ofd:ImageObject"
        );
        assert_eq!(
            OfdCommonQName::composite_object().qualified_name(),
            "ofd:CompositeObject"
        );
        assert_eq!(
            OfdCommonQName::signature().qualified_name(),
            "ofd:Signature"
        );
        assert_eq!(
            OfdCommonQName::stamp_annot().qualified_name(),
            "ofd:StampAnnot"
        );
        assert_eq!(
            OfdCommonQName::attachment().qualified_name(),
            "ofd:Attachment"
        );
        assert_eq!(
            OfdCommonQName::template_page().qualified_name(),
            "ofd:TemplatePage"
        );
    }

    #[test]
    fn test_clone_eq() {
        let a = OfdCommonQName::page();
        let b = a.clone();
        assert_eq!(a, b);
    }
}
