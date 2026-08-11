//! HTML 渲染器（排除）。
//!
//! 对应 Java: org.ofdrw.converter.HtmlMaker
//!
//! # 排除原因
//!
//! Java 版 `HtmlMaker` 依赖 `java.awt.Graphics2D` 进行 HTML 渲染，
//! 与 `SVGMaker` 类似，使用 AWT 渲染管线将 OFD 页面绘制为 HTML+CSS。
//!
//! Rust 版使用 [`crate::html::Element`] 提供 HTML 元素构建能力，
//! 或使用 [`crate::exporter::SvgExporter`] 输出 SVG 格式。
//!
//! # 对应关系
//!
//! | Java 类 | Rust 替代 |
//! |---------|-----------|
//! | `HtmlMaker` | `html::Element` + `SvgExporter` |

/// HTML 渲染器占位。
///
/// 对应 Java: `org.ofdrw.converter.HtmlMaker`
///
/// **排除**: 依赖 Java AWT，使用 `html::Element` + `SvgExporter` 替代。
#[derive(Debug, Clone, Copy)]
pub struct HtmlMaker;

impl HtmlMaker {
    /// 返回替代实现的名称。
    #[must_use]
    pub fn replacement() -> &'static str {
        "easyofd_convert::html::Element + easyofd_convert::exporter::SvgExporter"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_html_maker_exclusion_doc() {
        assert!(HtmlMaker::replacement().contains("Element"));
    }
}
