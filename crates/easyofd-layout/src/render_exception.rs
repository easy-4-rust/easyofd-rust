//! 渲染异常。
//!
//! 对应 Java: org.ofdrw.layout.engine.render.RenderException

use std::fmt;

/// 渲染异常，表示布局渲染过程中发生的错误。
///
/// 对应 Java: ofdrw layout engine render RenderException（RuntimeException）。
#[derive(Debug)]
pub struct RenderException {
    /// 错误消息。
    message: String,
    /// 原始错误（如果有）。
    source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl RenderException {
    /// 创建渲染异常。
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            source: None,
        }
    }

    /// 创建带原因的渲染异常（对应 Java: RenderException(String, Throwable)）。
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

    /// 从错误创建渲染异常（对应 Java: RenderException(Throwable)）。
    #[must_use]
    pub fn from_error(source: impl std::error::Error + Send + Sync + 'static) -> Self {
        let message = source.to_string();
        Self {
            message,
            source: Some(Box::new(source)),
        }
    }

    /// 获取错误消息。
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for RenderException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "渲染异常: {}", self.message)?;
        if let Some(source) = &self.source {
            write!(f, ": {source}")?;
        }
        Ok(())
    }
}

impl std::error::Error for RenderException {
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
        let e = RenderException::new("绘制失败");
        assert_eq!(e.message(), "绘制失败");
        assert!(e.source().is_none());
    }

    #[test]
    fn test_display() {
        let e = RenderException::new("测试错误");
        assert_eq!(e.to_string(), "渲染异常: 测试错误");
    }

    #[test]
    fn test_with_source() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "文件不存在");
        let e = RenderException::with_source("读取图片失败", io_err);
        assert_eq!(e.message(), "读取图片失败");
        assert!(e.source().is_some());
        assert!(e.to_string().contains("文件不存在"));
    }

    #[test]
    fn test_from_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "无权限");
        let e = RenderException::from_error(io_err);
        assert!(e.message().contains("无权限"));
        assert!(e.source().is_some());
    }

    #[test]
    fn test_is_std_error() {
        fn check_error(e: &dyn std::error::Error) -> bool {
            e.source().is_some() || !e.to_string().is_empty()
        }
        let e = RenderException::new("test");
        assert!(check_error(&e));
    }
}
