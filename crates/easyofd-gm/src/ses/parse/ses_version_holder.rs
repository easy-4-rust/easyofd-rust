//! SES 版本持有器。
//!
//! 对应 Java: `org.ofdrw.gm.ses.parse.SESVersionHolder`

use super::SESVersion;

/// 版本持有器，包装解析后的版本号与原始 DER 字节。
///
/// 对应 Java: `org.ofdrw.gm.ses.parse.SESVersionHolder`
///
/// 持有 [`SESVersion`] 和对应的原始 SEQUENCE DER 字节，
/// 供上层按版本号分发解码。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SESVersionHolder {
    /// 解析出的版本。
    version: SESVersion,
    /// 原始 SEQUENCE 内部字节（不含 SEQUENCE 头）。
    obj_bytes: Vec<u8>,
}

impl SESVersionHolder {
    /// 创建版本持有器。
    #[must_use]
    pub fn new(version: SESVersion, obj_bytes: Vec<u8>) -> Self {
        Self { version, obj_bytes }
    }

    /// 获取版本号。
    #[must_use]
    pub fn version(&self) -> SESVersion {
        self.version
    }

    /// 获取原始 SEQUENCE 内部字节。
    #[must_use]
    pub fn obj_bytes(&self) -> &[u8] {
        &self.obj_bytes
    }

    /// 尝试将内容解码为 V1 `SESSignature`。
    ///
    /// 仅当版本为 V1 且字节合法时返回 `Some`。
    #[must_use]
    pub fn as_v1_signature(&self) -> Option<crate::ses::v1::SESSignature> {
        if self.version != SESVersion::V1 {
            return None;
        }
        // 重新包装为完整 SEQUENCE DER
        let mut der = Vec::with_capacity(self.obj_bytes.len() + 4);
        crate::ses::encode_sequence(&self.obj_bytes, &mut der);
        crate::ses::v1::SESSignature::decode_der(&der).ok()
    }

    /// 尝试将内容解码为 V4 `SESSignature`。
    ///
    /// 仅当版本为 V4 且字节合法时返回 `Some`。
    #[must_use]
    pub fn as_v4_signature(&self) -> Option<crate::ses::v4::SESSignature> {
        if self.version != SESVersion::V4 {
            return None;
        }
        let mut der = Vec::with_capacity(self.obj_bytes.len() + 4);
        crate::ses::encode_sequence(&self.obj_bytes, &mut der);
        crate::ses::v4::SESSignature::decode_der(&der).ok()
    }

    /// 尝试将内容解码为 V5 `SESSignature`。
    ///
    /// 仅当版本为 V5 且字节合法时返回 `Some`。
    #[must_use]
    pub fn as_v5_signature(&self) -> Option<crate::ses::v5::SESSignature> {
        if self.version != SESVersion::V5 {
            return None;
        }
        let mut der = Vec::with_capacity(self.obj_bytes.len() + 4);
        crate::ses::encode_sequence(&self.obj_bytes, &mut der);
        crate::ses::v5::SESSignature::decode_der(&der).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_and_accessors() {
        let holder = SESVersionHolder::new(SESVersion::V4, vec![0x01, 0x02]);
        assert_eq!(holder.version(), SESVersion::V4);
        assert_eq!(holder.obj_bytes(), &[0x01, 0x02]);
    }

    #[test]
    fn wrong_version_returns_none() {
        let holder = SESVersionHolder::new(SESVersion::V1, vec![]);
        assert!(holder.as_v4_signature().is_none());
        assert!(holder.as_v5_signature().is_none());
    }

    #[test]
    fn empty_bytes_returns_none() {
        let holder = SESVersionHolder::new(SESVersion::V1, vec![]);
        assert!(holder.as_v1_signature().is_none());
    }
}
