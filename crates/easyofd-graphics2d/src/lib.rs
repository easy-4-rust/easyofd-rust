//! # `easyofd-graphics2d`
//!
//! OFD 2D 图形渲染抽象层，对应 Java 版 [`ofdrw-graphics2d`](https://github.com/ofdrw/ofdrw) 子项目。
//!
//! ## 模块结构
//!
//! - [`canvas`] — 2D 绘图画布，提供 `Graphics2D` 风格的绘图指令录制。
//! - [`converter`] — 将画布命令转译为 OFD [`OfdPage`] 上的 [`ContentObject`]。
//!
//! ## 快速上手
//!
//! ```rust
//! use easyofd_graphics2d::canvas::Canvas;
//! use easyofd_graphics2d::converter::canvas_to_page;
//!
//! let mut c = Canvas::new(210.0, 297.0);
//! c.rect(10.0, 10.0, 190.0, 30.0);
//! c.draw_text(20.0, 20.0, "标题", 24.0);
//! let page = canvas_to_page(&c);
//! assert_eq!(page.content.len(), 2);
//! ```

/// Canvas 2D API 辅助工厂。
pub mod awt_maker;
/// 2D 绘图画布。
pub mod canvas;
/// 裁剪区域工厂。
pub mod clip_factory;
/// Canvas 绘图命令到 OFD 页面的转换器。
pub mod converter;
/// 绘制参数缓存。
pub mod draw_param_cache;
/// 图片到 OFD ImageObject 转换器。
pub mod image_maker;

/// 返回模块标识，用于运行时识别。
#[must_use]
pub fn module_name() -> &'static str {
    "easyofd-graphics2d"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_name() {
        assert_eq!(module_name(), "easyofd-graphics2d");
    }
}
