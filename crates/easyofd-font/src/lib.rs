//! # `easyofd-font`
//!
//! OFD 字体管理与嵌入模块，对应 Java 版 [`ofdrw-font`](https://github.com/ofdrw/ofdrw) 子项目。
//!
//! ## 功能模块
//!
//! - [`font_descriptor`]：字体描述符，记录字体基本属性
//! - [`font_registry`]：字体注册表，管理字体 ID 到描述符的映射
//! - [`text_metrics`]：文本度量，估算文本的宽高
//!
//! ## 规划功能
//!
//! - TrueType / OpenType 字体文件解析
//! - 字体子集化（subsetting）以减小 OFD 体积
//! - 嵌入字体到 OFD 资源目录

/// 字体描述符，记录字体基本属性。
pub mod font_descriptor;
/// 字体注册表，管理字体 ID 到描述符的映射。
pub mod font_registry;
/// 文本度量，估算文本的宽高。
pub mod text_metrics;

/// 返回模块标识，用于运行时识别。
#[must_use]
pub fn module_name() -> &'static str {
    "easyofd-font"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_name() {
        assert_eq!(module_name(), "easyofd-font");
    }
}
