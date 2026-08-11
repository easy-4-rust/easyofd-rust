//! DER 编码错误类型。

/// DER 编码错误类型。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DerError(pub &'static str);

impl std::fmt::Display for DerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DER encoding error: {}", self.0)
    }
}

impl std::error::Error for DerError {}

/// DER 编码结果。
pub type DerResult<T> = Result<T, DerError>;
