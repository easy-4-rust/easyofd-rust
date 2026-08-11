//! OFD 文件解析异常。
//!
//! 对应 Java: org.ofdrw.layout.exception.DocReadException

use std::fmt;

/// OFD 文件解析异常。
///
/// 对应 Java: ofdrw layout exception DocReadException（IOException）。
#[derive(Debug)]
pub struct DocReadException {
    /// 错误消息。
    message: String,
    /// 原始错误（如果有）。
    source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl DocReadException {
    /// 创建文档读取异常。
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            source: None,
        }
    }

    /// 创建带原因的文档读取异常（对应 Java: DocReadException(String, Throwable)）。
    #[must_use]
    pub fn with_source(
        message: impl Into<String>,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }

    /// 获取错误消息。
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for DocReadException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "OFD 文件解析异常: {}", self.message)?;
        if let Some(source) = &self.source {
            write!(f, ": {source}")?;
        }
        Ok(())
    }
}

impl std::error::Error for DocReadException {
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
        let e = DocReadException::new("文件不存在");
        assert_eq!(e.message(), "文件不存在");
        assert!(e.source().is_none());
    }

    #[test]
    fn test_display() {
        let e = DocReadException::new("解析失败");
        assert_eq!(e.to_string(), "OFD 文件解析异常: 解析失败");
    }

    #[test]
    fn test_with_source() {
        let io_err = std::io::Error::new(std::io::ErrorKind::InvalidData, "损坏");
        let e = DocReadException::with_source("读取 OFD 失败", io_err);
        assert!(e.source().is_some());
        assert!(e.to_string().contains("损坏"));
    }
}
