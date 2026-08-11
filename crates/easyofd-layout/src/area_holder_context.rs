//! 区域占位上下文。
//!
//! 对应 Java: org.ofdrw.layout.areaholder.AreaHolderContext
//!
//! 在布局过程中传递占位区域的上下文信息。

use std::collections::HashMap;

/// 区域占位上下文。
///
/// 对应 Java: `org.ofdrw.layout.areaholder.AreaHolderContext`
///
/// 在布局渲染过程中，维护已解析的占位区域名称到边界框的映射。
/// 渲染器可以通过上下文查找某个占位区域的实际位置和尺寸。
#[derive(Debug, Default)]
pub struct AreaHolderContext {
    /// 区域名称 → 边界框 "x y width height"（mm）。
    boundaries: HashMap<String, String>,
}

impl AreaHolderContext {
    /// 创建空上下文。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 注册一个区域的边界框。
    pub fn set_boundary(&mut self, name: impl Into<String>, boundary: impl Into<String>) {
        self.boundaries.insert(name.into(), boundary.into());
    }

    /// 获取区域的边界框。
    #[must_use]
    pub fn get_boundary(&self, name: &str) -> Option<&str> {
        self.boundaries.get(name).map(|s| s.as_str())
    }

    /// 是否包含指定区域。
    #[must_use]
    pub fn contains(&self, name: &str) -> bool {
        self.boundaries.contains_key(name)
    }

    /// 已注册区域数量。
    #[must_use]
    pub fn len(&self) -> usize {
        self.boundaries.len()
    }

    /// 是否为空。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.boundaries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_is_empty() {
        let ctx = AreaHolderContext::new();
        assert!(ctx.is_empty());
    }

    #[test]
    fn set_and_get() {
        let mut ctx = AreaHolderContext::new();
        ctx.set_boundary("header", "0 0 210 20");
        ctx.set_boundary("footer", "0 277 210 20");

        assert_eq!(ctx.get_boundary("header"), Some("0 0 210 20"));
        assert_eq!(ctx.get_boundary("footer"), Some("0 277 210 20"));
        assert!(ctx.get_boundary("missing").is_none());
    }

    #[test]
    fn contains_check() {
        let mut ctx = AreaHolderContext::new();
        ctx.set_boundary("a", "0 0 10 10");
        assert!(ctx.contains("a"));
        assert!(!ctx.contains("b"));
    }

    #[test]
    fn len() {
        let mut ctx = AreaHolderContext::new();
        assert_eq!(ctx.len(), 0);
        ctx.set_boundary("a", "0 0 10 10");
        ctx.set_boundary("b", "0 0 10 10");
        assert_eq!(ctx.len(), 2);
    }
}
