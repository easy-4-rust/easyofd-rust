//! 内部辅助函数：ZIP/IO 错误转换与 XML 转义。

#[allow(clippy::needless_pass_by_value)]
pub(crate) fn zip_err(e: zip::result::ZipError) -> easyofd_core::OfdError {
    easyofd_core::OfdError::Zip(format!("{e}"))
}

pub(crate) fn io_err(e: std::io::Error) -> easyofd_core::OfdError {
    easyofd_core::OfdError::Io(e)
}

/// 转义 XML 特殊字符。
#[cfg(test)]
pub(crate) fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
