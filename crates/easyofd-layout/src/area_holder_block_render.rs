//! 区域占位块渲染器。
//!
//! 对应 Java: org.ofdrw.layout.engine.render.AreaHolderBlockRender
//!
//! 将区域占位块渲染为 OFD 页面内容。
//! 此为简化结构，记录渲染器标识。

/// 区域占位块渲染器。
///
/// 对应 Java: `org.ofdrw.layout.engine.render.AreaHolderBlockRender`
///
/// 负责将 [`crate::area_holder_block::AreaHolderBlock`] 渲染为 OFD CT_PageBlock。
/// Rust 版本为简化结构，实际渲染逻辑由布局引擎统一调度。
#[derive(Debug, Clone)]
pub struct AreaHolderBlockRender {
    /// 渲染器名称。
    name: String,
}

impl AreaHolderBlockRender {
    /// 创建区域占位块渲染器。
    #[must_use]
    pub fn new() -> Self {
        Self {
            name: "AreaHolderBlockRender".to_string(),
        }
    }

    /// 获取渲染器名称。
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Default for AreaHolderBlockRender {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_values() {
        let render = AreaHolderBlockRender::new();
        assert_eq!(render.name(), "AreaHolderBlockRender");
    }
}
