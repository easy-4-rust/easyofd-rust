//! 文本资源（关键字搜索上下文）。
//!
//! 对应 Java: org.ofdrw.reader.keyword.KeywordResource

/// 文本资源，描述关键字搜索过程中的字体和文本上下文。
///
/// 对应 Java: `org.ofdrw.reader.keyword.KeywordResource`
///
/// 在关键字搜索时，需要字体信息来计算字符宽度，从而确定关键字的
/// 精确矩形区域。
#[derive(Debug, Clone)]
pub struct KeywordResource {
    /// 页码（从 1 开始）。
    pub page: usize,
    /// 字体引用 ID。
    pub font_id: Option<String>,
    /// 字体大小（毫米）。
    pub font_size: Option<f64>,
}

impl KeywordResource {
    /// 创建新的文本资源。
    #[must_use]
    pub fn new(page: usize) -> Self {
        Self {
            page,
            font_id: None,
            font_size: None,
        }
    }

    /// 设置字体引用 ID。
    #[must_use]
    pub fn with_font_id(mut self, font_id: impl Into<String>) -> Self {
        self.font_id = Some(font_id.into());
        self
    }

    /// 设置字体大小。
    #[must_use]
    pub fn with_font_size(mut self, size: f64) -> Self {
        self.font_size = Some(size);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyword_resource_new() {
        let res = KeywordResource::new(1);
        assert_eq!(res.page, 1);
        assert!(res.font_id.is_none());
        assert!(res.font_size.is_none());
    }

    #[test]
    fn test_keyword_resource_with_font() {
        let res = KeywordResource::new(2)
            .with_font_id("font_0")
            .with_font_size(12.0);
        assert_eq!(res.font_id.as_deref(), Some("font_0"));
        assert!((res.font_size.unwrap() - 12.0).abs() < f64::EPSILON);
    }
}
