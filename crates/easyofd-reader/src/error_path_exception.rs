//! 错误路径异常。
//!
//! 对应 Java: org.ofdrw.reader.ErrorPathException

use thiserror::Error;

/// 错误路径异常。
///
/// 当资源定位器尝试切换到不存在的目录路径时抛出。
///
/// 对应 Java: `org.ofdrw.reader.ErrorPathException`
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ErrorPathException {
    /// 路径不存在。
    #[error("无法切换路径到 {path}，目录不存在")]
    PathNotFound {
        /// 尝试访问的路径。
        path: String,
    },

    /// 路径为空。
    #[error("路径为空")]
    EmptyPath,

    /// 路径越界（超出根目录）。
    #[error("路径越界: {path}")]
    OutOfBounds {
        /// 尝试访问的路径。
        path: String,
    },
}

impl ErrorPathException {
    /// 创建路径不存在异常。
    pub fn not_found(path: impl Into<String>) -> Self {
        Self::PathNotFound { path: path.into() }
    }

    /// 创建路径为空异常。
    pub fn empty() -> Self {
        Self::EmptyPath
    }

    /// 创建路径越界异常。
    pub fn out_of_bounds(path: impl Into<String>) -> Self {
        Self::OutOfBounds { path: path.into() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_not_found() {
        let e = ErrorPathException::not_found("/Doc_0/Res");
        assert!(e.to_string().contains("/Doc_0/Res"));
        assert!(e.to_string().contains("不存在"));
    }

    #[test]
    fn test_empty_path() {
        let e = ErrorPathException::empty();
        assert!(e.to_string().contains("为空"));
    }

    #[test]
    fn test_out_of_bounds() {
        let e = ErrorPathException::out_of_bounds("../../..");
        assert!(e.to_string().contains("../../.."));
    }
}
