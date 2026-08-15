/// 支持的签名算法。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignatureAlgorithm {
    /// SM2 签名配合 SM3 摘要（GM/T 0009）。
    Sm2WithSm3,
    /// SHA-256 摘要配合 RSA 签名。
    Sha256WithRsa,
}
