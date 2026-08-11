//! OFD 标准常量定义。
//!
//! 对应 Java: org.ofdrw.core.Const
//!
//! 定义 OFD 标准中使用的常量，包括命名空间 URI、元素名和属性名。

/// OFD 命名空间 URI。
///
/// 对应 Java: org.ofdrw.core.Const#OFD_NS
pub const OFD_NAMESPACE: &str = "http://www.ofdspec.org/2016";

/// OFD 元素前缀。
pub const OFD_PREFIX: &str = "ofd";

/// OFD 文档根元素名。
pub const OFD_ROOT_ELEMENT: &str = "OFD";

/// 文档根元素名。
pub const DOCUMENT_ELEMENT: &str = "Document";

/// 页面元素名。
pub const PAGE_ELEMENT: &str = "Page";

/// 页面树元素名。
pub const PAGES_ELEMENT: &str = "Pages";

/// 文档体元素名。
pub const DOC_BODY_ELEMENT: &str = "DocBody";

/// 资源元素名。
pub const RES_ELEMENT: &str = "Res";

/// 默认页面宽度（mm，A4）。
pub const DEFAULT_PAGE_WIDTH_MM: f64 = 210.0;

/// 默认页面高度（mm，A4）。
pub const DEFAULT_PAGE_HEIGHT_MM: f64 = 297.0;

/// OFD 版本号。
pub const OFD_VERSION: &str = "1.0";

/// 文本对象元素名。
pub const TEXT_OBJECT_ELEMENT: &str = "TextObject";

/// 路径对象元素名。
pub const PATH_OBJECT_ELEMENT: &str = "PathObject";

/// 图像对象元素名。
pub const IMAGE_OBJECT_ELEMENT: &str = "ImageObject";

/// 复合对象元素名。
pub const COMPOSITE_OBJECT_ELEMENT: &str = "CompositeObject";

/// 图层元素名。
pub const LAYER_ELEMENT: &str = "Layer";

/// 签名元素名。
pub const SIGNATURE_ELEMENT: &str = "Signature";

/// 印章注释元素名。
pub const STAMP_ANNOT_ELEMENT: &str = "StampAnnot";

/// 附件元素名。
pub const ATTACHMENT_ELEMENT: &str = "Attachment";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ofd_namespace() {
        assert_eq!(OFD_NAMESPACE, "http://www.ofdspec.org/2016");
    }

    #[test]
    fn test_default_page_size() {
        assert!((DEFAULT_PAGE_WIDTH_MM - 210.0).abs() < f64::EPSILON);
        assert!((DEFAULT_PAGE_HEIGHT_MM - 297.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_element_names() {
        assert_eq!(OFD_ROOT_ELEMENT, "OFD");
        assert_eq!(PAGE_ELEMENT, "Page");
        assert_eq!(TEXT_OBJECT_ELEMENT, "TextObject");
    }
}
