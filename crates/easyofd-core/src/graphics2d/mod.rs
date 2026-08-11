//! OFD 2D 图形 API。
//!
//! 对应 Java: org.ofdrw.graphics2d
//!
//! Java 版依赖 `java.awt.Graphics2D` 体系实现 OFD 渲染。
//! Rust 版提供简化结构，保留核心元数据与绘制参数，
//! 不包含 AWT 渲染管线。具体绘制逻辑由 `easyofd-layout` 等上层 crate 实现。

mod ofd_graphics2d_draw_param;
mod ofd_graphics_document;
mod ofd_page_graphics2d;
mod ofd_page_graphics_configuration;
mod ofd_page_graphics_device;
mod ofd_shapes;

pub use ofd_graphics_document::OfdGraphicsDocument;
pub use ofd_graphics2d_draw_param::OfdGraphics2DDrawParam;
pub use ofd_page_graphics_configuration::OfdPageGraphicsConfiguration;
pub use ofd_page_graphics_device::{GraphicsDeviceType, OfdPageGraphicsDevice};
pub use ofd_page_graphics2d::OfdPageGraphics2D;
pub use ofd_shapes::{OfdShape, OfdShapes};
