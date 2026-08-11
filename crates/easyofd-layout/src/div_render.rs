//! Div 渲染器。
//!
//! 对应 Java: org.ofdrw.layout.engine.render.DivRender
//!
//! 将 Div 盒式模型渲染为 OFD 页面内容对象。
//! 此为简化结构，记录渲染器标识与配置。

/// Div 渲染器。
///
/// 对应 Java: `org.ofdrw.layout.engine.render.DivRender`
///
/// 负责将 [`crate::div::Div`] 元素渲染为 OFD CT_PageBlock 内容。
/// Rust 版本为简化结构，实际渲染逻辑由布局引擎统一调度。
#[derive(Debug, Clone)]
pub struct DivRender {
    /// 渲染器名称。
    name: String,
    /// 是否启用边框渲染。
    render_border: bool,
    /// 是否启用背景渲染。
    render_background: bool,
}

impl DivRender {
    /// 创建默认 Div 渲染器。
    #[must_use]
    pub fn new() -> Self {
        Self {
            name: "DivRender".to_string(),
            render_border: true,
            render_background: true,
        }
    }

    /// 获取渲染器名称。
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// 是否启用边框渲染。
    #[must_use]
    pub fn render_border(&self) -> bool {
        self.render_border
    }

    /// 设置是否启用边框渲染。
    pub fn set_render_border(&mut self, enabled: bool) {
        self.render_border = enabled;
    }

    /// 是否启用背景渲染。
    #[must_use]
    pub fn render_background(&self) -> bool {
        self.render_background
    }

    /// 设置是否启用背景渲染。
    pub fn set_render_background(&mut self, enabled: bool) {
        self.render_background = enabled;
    }
}

impl Default for DivRender {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_values() {
        let render = DivRender::new();
        assert_eq!(render.name(), "DivRender");
        assert!(render.render_border());
        assert!(render.render_background());
    }

    #[test]
    fn setters() {
        let mut render = DivRender::new();
        render.set_render_border(false);
        render.set_render_background(false);
        assert!(!render.render_border());
        assert!(!render.render_background());
    }
}
