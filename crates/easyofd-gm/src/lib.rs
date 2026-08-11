//! # `easyofd-gm`
//!
//! 国密算法（SM2 / SM3 / SM4）集成模块，对应 Java 版 [`ofdrw-gm`](https://github.com/ofdrw/ofdrw) 子项目。
//!
//! ## 已实现功能
//!
//! - [`ses`] — SES（Secure Electronic Seal）电子印章 ASN.1 结构定义
//!   （V1 / V4 / V5 版本），实现 GB/T 38540-2020 标准。
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

/// SES（Secure Electronic Seal）电子印章 ASN.1 结构定义。
pub mod ses;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_name() {
        assert_eq!(module_name(), "easyofd-gm");
    }
}
