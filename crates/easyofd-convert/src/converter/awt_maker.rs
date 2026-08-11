//! AWT 渲染器（排除）。
//!
//! 对应 Java: org.ofdrw.converter.AWTMaker
//!
//! # 排除原因
//!
//! Java 版 `AWTMaker` 依赖 `java.awt.Graphics2D` 进行像素级渲染，
//! 包括字体渲染（`FontRenderContext`）、坐标变换（`AffineTransform`）、
//! 图片绘制（`BufferedImage`）等 AWT/Swing 专有 API。
//!
//! Rust 版使用 [`crate::exporter::SvgExporter`] 或
//! [`crate::exporter::ImageExporter`] 替代。
//!
//! 注意：[`crate::converter::config::Config`] 结构体对应
//! `AWTMaker` 内部的 `Config` 配置类，已独立移植。

/// AWT 渲染器占位。
///
/// 对应 Java: `org.ofdrw.converter.AWTMaker`
///
/// **排除**: 依赖 Java AWT，使用 `SvgExporter` / `ImageExporter` 替代。
#[derive(Debug, Clone, Copy)]
pub struct AWTMaker;

impl AWTMaker {
    /// 返回替代实现的名称。
    #[must_use]
    pub fn replacement() -> &'static str {
        "easyofd_convert::exporter::SvgExporter / ImageExporter"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_awt_maker_exclusion_doc() {
        assert!(AWTMaker::replacement().contains("SvgExporter"));
    }
}
