use easyofd_core::OfdResult;
use std::io::Write;

/// 签名结果。
#[derive(Debug)]
pub struct SignedOfd {
    pub(crate) data: Vec<u8>,
    pub digest: String,
    pub signature_value: String,
}

impl SignedOfd {
    pub fn save(self, path: impl AsRef<std::path::Path>) -> OfdResult<()> {
        easyofd_package::atomic_write(path, |file| {
            file.write_all(&self.data)?;
            Ok(())
        })
    }
    #[must_use]
    pub fn into_bytes(self) -> Vec<u8> {
        self.data
    }
}
