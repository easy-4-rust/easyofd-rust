//! 注解渲染器。
//!
//! 对应 Java: org.ofdrw.layout.edit.AnnotationRender

use crate::annotation::AnnotType;

/// 注解渲染配置。
///
/// 对应 Java: ofdrw layout edit AnnotationRender。
#[derive(Debug, Clone, PartialEq)]
pub struct AnnotationRender {
    /// 注解类型。
    pub annot_type: AnnotType,
    /// 注解 ID。
    pub id: u32,
    /// 注解内容描述。
    pub content: Option<String>,
}

impl AnnotationRender {
    /// 创建注解渲染配置。
    #[must_use]
    pub fn new(annot_type: AnnotType, id: u32) -> Self {
        Self {
            annot_type,
            id,
            content: None,
        }
    }

    /// 设置注解内容。
    #[must_use]
    pub fn content(mut self, content: impl Into<String>) -> Self {
        self.content = Some(content.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let ar = AnnotationRender::new(AnnotType::Link, 1);
        assert_eq!(ar.annot_type, AnnotType::Link);
        assert_eq!(ar.id, 1);
        assert!(ar.content.is_none());
    }

    #[test]
    fn test_content() {
        let ar = AnnotationRender::new(AnnotType::Highlight, 2).content("备注内容");
        assert_eq!(ar.content.as_deref(), Some("备注内容"));
    }
}
