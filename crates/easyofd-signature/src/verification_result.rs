use crate::algorithm::SignatureAlgorithm;

/// 签名验证结果。
#[derive(Debug)]
pub struct VerificationResult {
    /// 签名是否有效（References 完整性 + SM2 密码学全部通过）。
    pub valid: bool,
    /// SignedInfo.xml 的 SM3 摘要值（SM2 实际签名的消息摘要）。
    pub digest: String,
    /// SM2 签名值（来自 SignedValue.dat，hex 编码）。
    pub signature_value: String,
    /// 公钥（十六进制 sec1 编码）。
    pub public_key: String,
    /// 签名算法。
    pub algorithm: SignatureAlgorithm,
    /// References/FileRef 列出的文件路径，重算 SM3 与 CheckValue 不一致的列表
    /// （GB/T 38540 完整性失败清单）。
    pub reference_failures: Vec<String>,
}
