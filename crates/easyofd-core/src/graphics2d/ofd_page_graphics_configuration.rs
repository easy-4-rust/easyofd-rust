//! OFD 页面图形配置。
//!
//! 对应 Java: org.ofdrw.graphics2d.OFDPageGraphicsConfiguration
//!
//! Java 版继承 `java.awt.GraphicsConfiguration`，描述输出设备的
//! 显示能力（分辨率、色彩模型等）。Rust 版提供简化结构。

/// OFD 页面图形配置。
///
/// 对应 Java: org.ofdrw.graphics2d.OFDPageGraphicsConfiguration
///
/// 描述 OFD 页面的渲染配置：DPI、色彩模式等。
/// Java 版依赖 AWT GraphicsConfiguration；Rust 版保留核心配置字段。
#[derive(Debug, Clone)]
pub struct OfdPageGraphicsConfiguration {
    /// 水平 DPI（默认 96）。
    pub dpi_x: u32,
    /// 垂直 DPI（默认 96）。
    pub dpi_y: u32,
    /// 颜色位深度（默认 24）。
    pub color_depth: u32,
}

impl OfdPageGraphicsConfiguration {
    /// 创建默认配置（96 DPI，24 位色深）。
    #[must_use]
    pub fn new() -> Self {
        Self {
            dpi_x: 96,
            dpi_y: 96,
            color_depth: 24,
        }
    }

    /// 设置 DPI（水平和垂直）。
    #[must_use]
    pub fn dpi(mut self, dpi: u32) -> Self {
        self.dpi_x = dpi;
        self.dpi_y = dpi;
        self
    }

    /// 设置水平 DPI。
    #[must_use]
    pub fn dpi_x(mut self, dpi: u32) -> Self {
        self.dpi_x = dpi;
        self
    }

    /// 设置垂直 DPI。
    #[must_use]
    pub fn dpi_y(mut self, dpi: u32) -> Self {
        self.dpi_y = dpi;
        self
    }

    /// 设置颜色位深度。
    #[must_use]
    pub fn color_depth(mut self, depth: u32) -> Self {
        self.color_depth = depth;
        self
    }
}

impl Default for OfdPageGraphicsConfiguration {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_defaults() {
        let c = OfdPageGraphicsConfiguration::new();
        assert_eq!(c.dpi_x, 96);
        assert_eq!(c.dpi_y, 96);
        assert_eq!(c.color_depth, 24);
    }

    #[test]
    fn test_builder() {
        let c = OfdPageGraphicsConfiguration::new().dpi(300).color_depth(32);
        assert_eq!(c.dpi_x, 300);
        assert_eq!(c.dpi_y, 300);
        assert_eq!(c.color_depth, 32);
    }

    #[test]
    fn test_separate_dpi() {
        let c = OfdPageGraphicsConfiguration::new().dpi_x(150).dpi_y(200);
        assert_eq!(c.dpi_x, 150);
        assert_eq!(c.dpi_y, 200);
    }

    #[test]
    fn test_default() {
        let c = OfdPageGraphicsConfiguration::default();
        assert_eq!(c.dpi_x, 96);
    }
}
