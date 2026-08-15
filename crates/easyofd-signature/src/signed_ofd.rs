use easyofd_core::OfdResult;
use std::io::Write;

/// 签名结果。
#[derive(Debug)]
pub struct SignedOfd {
    pub(crate) data: Vec<u8>,
    /// 签名摘要值（Base64 编码）。
    pub digest: String,
    /// 签名值（Base64 编码）。
    pub signature_value: String,
}

impl SignedOfd {
    /// 将签名后的 OFD 保存到指定路径。
    ///
    /// # 错误
    ///
    /// 文件写入失败时返回错误。
    pub fn save(self, path: impl AsRef<std::path::Path>) -> OfdResult<()> {
        easyofd_package::atomic_write(path, |file| {
            file.write_all(&self.data)?;
            Ok(())
        })
    }
    /// 将签名后的 OFD 转为字节。
    #[must_use]
    pub fn into_bytes(self) -> Vec<u8> {
        self.data
    }
}
