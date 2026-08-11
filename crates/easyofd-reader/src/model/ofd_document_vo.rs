//! OFD 文档视图对象。
//!
//! 对应 Java: org.ofdrw.reader.model.OFDDocumentVo
//!
//! 已废弃：Java 原始类标记为 `@Deprecated`，建议使用 `DLOFDReader`。

use super::AnnotionEntity;

/// OFD 文档视图对象，包含文档的基本信息和页面列表。
///
/// 对应 Java: `org.ofdrw.reader.model.OFDDocumentVo`
///
/// **废弃**：Java 原始类标记为 `@Deprecated`。
#[derive(Debug, Clone)]
#[deprecated(since = "1.0.0", note = "使用 OfdReader 直接解析替代")]
#[allow(deprecated)]
pub struct OfdDocumentVo {
    /// 文档路径。
    pub doc_path: String,
    /// 页面宽度（毫米）。
    pub page_width: f64,
    /// 页面高度（毫米）。
    pub page_height: f64,
    /// 注释列表。
    pub annotations: Vec<AnnotionEntity>,
}

#[allow(deprecated)]
impl OfdDocumentVo {
    /// 创建新的文档视图对象。
    #[must_use]
    pub fn new(
        doc_path: impl Into<String>,
        page_width: f64,
        page_height: f64,
        annotations: Vec<AnnotionEntity>,
    ) -> Self {
        Self {
            doc_path: doc_path.into(),
            page_width,
            page_height,
            annotations,
        }
    }

    /// 获取注释列表。
    #[must_use]
    pub fn annotations(&self) -> &[AnnotionEntity] {
        &self.annotations
    }
}

#[allow(deprecated)]
#[cfg(test)]
mod tests {
    #[allow(deprecated)]
    use super::*;

    #[test]
    #[allow(deprecated)]
    fn test_ofd_document_vo_new() {
        let vo = OfdDocumentVo::new("Doc_0", 210.0, 297.0, vec![]);
        assert_eq!(vo.doc_path, "Doc_0");
        assert!((vo.page_width - 210.0).abs() < f64::EPSILON);
        assert!((vo.page_height - 297.0).abs() < f64::EPSILON);
        assert!(vo.annotations.is_empty());
    }

    #[test]
    #[allow(deprecated)]
    fn test_ofd_document_vo_with_annotations() {
        let ann = AnnotionEntity::new("page_0", vec!["<Annot/>".into()]);
        let vo = OfdDocumentVo::new("Doc_0", 210.0, 297.0, vec![ann]);
        assert_eq!(vo.annotations().len(), 1);
    }
}
