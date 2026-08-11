//! 注释实体。
//!
//! 对应 Java: org.ofdrw.reader.model.AnnotionEntity
//!
//! 注意：Java 原始类名为 `AnnotionEntity`（拼写错误），此处保留原名以保持兼容。

/// 注释实体，描述一个页面上的注释集合。
///
/// 对应 Java: `org.ofdrw.reader.model.AnnotionEntity`
///
/// 注意：Java 原始类名为 `AnnotionEntity`（拼写错误，应为 `AnnotationEntity`），
/// 此处保留原名以保持 API 兼容。
#[derive(Debug, Clone)]
pub struct AnnotionEntity {
    /// 注释所在页面 ID。
    pub page_id: String,
    /// 注释列表（原始 XML 字符串表示）。
    ///
    /// Java 版使用 `List<Annot>` 对象，Rust 版存储为原始 XML 字符串，
    /// 因为 easyofd-core 的 `Annot` 类型已提供完整解析。
    pub annot_xmls: Vec<String>,
}

impl AnnotionEntity {
    /// 创建新的注释实体。
    #[must_use]
    pub fn new(page_id: impl Into<String>, annot_xmls: Vec<String>) -> Self {
        Self {
            page_id: page_id.into(),
            annot_xmls,
        }
    }

    /// 注释数量。
    #[must_use]
    pub fn len(&self) -> usize {
        self.annot_xmls.len()
    }

    /// 是否没有注释。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.annot_xmls.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_annotation_entity_new() {
        let entity = AnnotionEntity::new("page_0", vec!["<Annot/>".into()]);
        assert_eq!(entity.page_id, "page_0");
        assert_eq!(entity.len(), 1);
        assert!(!entity.is_empty());
    }

    #[test]
    fn test_annotation_entity_empty() {
        let entity = AnnotionEntity::new("page_1", vec![]);
        assert!(entity.is_empty());
        assert_eq!(entity.len(), 0);
    }

    #[test]
    fn test_annotation_entity_clone() {
        let entity = AnnotionEntity::new("p0", vec!["a".into(), "b".into()]);
        let cloned = entity.clone();
        assert_eq!(cloned.page_id, entity.page_id);
        assert_eq!(cloned.annot_xmls, entity.annot_xmls);
    }
}
