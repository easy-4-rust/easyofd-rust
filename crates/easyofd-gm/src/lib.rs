//! # `easyofd-gm`
//!
//! 国密算法（SM2 / SM3 / SM4）集成模块，对应 Java 版 [`ofdrw-gm`](https://github.com/ofdrw/ofdrw) 子项目。
//!
//! ## 已实现功能
//!
//! - [`ses`] — SES（Secure Electronic Seal）电子印章 ASN.1 结构定义
//!   （V1 / V4 / V5 版本），实现 GB/T 38540-2020 标准。
//! - [`sm2_struct`] — GB/T 35275 SM2 签名数据结构（SignedData /
//!   ContentInfo / SignerInfo / IssuerAndSerialNumber / SM2Cipher / OIDs），
//!   对应 Java `org.ofdrw.gm.sm2strut` 包。
//!
//! ## 规划功能
//!
//! - SM2 非对称加密 / 签名
//! - SM3 摘要算法
//! - SM4 对称加密
//! - 与 `easyofd-signature` 协作完成国密签名流程

/// 返回模块标识，用于运行时识别。
#[must_use]
pub fn module_name() -> &'static str {
    "easyofd-gm"
}

/// 证书工具（对应 Java: org.ofdrw.gm.cert）。
pub mod cert;
/// SES（Secure Electronic Seal）电子印章 ASN.1 结构定义。
pub mod ses;
/// GB/T 35275 SM2 签名数据结构（对应 Java: org.ofdrw.gm.sm2strut）。
pub mod sm2_struct;
/// 国密算法支持工具（对应 Java: org.ofdrw.gm.support）。
pub mod support;

/// ofdrw-gm Java 类名兼容别名（Java 名 → Rust 名）。
///
/// 详见 [`compat`] 模块文档。
pub mod compat;

/// 对应 Java: GBT35275Validate（Rust 命名别名）。
pub type GBT35275Validate = crate::sm2_struct::builder::Gbt35275Validate;

/// 对应 Java: PKCS9SignedDataBuilder（Rust 命名别名）。
pub type PKCS9SignedDataBuilder = crate::sm2_struct::builder::Pkcs9SignedDataBuilder;

#[cfg(test)]
#[allow(clippy::items_after_statements)]
mod tests {
    use super::*;

    #[test]
    fn test_module_name() {
        assert_eq!(module_name(), "easyofd-gm");
    }
}
