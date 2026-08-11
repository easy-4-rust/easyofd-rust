//! 通用转换异常。
//!
//! 对应 Java: org.ofdrw.converter.GeneralConvertException

/// 通用转换异常。
///
/// 对应 Java: org.ofdrw.converter.GeneralConvertException
///
/// 在 OFD 转换过程中发生的通用异常，包括格式不支持、
/// 资源缺失、渲染失败等情况。
#[derive(Debug, Clone)]
pub struct GeneralConvertException {
    /// 错误消息。
    pub message: String,
    /// 错误来源（可选）。
    pub source: Option<String>,
}

impl GeneralConvertException {
    /// 创建新的转换异常。
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            source: None,
        }
    }

    /// 设置错误来源。
    #[must_use]
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    /// 获取错误消息。
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// 获取错误来源。
    #[must_use]
    pub fn source(&self) -> Option<&str> {
        self.source.as_deref()
    }
}

impl std::fmt::Display for GeneralConvertException {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "OFD 转换异常: {}", self.message)?;
        if let Some(ref src) = self.source {
            write!(f, " (来源: {src})")?;
        }
        Ok(())
    }
}

impl std::error::Error for GeneralConvertException {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_general_convert_exception_new() {
        let e = GeneralConvertException::new("不支持的格式");
        assert_eq!(e.message(), "不支持的格式");
        assert!(e.source().is_none());
    }

    #[test]
    fn test_general_convert_exception_with_source() {
        let e = GeneralConvertException::new("渲染失败")
            .with_source("PdfExporter");
        assert_eq!(e.message(), "渲染失败");
        assert_eq!(e.source(), Some("PdfExporter"));
    }

    #[test]
    fn test_general_convert_exception_display() {
        let e = GeneralConvertException::new("test error");
        assert!(e.to_string().contains("test error"));
    }

    #[test]
    fn test_general_convert_exception_display_with_source() {
        let e = GeneralConvertException::new("test")
            .with_source("src");
        assert!(e.to_string().contains("src"));
    }

    #[test]
    fn test_general_convert_exception_is_error() {
        let e: Box<dyn std::error::Error> =
            Box::new(GeneralConvertException::new("test"));
        assert!(e.to_string().contains("test"));
    }
}
