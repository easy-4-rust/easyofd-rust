/// 支持的签名算法。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignatureAlgorithm {
    Sm2WithSm3,
    Sha256WithRsa,
}
