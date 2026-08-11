//! 错误 OFD 文件结构和文档格式异常。
//!
//! 对应 Java: org.ofdrw.reader.BadOFDException

use thiserror::Error;

/// 错误 OFD 文件结构和文档格式异常。
///
/// 当 OFD 文档结构损坏、格式不符合 GB/T 33190 规范，
/// 或解析过程中遇到不可恢复的错误时抛出。
///
/// 对应 Java: `org.ofdrw.reader.BadOFDException`
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum BadOfdException {
    /// 文档结构损坏。
    #[error("错误OFD结构和文件格式: {message}")]
    CorruptedStructure {
        /// 错误描述。
        message: String,
    },

    /// 文档解析失败。
    #[error("OFD解析失败: {message}")]
    ParseFailed {
        /// 错误描述。
        message: String,
    },

    /// 文件不存在。
    #[error("文件不存在: {path}")]
    FileNotFound {
        /// 文件路径。
        path: String,
    },

    /// 底层 IO 或 ZIP 错误。
    #[error(transparent)]
    Other(#[from] easyofd_core::OfdError),
}

impl BadOfdException {
    /// 创建文档结构损坏异常。
    pub fn corrupted(message: impl Into<String>) -> Self {
        Self::CorruptedStructure {
            message: message.into(),
        }
    }

    /// 创建解析失败异常。
    pub fn parse_failed(message: impl Into<String>) -> Self {
        Self::ParseFailed {
            message: message.into(),
        }
    }

    /// 创建文件不存在异常。
    pub fn file_not_found(path: impl Into<String>) -> Self {
        Self::FileNotFound { path: path.into() }
    }
}

impl From<BadOfdException> for easyofd_core::OfdError {
    fn from(e: BadOfdException) -> Self {
        match e {
            BadOfdException::CorruptedStructure { message } => {
                easyofd_core::OfdError::InvalidDocument(message)
            }
            BadOfdException::ParseFailed { message } => {
                easyofd_core::OfdError::InvalidDocument(message)
            }
            BadOfdException::FileNotFound { path } => easyofd_core::OfdError::Zip(path),
            BadOfdException::Other(inner) => inner,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_corrupted_structure() {
        let e = BadOfdException::corrupted("missing DocRoot");
        assert!(e.to_string().contains("missing DocRoot"));
    }

    #[test]
    fn test_parse_failed() {
        let e = BadOfdException::parse_failed("invalid XML");
        assert!(e.to_string().contains("invalid XML"));
    }

    #[test]
    fn test_file_not_found() {
        let e = BadOfdException::file_not_found("/Doc_0/missing.xml");
        assert!(e.to_string().contains("/Doc_0/missing.xml"));
    }

    #[test]
    fn test_from_ofd_error() {
        let ofd_err = easyofd_core::OfdError::Zip("bad zip".into());
        let bad: BadOfdException = ofd_err.into();
        assert!(matches!(bad, BadOfdException::Other(_)));
    }

    #[test]
    fn test_into_ofd_error() {
        let bad = BadOfdException::corrupted("test");
        let ofd_err: easyofd_core::OfdError = bad.into();
        assert!(matches!(ofd_err, easyofd_core::OfdError::InvalidDocument(_)));
    }
}
