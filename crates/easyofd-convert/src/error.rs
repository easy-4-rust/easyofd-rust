//! 转换通用异常。
//!
//! 对应 Java: org.ofdrw.converter.GeneralConvertException

use std::fmt;

/// 转换过程中发生的通用错误。
///
/// 对应 Java `GeneralConvertException`，用于 OFD/PDF 转换过程中
/// 无法归类为特定错误类型的一般性错误。
///
/// 与 `easyofd_core::OfdError` 的区别在于：本类型专用于 converter 模块，
/// 可携带源错误链信息，且不依赖 OFD 核心类型。
#[derive(Debug)]
pub struct GeneralConvertError {
    /// 错误描述。
    message: String,
    /// 可选的源错误。
    source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl GeneralConvertError {
    /// 创建仅包含消息的错误。
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            source: None,
        }
    }

    /// 创建包含源错误的错误。
    pub fn with_source(
        message: impl Into<String>,
        source: impl Into<Box<dyn std::error::Error + Send + Sync>>,
    ) -> Self {
        Self {
            message: message.into(),
            source: Some(source.into()),
        }
    }

    /// 返回错误描述。
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for GeneralConvertError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "转换错误: {}", self.message)
    }
}

impl std::error::Error for GeneralConvertError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source
            .as_ref()
            .map(|e| e.as_ref() as &(dyn std::error::Error + 'static))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn test_new() {
        let err = GeneralConvertError::new("文件不存在");
        assert_eq!(err.message(), "文件不存在");
        assert!(err.source().is_none());
    }

    #[test]
    fn test_display() {
        let err = GeneralConvertError::new("解析失败");
        let display = format!("{err}");
        assert!(display.contains("解析失败"));
    }

    #[test]
    fn test_with_source() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "not found");
        let err = GeneralConvertError::with_source("读取失败", io_err);
        assert_eq!(err.message(), "读取失败");
        assert!(err.source().is_some());
    }

    #[test]
    fn test_error_trait() {
        let err: Box<dyn std::error::Error> = Box::new(GeneralConvertError::new("test"));
        assert!(err.source().is_none());
    }

    #[test]
    fn test_error_chain() {
        let inner = GeneralConvertError::new("inner");
        let outer = GeneralConvertError::with_source("outer", inner);
        let source = outer.source().unwrap();
        assert!(source.downcast_ref::<GeneralConvertError>().is_some());
    }
}
