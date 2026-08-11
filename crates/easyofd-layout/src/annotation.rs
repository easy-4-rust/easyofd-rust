//! 布局注释。
//!
//! 对应 Java: org.ofdrw.layout.edit.Annotation

use std::collections::BTreeMap;

/// 注释类型（对应 Java: ofdrw AnnotType）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnnotType {
    /// 链接。
    Link,
    /// 路径。
    Path,
    /// 高亮。
    Highlight,
    /// 印章。
    Stamp,
    /// 水印。
    Watermark,
}

/// 布局注释（ofdrw layout Annotation）。
///
/// 对应 Java: ofdrw Annotation，在页面上放置注释对象。
#[derive(Debug, Clone, PartialEq)]
pub struct Annotation {
    /// 注释类型。
    pub annot_type: AnnotType,
    /// 创建者。
    pub creator: Option<String>,
    /// 子类型。
    pub subtype: Option<String>,
    /// 是否可见。
    pub visible: Option<bool>,
    /// 是否禁止旋转。
    pub no_rotate: Option<bool>,
    /// 参数表。
    pub parameters: BTreeMap<String, String>,
    /// 备注。
    pub remark: Option<String>,
    /// 是否只读。
    pub read_only: Option<bool>,
    /// 是否打印。
    pub print: Option<bool>,
    /// 边界（x, y, width, height）。
    pub boundary: Option<(f64, f64, f64, f64)>,
}

impl Annotation {
    /// 创建注释（对应 Java: Annotation(x, y, width, height, type)）。
    #[must_use]
    pub fn new(x: f64, y: f64, width: f64, height: f64, annot_type: AnnotType) -> Self {
        Self {
            annot_type,
            creator: None,
            subtype: None,
            visible: None,
            no_rotate: None,
            parameters: BTreeMap::new(),
            remark: None,
            read_only: None,
            print: None,
            boundary: Some((x, y, width, height)),
        }
    }

    /// 设置创建者（对应 Java: Annotation#setCreator）。
    #[must_use]
    pub fn creator(mut self, creator: impl Into<String>) -> Self {
        self.creator = Some(creator.into());
        self
    }

    /// 设置可见性（对应 Java: Annotation#setVisible）。
    #[must_use]
    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = Some(visible);
        self
    }

    /// 设置只读（对应 Java: Annotation#setReadOnly）。
    #[must_use]
    pub fn read_only(mut self, read_only: bool) -> Self {
        self.read_only = Some(read_only);
        self
    }

    /// 设置参数（对应 Java: Annotation#setParameters）。
    #[must_use]
    pub fn parameter(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.parameters.insert(key.into(), value.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_annotation_new() {
        let a = Annotation::new(10.0, 20.0, 100.0, 50.0, AnnotType::Highlight);
        assert_eq!(a.boundary, Some((10.0, 20.0, 100.0, 50.0)));
        assert_eq!(a.annot_type, AnnotType::Highlight);
    }

    #[test]
    fn test_builders() {
        let a = Annotation::new(0.0, 0.0, 10.0, 10.0, AnnotType::Stamp)
            .creator("dzzz")
            .visible(true)
            .read_only(true)
            .parameter("key", "value");
        assert_eq!(a.creator.as_deref(), Some("dzzz"));
        assert_eq!(a.visible, Some(true));
        assert_eq!(a.parameters.get("key").map(String::as_str), Some("value"));
    }

    #[test]
    fn test_annot_type_variants() {
        assert_ne!(AnnotType::Link, AnnotType::Watermark);
        assert_eq!(AnnotType::Path, AnnotType::Path);
    }
}
