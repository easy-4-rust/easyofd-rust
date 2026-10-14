//! SES V1真实签名验证。
use crate::errors::OfdVerifyException;
use crate::verify_containers::SignedDataValidateContainer;
/// 对应Java: org.ofdrw.sign.verify.container.SESV1ValidateContainer。
/// 验证摘要和SM2密码签名，不替代证书信任链/吊销检查。
pub struct SesV1ValidateContainer;
impl SignedDataValidateContainer for SesV1ValidateContainer {
    /// 参数为原文和SES DER签名，返回验证结果。
    fn validate(&self, message: &[u8], value: &[u8]) -> Result<(), OfdVerifyException> {
        crate::ses_verification::validate(1, message, value)
    }
}
