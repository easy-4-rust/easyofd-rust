//! 证书签名持有者。
//!
//! 对应 Java: org.ofdrw.gm.sm2strut.builder.CertSigHolder
//!
//! 持有证书 DER 数据与对应的签名值，用于构建 SignedData 的 signerInfos。
//! Java 版在 PKCS9SignedDataBuilder 中作为内部状态；Rust 版提取为独立结构。

/// 证书签名持有者。
///
/// 对应 Java: org.ofdrw.gm.sm2strut.builder.CertSigHolder
///
/// 将一个 X.509 证书与其对应的 SM2 签名值绑定在一起。
#[derive(Debug, Clone)]
pub struct CertSigHolder {
    /// 证书 DER 编码。
    pub cert_der: Vec<u8>,
    /// 签名值（SM2 encryptedDigest）。
    pub signature: Vec<u8>,
}

impl CertSigHolder {
    /// 创建新的证书签名持有者。
    #[must_use]
    pub fn new(cert_der: Vec<u8>, signature: Vec<u8>) -> Self {
        Self {
            cert_der,
            signature,
        }
    }

    /// 获取证书 DER 数据引用。
    #[must_use]
    pub fn cert_der(&self) -> &[u8] {
        &self.cert_der
    }

    /// 获取签名值引用。
    #[must_use]
    pub fn signature(&self) -> &[u8] {
        &self.signature
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let holder = CertSigHolder::new(vec![0x30, 0x82], vec![0xDE, 0xAD]);
        assert_eq!(holder.cert_der(), &[0x30, 0x82]);
        assert_eq!(holder.signature(), &[0xDE, 0xAD]);
    }

    #[test]
    fn test_clone_debug() {
        let h = CertSigHolder::new(vec![0x01], vec![0x02]);
        let h2 = h.clone();
        assert_eq!(h2.cert_der(), h.cert_der());
        assert!(format!("{h:?}").contains("CertSigHolder"));
    }
}
