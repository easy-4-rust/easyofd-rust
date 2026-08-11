//! 签名与验证异常类型。
//!
//! 对应 Java: `org.ofdrw.sign` 及 `org.ofdrw.sign.verify.exceptions`
//!
//! 提供签名流程和验证流程中使用的错误类型层次结构。

use std::fmt;

/// 电子签名通用异常。
///
/// 对应 Java: `org.ofdrw.sign.SignatureException`
#[derive(Debug)]
pub struct SignatureException {
    /// 错误描述。
    message: String,
    /// 可选的底层原因。
    source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl SignatureException {
    /// 创建签名异常。
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            source: None,
        }
    }

    /// 创建带底层原因的签名异常。
    pub fn with_source(
        message: impl Into<String>,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }
}

impl fmt::Display for SignatureException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "签名异常: {}", self.message)
    }
}

impl std::error::Error for SignatureException {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source.as_ref().map(|e| e.as_ref() as _)
    }
}

/// 签名终止异常。
///
/// 对应 Java: `org.ofdrw.sign.SignatureTerminateException`
///
/// 表示该文档不允许再进行签名（例如已整体保护）。
#[derive(Debug)]
pub struct SignatureTerminateException {
    inner: SignatureException,
}

impl SignatureTerminateException {
    /// 创建签名终止异常。
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            inner: SignatureException::new(message),
        }
    }
}

impl fmt::Display for SignatureTerminateException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "签名终止: {}", self.inner.message)
    }
}

impl std::error::Error for SignatureTerminateException {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.inner.source()
    }
}

/// OFD 验证异常基类。
///
/// 对应 Java: `org.ofdrw.sign.verify.exceptions.OFDVerifyException`
#[derive(Debug)]
pub struct OfdVerifyException {
    /// 错误描述。
    message: String,
    /// 可选的底层原因。
    source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl OfdVerifyException {
    /// 创建验证异常。
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            source: None,
        }
    }

    /// 创建带底层原因的验证异常。
    pub fn with_source(
        message: impl Into<String>,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }
}

impl fmt::Display for OfdVerifyException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "OFD 验证异常: {}", self.message)
    }
}

impl std::error::Error for OfdVerifyException {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source.as_ref().map(|e| e.as_ref() as _)
    }
}

/// 电子签名数据失效异常。
///
/// 对应 Java: `org.ofdrw.sign.verify.exceptions.InvalidSignedValueException`
#[derive(Debug)]
pub struct InvalidSignedValueException {
    /// 失效原因。
    reason: String,
    /// 状态码。
    code: Option<i32>,
}

impl InvalidSignedValueException {
    /// 创建签名数据失效异常。
    #[must_use]
    pub fn new(reason: impl Into<String>) -> Self {
        Self {
            reason: reason.into(),
            code: None,
        }
    }

    /// 设置状态码。
    #[must_use]
    pub fn with_code(mut self, code: i32) -> Self {
        self.code = Some(code);
        self
    }

    /// 获取失效原因。
    #[must_use]
    pub fn reason(&self) -> &str {
        &self.reason
    }

    /// 获取状态码。
    #[must_use]
    pub fn code(&self) -> Option<i32> {
        self.code
    }
}

impl fmt::Display for InvalidSignedValueException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "电子签章数据失效: {}", self.reason)
    }
}

impl std::error::Error for InvalidSignedValueException {}

/// 文件完整性验证异常。
///
/// 对应 Java: `org.ofdrw.sign.verify.exceptions.FileIntegrityException`
#[derive(Debug)]
pub struct FileIntegrityException {
    /// 被篡改文件在 OFD 容器中的绝对路径。
    file_abs_path: String,
    /// 预期的文件杂凑值。
    expected_hash: Vec<u8>,
    /// 实际的文件杂凑值。
    actual_hash: Vec<u8>,
}

impl FileIntegrityException {
    /// 创建文件完整性异常。
    #[must_use]
    pub fn new(
        file_abs_path: impl Into<String>,
        expected_hash: Vec<u8>,
        actual_hash: Vec<u8>,
    ) -> Self {
        Self {
            file_abs_path: file_abs_path.into(),
            expected_hash,
            actual_hash,
        }
    }

    /// 获取文件路径。
    #[must_use]
    pub fn file_path(&self) -> &str {
        &self.file_abs_path
    }

    /// 获取预期杂凑值。
    #[must_use]
    pub fn expected_hash(&self) -> &[u8] {
        &self.expected_hash
    }

    /// 获取实际杂凑值。
    #[must_use]
    pub fn actual_hash(&self) -> &[u8] {
        &self.actual_hash
    }
}

impl fmt::Display for FileIntegrityException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "文件被篡改: {}", self.file_abs_path)
    }
}

impl std::error::Error for FileIntegrityException {}

/// 文件未签章异常。
///
/// 对应 Java: `org.ofdrw.sign.verify.exceptions.DocNotSignException`
#[derive(Debug)]
pub struct DocNotSignException {
    /// 错误描述。
    message: String,
}

impl DocNotSignException {
    /// 创建未签章异常。
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl Default for DocNotSignException {
    fn default() -> Self {
        Self {
            message: "文档未签章".into(),
        }
    }
}

impl fmt::Display for DocNotSignException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "文件未签章: {}", self.message)
    }
}

impl std::error::Error for DocNotSignException {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn signature_exception_display() {
        let e = SignatureException::new("签名失败");
        assert_eq!(e.to_string(), "签名异常: 签名失败");
        assert!(e.source().is_none());
    }

    #[test]
    fn signature_exception_with_source() {
        let inner = std::io::Error::other("IO error");
        let e = SignatureException::with_source("读取失败", inner);
        assert!(e.source().is_some());
    }

    #[test]
    fn signature_terminate_exception() {
        let e = SignatureTerminateException::new("文档已整体保护");
        assert_eq!(e.to_string(), "签名终止: 文档已整体保护");
    }

    #[test]
    fn ofd_verify_exception() {
        let e = OfdVerifyException::new("验证失败");
        assert_eq!(e.to_string(), "OFD 验证异常: 验证失败");
    }

    #[test]
    fn invalid_signed_value_exception() {
        let e = InvalidSignedValueException::new("签名值不匹配").with_code(403);
        assert_eq!(e.to_string(), "电子签章数据失效: 签名值不匹配");
        assert_eq!(e.reason(), "签名值不匹配");
        assert_eq!(e.code(), Some(403));
    }

    #[test]
    fn file_integrity_exception() {
        let e =
            FileIntegrityException::new("/Doc_0/Content.xml", vec![0x01, 0x02], vec![0x03, 0x04]);
        assert_eq!(e.to_string(), "文件被篡改: /Doc_0/Content.xml");
        assert_eq!(e.file_path(), "/Doc_0/Content.xml");
        assert_eq!(e.expected_hash(), &[0x01, 0x02]);
        assert_eq!(e.actual_hash(), &[0x03, 0x04]);
    }

    #[test]
    fn doc_not_sign_exception() {
        let e = DocNotSignException::default();
        assert_eq!(e.to_string(), "文件未签章: 文档未签章");

        let e = DocNotSignException::new("无签名目录");
        assert_eq!(e.to_string(), "文件未签章: 无签名目录");
    }
}
