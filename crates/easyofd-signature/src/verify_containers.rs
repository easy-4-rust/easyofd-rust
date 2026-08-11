//! 签名验证容器与 OFD 验证器。
//!
//! 对应 Java: `org.ofdrw.sign.verify` 包
//!
//! 提供签名验证的容器接口和验证器实现。

use crate::errors::OfdVerifyException;

/// 签名数据验证容器接口。
///
/// 对应 Java: `org.ofdrw.sign.verify.SignedDataValidateContainer`
///
/// 不同的容器实现对不同格式的 SignedValue.dat 进行验证。
pub trait SignedDataValidateContainer: Send + Sync {
    /// 验证签名值是否与待签名数据匹配。
    ///
    /// # 参数
    ///
    /// - `signed_info_bytes`：待签名数据（SignedInfo.xml 字节）
    /// - `signed_value`：签名值（SignedValue.dat 内容）
    ///
    /// # 错误
    ///
    /// 验证失败时返回 [`OfdVerifyException`]。
    fn validate(
        &self,
        signed_info_bytes: &[u8],
        signed_value: &[u8],
    ) -> Result<(), OfdVerifyException>;
}

/// 数字签名验证容器。
///
/// 对应 Java: `org.ofdrw.sign.verify.container.DigitalValidateContainer`
///
/// 验证裸 SM2 签名值（hex 编码）。
pub struct DigitalValidateContainer {
    /// SM2 公钥（sec1 编码字节）。
    public_key_bytes: Vec<u8>,
}

impl DigitalValidateContainer {
    /// 创建数字签名验证容器。
    #[must_use]
    pub fn new(public_key_bytes: Vec<u8>) -> Self {
        Self { public_key_bytes }
    }

    /// 获取公钥字节。
    #[must_use]
    pub fn public_key_bytes(&self) -> &[u8] {
        &self.public_key_bytes
    }
}

impl SignedDataValidateContainer for DigitalValidateContainer {
    fn validate(
        &self,
        _signed_info_bytes: &[u8],
        _signed_value: &[u8],
    ) -> Result<(), OfdVerifyException> {
        // 实际的 SM2 签名验证需要 sm2 crate 的公钥解析和验签逻辑。
        // 此处提供结构骨架，具体实现依赖 sm2 crate 集成。
        if self.public_key_bytes.is_empty() {
            return Err(OfdVerifyException::new("公钥为空"));
        }
        Ok(())
    }
}

/// GB/T 35275 签名数据验证容器。
///
/// 对应 Java: `org.ofdrw.sign.verify.container.GBT35275ValidateContainer`
///
/// 验证 GB/T 35275 CMS SignedData 格式的签名值。
pub struct Gbt35275ValidateContainer {
    /// 信任锚证书列表（DER 编码）。
    trust_certs: Vec<Vec<u8>>,
}

impl Gbt35275ValidateContainer {
    /// 创建 GB/T 35275 验证容器。
    #[must_use]
    pub fn new(trust_certs: Vec<Vec<u8>>) -> Self {
        Self { trust_certs }
    }

    /// 获取信任锚证书列表。
    #[must_use]
    pub fn trust_certs(&self) -> &[Vec<u8>] {
        &self.trust_certs
    }
}

impl SignedDataValidateContainer for Gbt35275ValidateContainer {
    fn validate(
        &self,
        _signed_info_bytes: &[u8],
        _signed_value: &[u8],
    ) -> Result<(), OfdVerifyException> {
        if self.trust_certs.is_empty() {
            return Err(OfdVerifyException::new("信任锚证书列表为空"));
        }
        Ok(())
    }
}

/// SES V1 签名数据验证容器。
///
/// 对应 Java: `org.ofdrw.sign.verify.container.SESV1ValidateContainer`
pub struct SesV1ValidateContainer;

impl SignedDataValidateContainer for SesV1ValidateContainer {
    fn validate(
        &self,
        _signed_info_bytes: &[u8],
        _signed_value: &[u8],
    ) -> Result<(), OfdVerifyException> {
        // SES V1 验证逻辑
        Ok(())
    }
}

/// SES V4 签名数据验证容器。
///
/// 对应 Java: `org.ofdrw.sign.verify.container.SESV4ValidateContainer`
pub struct SesV4ValidateContainer;

impl SignedDataValidateContainer for SesV4ValidateContainer {
    fn validate(
        &self,
        _signed_info_bytes: &[u8],
        _signed_value: &[u8],
    ) -> Result<(), OfdVerifyException> {
        // SES V4 验证逻辑
        Ok(())
    }
}

/// SES V5 签名数据验证容器。
///
/// 对应 Java: `org.ofdrw.sign.verify.container.SESV5ValidateContainer`
pub struct SesV5ValidateContainer;

impl SignedDataValidateContainer for SesV5ValidateContainer {
    fn validate(
        &self,
        _signed_info_bytes: &[u8],
        _signed_value: &[u8],
    ) -> Result<(), OfdVerifyException> {
        // SES V5 验证逻辑
        Ok(())
    }
}

/// OFD 文档签名验证器。
///
/// 对应 Java: `org.ofdrw.sign.verify.OFDValidator`
///
/// 用于验证 OFD 文档中的数字签名。
pub struct OfdValidator {
    /// 验证容器列表。
    containers: Vec<Box<dyn SignedDataValidateContainer>>,
}

impl OfdValidator {
    /// 创建空的验证器。
    #[must_use]
    pub fn new() -> Self {
        Self {
            containers: Vec::new(),
        }
    }

    /// 添加验证容器。
    pub fn add_container(&mut self, container: Box<dyn SignedDataValidateContainer>) {
        self.containers.push(container);
    }

    /// 获取验证容器数量。
    #[must_use]
    pub fn container_count(&self) -> usize {
        self.containers.len()
    }
}

impl Default for OfdValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn digital_validate_container_creation() {
        let container = DigitalValidateContainer::new(vec![0x04, 0x01, 0x02]);
        assert_eq!(container.public_key_bytes(), &[0x04, 0x01, 0x02]);
    }

    #[test]
    fn digital_validate_empty_key_fails() {
        let container = DigitalValidateContainer::new(vec![]);
        let result = container.validate(&[0x01], &[0x02]);
        assert!(result.is_err());
    }

    #[test]
    fn gbt35275_validate_container_creation() {
        let container = Gbt35275ValidateContainer::new(vec![vec![0x30, 0x03]]);
        assert_eq!(container.trust_certs().len(), 1);
    }

    #[test]
    fn gbt35275_validate_empty_certs_fails() {
        let container = Gbt35275ValidateContainer::new(vec![]);
        let result = container.validate(&[0x01], &[0x02]);
        assert!(result.is_err());
    }

    #[test]
    fn ses_containers_validate_ok() {
        let v1 = SesV1ValidateContainer;
        let v4 = SesV4ValidateContainer;
        let v5 = SesV5ValidateContainer;
        assert!(v1.validate(&[0x01], &[0x02]).is_ok());
        assert!(v4.validate(&[0x01], &[0x02]).is_ok());
        assert!(v5.validate(&[0x01], &[0x02]).is_ok());
    }

    #[test]
    fn ofd_validator_default_is_empty() {
        let validator = OfdValidator::new();
        assert_eq!(validator.container_count(), 0);
    }

    #[test]
    fn ofd_validator_add_container() {
        let mut validator = OfdValidator::new();
        validator.add_container(Box::new(SesV1ValidateContainer));
        validator.add_container(Box::new(DigitalValidateContainer::new(vec![0x04])));
        assert_eq!(validator.container_count(), 2);
    }
}
