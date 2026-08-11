//! 画布渲染器。
//!
//! 对应 Java: org.ofdrw.layout.engine.render.CanvasRender
//!
//! 将 Canvas 绘图指令渲染为 OFD 路径和文本对象。
//! 此为简化结构，记录渲染器标识。

/// 画布渲染器。
///
/// 对应 Java: `org.ofdrw.layout.engine.render.CanvasRender`
///
/// 负责将画布（Canvas）绘图上下文中的指令转换为 OFD CT_PageBlock。
/// Rust 版本为简化结构，实际渲染逻辑由布局引擎统一调度。
#[derive(Debug, Clone)]
pub struct CanvasRender {
    /// 渲染器名称。
    name: String,
}

impl CanvasRender {
    /// 创建画布渲染器。
    #[must_use]
    pub fn new() -> Self {
        Self {
            name: "CanvasRender".to_string(),
        }
    }

    /// 获取渲染器名称。
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Default for CanvasRender {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_values() {
        let render = CanvasRender::new();
        assert_eq!(render.name(), "CanvasRender");
    }
}
