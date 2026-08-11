//! 段落渲染器。
//!
//! 对应 Java: org.ofdrw.layout.engine.render.ParagraphRender
//!
//! 将段落元素渲染为 OFD 文本块。
//! 此为简化结构，记录渲染器标识。

/// 段落渲染器。
///
/// 对应 Java: `org.ofdrw.layout.engine.render.ParagraphRender`
///
/// 负责将段落元素渲染为 OFD CT_Text。
/// Rust 版本为简化结构，实际渲染逻辑由布局引擎统一调度。
#[derive(Debug, Clone)]
pub struct ParagraphRender {
    /// 渲染器名称。
    name: String,
}

impl ParagraphRender {
    /// 创建段落渲染器。
    #[must_use]
    pub fn new() -> Self {
        Self {
            name: "ParagraphRender".to_string(),
        }
    }

    /// 获取渲染器名称。
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Default for ParagraphRender {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_values() {
        let render = ParagraphRender::new();
        assert_eq!(render.name(), "ParagraphRender");
    }
}
