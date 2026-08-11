//! SVG 渲染器（排除）。
//!
//! 对应 Java: org.ofdrw.converter.SVGMaker
//!
//! # 排除原因
//!
//! Java 版 `SVGMaker` 依赖 `java.awt.Graphics2D` 进行 SVG 渲染，
//! 包括字体度量（`FontMetrics`）、坐标变换（`AffineTransform`）、
//! 图形上下文（`Graphics2D`）等 AWT 专有 API。
//!
//! Rust 版使用 [`crate::exporter::SvgExporter`] 替代，
//! 直接将 OFD 对象映射为 SVG XML 元素，不依赖 AWT 渲染管线。
//!
//! # 对应关系
//!
//! | Java 类 | Rust 替代 |
//! |---------|-----------|
//! | `SVGMaker` | `SvgExporter` |
//! | `SVGMaker.make()` | `SvgExporter.convert()` |

/// SVG 渲染器占位。
///
/// 对应 Java: `org.ofdrw.converter.SVGMaker`
///
/// **排除**: 依赖 Java AWT，使用 [`crate::exporter::SvgExporter`] 替代。
#[derive(Debug, Clone, Copy)]
pub struct SVGMaker;

impl SVGMaker {
    /// 返回替代实现的名称。
    #[must_use]
    pub fn replacement() -> &'static str {
        "easyofd_convert::exporter::SvgExporter"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_svg_maker_exclusion_doc() {
        assert_eq!(
            SVGMaker::replacement(),
            "easyofd_convert::exporter::SvgExporter"
        );
    }
}
