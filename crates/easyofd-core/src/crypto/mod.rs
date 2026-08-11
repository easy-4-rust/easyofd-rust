//! 加密子包。
//!
//! 对应 Java: org.ofdrw.core.crypto.encryt
//!
//! 提供 OFD 加密相关的数据类型：
//! - [`CryptoParameter`] — 加密参数
//! - [`SigParameter`] — 签名参数
//! - [`SigParameters`] — 签名参数列表
//! - [`UserInfo`] — 用户信息
//! - [`Encryptions`] — 加密信息列表
//! - [`ExtendParams`] — 扩展参数
//! - [`DecyptSeed`] — 解密种子
//! - [`CryptoProvider`] — 加密组件提供者

mod provider;

pub use provider::CryptoProvider;

/// 加密参数。
///
/// 对应 Java: org.ofdrw.core.crypto.encryt.Parameter
#[derive(Debug, Clone, Default)]
pub struct CryptoParameter {
    /// 参数名。
    pub name: String,
    /// 参数值。
    pub value: String,
}

impl CryptoParameter {
    /// 创建加密参数。
    #[must_use]
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
        }
    }
}

/// 签名参数。
///
/// 对应 Java: org.ofdrw.core.signatures.sig.Parameter
#[derive(Debug, Clone, Default)]
pub struct SigParameter {
    /// 参数名。
    pub name: String,
    /// 参数值。
    pub value: String,
}

impl SigParameter {
    /// 创建签名参数。
    #[must_use]
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
        }
    }
}

/// 签名参数列表。
///
/// 对应 Java: org.ofdrw.core.signatures.sig.Parameters
#[derive(Debug, Clone, Default)]
pub struct SigParameters {
    /// 参数列表。
    pub parameters: Vec<SigParameter>,
}

impl SigParameters {
    /// 创建空参数列表。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 添加参数。
    pub fn add(&mut self, param: SigParameter) {
        self.parameters.push(param);
    }

    /// 获取参数数量。
    #[must_use]
    pub fn len(&self) -> usize {
        self.parameters.len()
    }

    /// 是否为空。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.parameters.is_empty()
    }
}

/// 用户信息。
///
/// 对应 Java: org.ofdrw.core.crypto.encryt.UserInfo
#[derive(Debug, Clone, Default)]
pub struct UserInfo {
    /// 用户名。
    pub user_name: Option<String>,
    /// 用户证书。
    pub cert: Option<String>,
}

impl UserInfo {
    /// 创建用户信息。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置用户名。
    #[must_use]
    pub fn user_name(mut self, name: impl Into<String>) -> Self {
        self.user_name = Some(name.into());
        self
    }
}

/// 加密信息列表。
///
/// 对应 Java: org.ofdrw.core.crypto.encryt.Encryptions
#[derive(Debug, Clone, Default)]
pub struct Encryptions {
    /// 加密信息列表。
    pub items: Vec<String>,
}

impl Encryptions {
    /// 创建空列表。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 添加加密信息。
    pub fn add(&mut self, item: impl Into<String>) {
        self.items.push(item.into());
    }
}

/// 扩展参数。
///
/// 对应 Java: org.ofdrw.core.crypto.encryt.ExtendParams
#[derive(Debug, Clone, Default)]
pub struct ExtendParams {
    /// 参数列表。
    pub params: Vec<(String, String)>,
}

impl ExtendParams {
    /// 创建空扩展参数。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 添加参数。
    pub fn add(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.params.push((key.into(), value.into()));
    }
}

/// 解密种子。
///
/// 对应 Java: org.ofdrw.core.crypto.encryt.DecyptSeed
#[derive(Debug, Clone, Default)]
pub struct DecyptSeed {
    /// 种子值。
    pub seed: Vec<u8>,
}

impl DecyptSeed {
    /// 创建解密种子。
    #[must_use]
    pub fn new(seed: Vec<u8>) -> Self {
        Self { seed }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crypto_parameter_new() {
        let p = CryptoParameter::new("key", "value");
        assert_eq!(p.name, "key");
        assert_eq!(p.value, "value");
    }

    #[test]
    fn sig_parameter_new() {
        let p = SigParameter::new("algo", "SM2");
        assert_eq!(p.name, "algo");
    }

    #[test]
    fn sig_parameters_add() {
        let mut params = SigParameters::new();
        params.add(SigParameter::new("a", "1"));
        assert_eq!(params.len(), 1);
    }

    #[test]
    fn user_info_builder() {
        let ui = UserInfo::new().user_name("test");
        assert_eq!(ui.user_name.unwrap(), "test");
    }

    #[test]
    fn encryptions_add() {
        let mut e = Encryptions::new();
        e.add("entry1");
        assert_eq!(e.items.len(), 1);
    }

    #[test]
    fn extend_params_add() {
        let mut ep = ExtendParams::new();
        ep.add("k", "v");
        assert_eq!(ep.params.len(), 1);
    }

    #[test]
    fn decypt_seed_new() {
        let ds = DecyptSeed::new(vec![1, 2, 3]);
        assert_eq!(ds.seed.len(), 3);
    }
}
