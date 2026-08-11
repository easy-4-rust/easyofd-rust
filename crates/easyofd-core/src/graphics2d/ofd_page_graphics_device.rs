//! OFD 页面图形设备。
//!
//! 对应 Java: org.ofdrw.graphics2d.OFDPageGraphicsDevice
//!
//! Java 版继承 `java.awt.GraphicsDevice`，代表物理/逻辑显示设备。
//! Rust 版提供简化结构，保留设备标识与类型。

use super::OfdPageGraphicsConfiguration;

/// 图形设备类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphicsDeviceType {
    /// 屏幕显示。
    Screen,
    /// 打印机。
    Printer,
    /// 图像缓冲区。
    Image,
}

/// OFD 页面图形设备。
///
/// 对应 Java: org.ofdrw.graphics2d.OFDPageGraphicsDevice
///
/// 描述 OFD 输出的目标设备。Java 版依赖 AWT GraphicsDevice；
/// Rust 版保留设备类型和默认配置。
#[derive(Debug, Clone)]
pub struct OfdPageGraphicsDevice {
    /// 设备类型。
    pub device_type: GraphicsDeviceType,
    /// 设备名称。
    pub name: Option<String>,
    /// 默认图形配置。
    pub configuration: OfdPageGraphicsConfiguration,
}

impl OfdPageGraphicsDevice {
    /// 创建默认图像缓冲设备。
    #[must_use]
    pub fn new() -> Self {
        Self {
            device_type: GraphicsDeviceType::Image,
            name: None,
            configuration: OfdPageGraphicsConfiguration::new(),
        }
    }

    /// 创建指定类型的设备。
    #[must_use]
    pub fn with_type(device_type: GraphicsDeviceType) -> Self {
        Self {
            device_type,
            name: None,
            configuration: OfdPageGraphicsConfiguration::new(),
        }
    }

    /// 设置设备名称。
    #[must_use]
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// 设置图形配置。
    #[must_use]
    pub fn configuration(mut self, config: OfdPageGraphicsConfiguration) -> Self {
        self.configuration = config;
        self
    }

    /// 获取设备类型。
    #[must_use]
    pub fn device_type(&self) -> GraphicsDeviceType {
        self.device_type
    }
}

impl Default for OfdPageGraphicsDevice {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_default() {
        let d = OfdPageGraphicsDevice::new();
        assert_eq!(d.device_type(), GraphicsDeviceType::Image);
        assert!(d.name.is_none());
    }

    #[test]
    fn test_with_type() {
        let d = OfdPageGraphicsDevice::with_type(GraphicsDeviceType::Screen);
        assert_eq!(d.device_type(), GraphicsDeviceType::Screen);
    }

    #[test]
    fn test_builder() {
        let config = OfdPageGraphicsConfiguration::new().dpi(300);
        let d = OfdPageGraphicsDevice::new()
            .name("Printer1")
            .configuration(config);
        assert_eq!(d.name.as_deref(), Some("Printer1"));
        assert_eq!(d.configuration.dpi_x, 300);
    }

    #[test]
    fn test_device_type_variants() {
        assert_ne!(GraphicsDeviceType::Screen, GraphicsDeviceType::Printer);
        assert_ne!(GraphicsDeviceType::Printer, GraphicsDeviceType::Image);
    }

    #[test]
    fn test_default() {
        let d = OfdPageGraphicsDevice::default();
        assert_eq!(d.device_type(), GraphicsDeviceType::Image);
    }
}
