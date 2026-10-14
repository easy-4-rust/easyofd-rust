//! 裸SM2签名验证容器。
use crate::errors::OfdVerifyException;
use crate::verify_containers::SignedDataValidateContainer;
use sm2::dsa::signature::Verifier;
/// 裸SM2验证器。对应Java: org.ofdrw.sign.verify.container.DigitalValidateContainer。
pub struct DigitalValidateContainer {
    public_key_bytes: Vec<u8>,
}
impl DigitalValidateContainer {
    /// 参数为SEC1编码公钥；返回验证容器，不将非空配置视为验签成功。
    #[must_use]
    pub fn new(public_key_bytes: Vec<u8>) -> Self {
        Self { public_key_bytes }
    }
    /// 返回配置的公钥SEC1字节。
    #[must_use]
    pub fn public_key_bytes(&self) -> &[u8] {
        &self.public_key_bytes
    }
}
/// 使用标准SM2用户ID校验原文与DER/raw签名；失败返回验证错误。
pub(crate) fn verify(
    public: &[u8],
    message: &[u8],
    value: &[u8],
) -> Result<(), OfdVerifyException> {
    let key = sm2::dsa::VerifyingKey::from_sec1_bytes(
        crate::sign_containers::SM2_DEFAULT_USER_ID,
        public,
    )
    .map_err(|e| OfdVerifyException::new(format!("SM2公钥错误: {e}")))?;
    let signature = sm2::dsa::Signature::from_der(value)
        .or_else(|_| sm2::dsa::Signature::from_slice(value))
        .map_err(|e| OfdVerifyException::new(format!("SM2签名格式错误: {e}")))?;
    key.verify(message, &signature)
        .map_err(|e| OfdVerifyException::new(format!("SM2验签失败: {e}")))
}
impl SignedDataValidateContainer for DigitalValidateContainer {
    fn validate(&self, message: &[u8], value: &[u8]) -> Result<(), OfdVerifyException> {
        if value.len() > 256 {
            return Err(OfdVerifyException::new("裸签名长度超限"));
        }
        // 优先保留binary DER/raw语义；HEX必须是完整偶数长度ASCII编码。
        if verify(&self.public_key_bytes, message, value).is_ok() {
            return Ok(());
        }
        let hex = std::str::from_utf8(value)
            .map_err(|_| OfdVerifyException::new("签名不是DER/raw或HEX"))?;
        let bytes = crate::internal_helpers::unhex(hex)
            .map_err(|_| OfdVerifyException::new("HEX签名无效"))?;
        verify(&self.public_key_bytes, message, &bytes)
    }
}
