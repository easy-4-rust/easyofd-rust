//! 页面块类型枚举。
//!
//! 对应 Java: org.ofdrw.core.basicStructure.pageObj.layer.PageBlockType

/// 页面块类型，用于标识页面块中不同类型的图元。
///
/// 对应 Java: org.ofdrw.core.basicStructure.pageObj.layer.PageBlockType
///
/// 在 GB/T 33190-2016 中，页面块可以包含文本对象、路径对象、
/// 图像对象、复合对象等多种图元类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageBlockType {
    /// 文本对象。
    TextObject,
    /// 路径对象。
    PathObject,
    /// 图像对象。
    ImageObject,
    /// 复合对象。
    CompositeObject,
}

impl PageBlockType {
    /// 转为 OFD XML 元素名。
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::TextObject => "TextObject",
            Self::PathObject => "PathObject",
            Self::ImageObject => "ImageObject",
            Self::CompositeObject => "CompositeObject",
        }
    }
}

impl std::fmt::Display for PageBlockType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_page_block_type_as_str() {
        assert_eq!(PageBlockType::TextObject.as_str(), "TextObject");
        assert_eq!(PageBlockType::PathObject.as_str(), "PathObject");
        assert_eq!(PageBlockType::ImageObject.as_str(), "ImageObject");
        assert_eq!(
            PageBlockType::CompositeObject.as_str(),
            "CompositeObject"
        );
    }

    #[test]
    fn test_page_block_type_display() {
        assert_eq!(PageBlockType::TextObject.to_string(), "TextObject");
        assert_eq!(PageBlockType::PathObject.to_string(), "PathObject");
    }

    #[test]
    fn test_page_block_type_debug() {
        assert!(format!("{:?}", PageBlockType::ImageObject).contains("ImageObject"));
    }
}
