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

pub use crate::digital_validate_container::DigitalValidateContainer;
pub use crate::gbt35275_validate_container::Gbt35275ValidateContainer;

pub use crate::ses_v1_validate_container::SesV1ValidateContainer;
pub use crate::ses_v4_validate_container::SesV4ValidateContainer;
pub use crate::ses_v5_validate_container::SesV5ValidateContainer;

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
    fn ses_containers_reject_invalid_der() {
        let v1 = SesV1ValidateContainer;
        let v4 = SesV4ValidateContainer;
        let v5 = SesV5ValidateContainer;
        assert!(v1.validate(&[0x01], &[0x02]).is_err());
        assert!(v4.validate(&[0x01], &[0x02]).is_err());
        assert!(v5.validate(&[0x01], &[0x02]).is_err());
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
