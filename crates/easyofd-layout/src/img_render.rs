//! 图片渲染器。
//!
//! 对应 Java: org.ofdrw.layout.engine.render.ImgRender
//!
//! 将图片元素渲染为 OFD 图像对象。
//! 此为简化结构，记录渲染器标识。

/// 图片渲染器。
///
/// 对应 Java: `org.ofdrw.layout.engine.render.ImgRender`
///
/// 负责将图片元素渲染为 OFD CT_DrawImage。
/// Rust 版本为简化结构，实际渲染逻辑由布局引擎统一调度。
#[derive(Debug, Clone)]
pub struct ImgRender {
    /// 渲染器名称。
    name: String,
}

impl ImgRender {
    /// 创建图片渲染器。
    #[must_use]
    pub fn new() -> Self {
        Self {
            name: "ImgRender".to_string(),
        }
    }

    /// 获取渲染器名称。
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Default for ImgRender {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_values() {
        let render = ImgRender::new();
        assert_eq!(render.name(), "ImgRender");
    }
}
